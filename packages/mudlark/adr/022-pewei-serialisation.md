# ADR-M-022: PEWEI Serialisation

**Status:** Decided  
**Date:** 2026-02-26  
**Phase:** 4 (Output)  
**Relates to:** [ADR-M-008](008-span-type.md) (view types),
[ADR-M-021](021-pewei-output-representation.md) (output representation),
[ADR-M-026](026-point-query-and-plateau-semantics.md) (plateau types),
[ADR-M-032](032-three-surface-model.md) (three-surface model)  
**Spec:** §PEWEI M-2 (structure — two-population layer model)  
**API:** [§API M-7.3](../docs/api.md#73-feature-gates) (feature gates)  
**Surface:** 1 (Prints)

## Context

The `Pewei<C, V>` type is an owned, static snapshot — a natural
candidate for serialisation. Users will want to persist snapshots,
transmit them over the network, or compare snapshots from different
time points. The same argument applies to all Surface 1 snapshot
types (ADR-M-032 §Prints): view types (`Span`, `Cell`, `Node`,
`GState`), PEWEI output types (`Pewei`, `Layer`, `Transition`,
`Terminal`), and plateau types (`BasisEdge`, `Plateau`). All are
pure-data snapshots with no references back into the graph.

The crate currently has zero required dependencies beyond `tracing`.
`serde` is a dev-dependency only (used in test harness
serialisation).

## Question

Should `Pewei<C, V>` (and its constituent types) implement
`serde::Serialize` / `serde::Deserialize`?

| Option | Approach                                                | Dependency impact                          | Notes                                                                                                                                          |
| ------ | ------------------------------------------------------- | ------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| A      | Yes, unconditionally derive `Serialize` + `Deserialize` | `serde` becomes a required dependency      | Simplest for consumers. Adds ~200 KB to compile graph.                                                                                         |
| B      | Yes, behind a `serde` Cargo feature flag                | `serde` is optional, zero-cost when unused | Standard Rust pattern (`chrono`, `uuid`, `url`, etc.). Requires `#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]` on each type. |
| C      | No serde integration; users serialise manually          | No dependency change                       | Maximum control for users; no coupling. But PEWEI types are simple data — manual serialisation is busywork.                                    |

**Considerations:**

- All Surface 1 snapshot types (ADR-M-032 §Prints) are pure data (no
  references, no handles, no interior mutability) — ideal for
  `Serialize`/`Deserialize`.
- Feature-gated serde is well-established Rust convention.
- The crate's `Coordinate` and `Accumulator` types (`u64`, `f64`,
  etc.) already implement `Serialize` via serde's blanket impls.
- Option B means `Pewei<C, V>` is only `Serialize` when both `C`
  and `V` are `Serialize` — which is always true for primitive types
  but not for custom newtypes unless the user derives it.
- The `GvGraph` itself should **not** be serialisable (it contains
  arenas, handles, internal state). Only extracted snapshots and
  view types should be serialisable.

## Decision

**DC-022-1: Option B — feature-gated `serde`.**

```toml
[features]
serde = ["dep:serde"]

[dependencies]
serde = { version = "1", features = ["derive"], optional = true }
```

All Surface 1 snapshot types (ADR-M-032 §Prints) get:

```rust
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
```

Specifically:

| Module          | Types                                                                                                   |
| --------------- | ------------------------------------------------------------------------------------------------------- |
| `view`          | `Span<C, V>`, `Cell<C, V>`, `Node<C, V>`                                                                |
| `gnode`         | `GState`                                                                                                 |
| `pewei`         | `Pewei<C, V>`, `Layer<C, V>`, `Transition<C, V>`, `Terminal<C, V>`                                       |
| `plateau`       | `BasisEdge<C>`, `Plateau<C, V>`                                                                          |
| `contour_range` | `BasisElement<C, V>`, `ContourRange<C, V>`, `ContourRangeEnergy<V>` |

**Excluded:** `GNodeId` and `VNodeId` are opaque arena handles —
meaningless outside the originating `GvGraph` instance. Serialising
them would encourage invalid cross-graph handle use. They are
Surface 1 types by classification (ADR-M-032 §Handles) but not
serialisation candidates.

**Excluded:** `GvGraph` itself is **not** serialisable — it contains
arenas, handles, and internal state. Only extracted snapshots
(`Pewei`) and view types are serialisation candidates.

Follows the standard Rust pattern (`chrono`, `uuid`, `url`). The
crate stays dependency-light when the feature is disabled. `serde`
is always available as a dev-dependency for the test harness,
regardless of the feature flag.

**Note:** The `serde` feature is enabled by default in `Cargo.toml`
(`default = ["dynamic-contour-tracking", "rand", "serde"]`), so
most consumers get serialisation support without opting in.
Consumers who want a minimal dependency tree can specify
`default-features = false`.

## Consequences

- Zero compile-time cost for consumers who disable the `serde`
  feature (`default-features = false`).
- Consumers using default features get `Serialize`/`Deserialize` on
  all Surface 1 snapshot types with no additional configuration.
- Users who want JSON/CBOR/MessagePack output get
  `Serialize`/`Deserialize` on all Surface 1 snapshot types.
- `Pewei<C, V>` is `Serialize` when both `C: Serialize` and
  `V: Serialize` — always true for primitive types. Custom newtypes
  must derive `Serialize` themselves.
- The `dynamic-contour-tracking`, `rand`, and `serde` features are
  independent and composable.
