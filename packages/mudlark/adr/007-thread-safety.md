# ADR-M-007: Thread Safety

**Status:** Decided  
**Date:** 2026-02-24  
**Relates to:** [ADR-M-001](001-node-storage.md) (arena storage),
[ADR-M-006](006-generic-parameters.md) (generic parameters — trait bounds),
[ADR-M-029](029-opportunistic-depth-caching.md) (introduces `AtomicU32`
in `VNode` — the sole interior-mutable field),
[ADR-M-032](032-three-surface-model.md) (three-surface model)  
**Spec:** §IDEA M-17.3 (Write-Set Locality)  
**API:** [§API M-7.2](../docs/api.md#72-thread-safety) (thread safety)  
**Surface:** 2 (Film)

## Context

`GvGraph` is shared between async tasks, sent to worker threads,
and held behind `Arc<RwLock<_>>`. This requires
`GvGraph<C, V, N>: Send + Sync`.

## Decision

**`GvGraph` is `Send + Sync` by construction. The traits enforce it.**

### 1. Trait bounds include `Send + Sync`

```rust
pub trait Coordinate: Copy + PartialOrd + Debug + Default + Send + Sync + 'static { ... }
pub trait Accumulator: Copy + PartialOrd + Debug + Default + Send + Sync + 'static { ... }
pub trait Observation<V: Accumulator>: Copy + Debug + Send + Sync { ... }
```

`Default` is required by `Arena<T: Default>` (ADR-M-001) and `'static`
ensures arena-stored types carry no borrows.

All provided implementations (`u8`..`u128`, `f32`, `f64`) satisfy
these bounds. User-defined newtypes (e.g., `Saturating<u64>`) must
also be `Send + Sync` — which any `#[repr(transparent)]` wrapper
over a primitive already is.

### 2. Interior mutability limited to `AtomicU32`

The only interior-mutable field in the entire struct graph is
`VNode::cached_depth: AtomicU32`, introduced by
[ADR-M-029](029-opportunistic-depth-caching.md) for opportunistic
V-Tree depth caching. `AtomicU32` is unconditionally `Send + Sync`
(it is designed for exactly this use case). All other fields are
plain data — no `Rc`, `Cell`, `RefCell`, raw pointers, or
user-facing `UnsafeCell`.

Because every field is either a plain-data type or `AtomicU32`,
Rust auto-derives `Send + Sync` for the entire struct.

**Top-level fields:**

| Field               | Type                                    | Auto-trait                    |
| ------------------- | --------------------------------------- | ----------------------------- |
| `gnodes`            | `Arena<GNode<C, V>>`                    | `Send + Sync` when `C, V` are |
| `vnodes`            | `Arena<VNode<V>>`                       | `Send + Sync` when `V` is ¹   |
| `g_root`            | `GNodeId` (`NonZeroU32`)                | `Copy + Send + Sync`          |
| `v_root`            | `Option<VNodeId>` (`NonZeroU32` niche)  | `Copy + Send + Sync`          |
| `config`            | `Config<V>`                             | `Send + Sync` when `V` is     |
| `violations`        | `Vec<VNodeId>`                          | `Send + Sync`                 |
| `node_count`        | `u32`                                   | `Send + Sync`                 |
| `terminal_count`    | `u32`                                   | `Send + Sync`                 |
| `live_depth_evict`  | `u32`                                   | `Send + Sync`                 |
| `live_depth_create` | `u32`                                   | `Send + Sync`                 |
| `depth_buffer`      | `u32`                                   | `Send + Sync`                 |
| `headroom`          | `usize`                                 | `Send + Sync`                 |
| `soft_limit`        | `Option<usize>`                         | `Send + Sync`                 |
| `plateaus`†         | `BTreeMap<BasisEdge<C>, Plateau<C, V>>` | `Send + Sync` when `C, V` are |
| `pending_p_i4`†     | `Vec<(GNodeId, BasisEdge<C>)>`          | `Send + Sync` when `C` is     |
| `plateau_basis`†    | `PlateauBasis<C>` ²                     | `Send + Sync` when `C` is     |
| `plateaus_dirty`†   | `bool`                                  | `Send + Sync`                 |

† Behind `#[cfg(feature = "dynamic-contour-tracking")]`.

¹ `VNode<V>` contains `cached_depth: AtomicU32` (ADR-M-029).
`AtomicU32` is unconditionally `Send + Sync`, so the arena's
auto-trait depends only on `V`.

² `PlateauBasis<C>` is backed by a `BTreeMap<BasisEdge<C>,
HashSet<GNodeId>>` (forward map) and a `HashMap<GNodeId,
BasisEdge<C>>` (back map). All three collections are `Send + Sync`
when their element types are — satisfied when `C: Send + Sync`.

**Arena internals** (inside each `Arena<T: Default>`):

| Component | Type       | Auto-trait                |
| --------- | ---------- | ------------------------- |
| Slots     | `Vec<T>`   | `Send + Sync` when `T` is |
| Bitset    | `Vec<u64>` | `Send + Sync`             |
| Free list | `Vec<u32>` | `Send + Sync`             |
| Count     | `u32`      | `Send + Sync`             |

### 3. Compile-time assertions

```rust
// In src/tests/graph.rs (crate-internal test module).
const _: () = {
    fn assert_send_sync<T: Send + Sync>() {}

    fn check() {
        assert_send_sync::<GvGraph<u64, u64, 32>>();
        assert_send_sync::<GvGraph<u128, f64, 128>>();
        assert_send_sync::<GvGraph<f64, f64, 52>>();
        assert_send_sync::<Config<u64>>();
        assert_send_sync::<Config<f64>>();
    }
};
```

These assertions fail at compile time if any internal field breaks
the auto-trait. They guard against future regressions.

### 4. Concurrency contract

| Access pattern       | Mechanism              | Notes                                               |
| -------------------- | ---------------------- | --------------------------------------------------- |
| Share for reads      | `Arc<GvGraph>`         | `&GvGraph` is `Sync` — all read methods are `&self` |
| Exclusive writes     | `Arc<Mutex<GvGraph>>`  | Standard Rust — one writer at a time                |
| Read-many write-rare | `Arc<RwLock<GvGraph>>` | Readers don't block each other                      |
| Send to thread       | `move` or channel      | `GvGraph` is `Send` — ownership transfers freely    |

The crate does **not** provide internal locking. Concurrent mutation
is the caller's responsibility via standard synchronization
primitives. This follows `std::collections` convention — `Vec`,
`HashMap`, and `BTreeMap` are all `Send + Sync` but require external
locking for concurrent writes.

### 5. Relationship to §IDEA M-17.3 (Write-Set Locality)

§IDEA M-17.3 analyzes _algorithmic_ concurrency: each observation's
write set decomposes into commutative sum propagation and local
structural writes confined to a grandparent neighborhood (~10 nodes).
Two observations hitting different G-Tree leaves whose V-entries do
not share a grandparent have completely non-overlapping structural
write sets.

This ADR addresses the _Rust-level_ prerequisite: `GvGraph` must be
`Send + Sync` before any concurrency strategy — external `RwLock`,
fine-grained locking, or atomic writes — can be applied. The two
concerns are complementary: §IDEA M-17.3 shows the algorithm is amenable to
concurrency; this ADR shows the implementation carries the types
through.

## Rationale

- `Send + Sync` is a hard requirement for async runtimes (tokio,
  async-std) and thread pools (rayon). Without it the type is
  unusable in most real-world Rust applications.
- The design achieves `Send + Sync` without any `unsafe` code.
  The sole interior-mutable field (`AtomicU32` in `VNode`, ADR-M-029)
  uses a safe atomic — no raw `UnsafeCell` or `unsafe` blocks.
- Internal locking would add overhead to single-threaded use and
  complicate the API. External `RwLock` is strictly more flexible.

## Consequences

- `Coordinate` and `Accumulator` require `Send + Sync` — excludes
  types containing `Rc`, `Cell`, or non-`Send` raw pointers.
  This is not a practical restriction since coordinate and intensity
  types are numeric primitives.
- All Surface 1 types are also `Send + Sync`. The `Copy` types
  (`Span`, `Cell`, `Node`, `Plateau`, `BasisEdge`, `Transition`,
  `Terminal`) are structs of primitives with no interior mutability.
  The `Clone` types (`Pewei`, `Layer`) own `Vec`s of `Send + Sync`
  elements.
- Compile-time assertions catch regressions immediately.
