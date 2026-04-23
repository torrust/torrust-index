// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

use std::fmt::Debug;

/// Intensity / accumulation type for the G-V Tree.
///
/// # Summary
///
/// Core bound on the `V` parameter in
/// [`GvGraph<C, V, N>`](crate::GvGraph).  Provides the arithmetic
/// surface needed by the internal tree machinery:
/// [`zero`](Self::zero) (additive identity), [`add`](Self::add),
/// and [`sub`](Self::sub).  Additional capabilities are gated
/// behind independent sub-traits — each unlocks specific
/// [`GvGraph`](crate::GvGraph) operations (§IDEA M-8.8):
///
/// | Sub-trait | Unlocks |
/// |-----------|---------|
/// | [`Attenuatable`](super::Attenuatable) | `decay()` — temporal scaling |
/// | [`Weighable`](super::Weighable) | `sample()` — proportional sampling |
/// | [`Proratable`](super::Proratable) | `range_sum()` — fractional range queries |
/// | [`Inspectable`](super::Inspectable) | `observe()`, `extract()`, `layers()` — core ops + diagnostics |
///
/// All built-in types (`u8`–`u128`, `f32`, `f64`) implement every
/// sub-trait.  Custom accumulators implement only what they need.
///
/// # Analogy
///
/// In the silver halide model ([ADR-M-032]), `Accumulator` *is* the
/// choice of crystal — `AgBr`, `AgCl`, `AgI`.  Different crystals
/// have different sensitivity profiles and combination rules;
/// likewise, different `Accumulator` implementations define
/// different zero elements and addition/subtraction semantics.
///
/// # Contract
///
/// ## Algebraic contract — the Standard configuration
///
/// Implementations must satisfy the Standard configuration from the
/// spec (§IDEA M-1.6.6) — properties P0 through P5 all hold.  The
/// spec organises these as a base axiom set (P0, always required)
/// plus five optional properties (P1–P5) that each unlock specific
/// features in the V-Tree's competitive governance.  The Standard
/// configuration requires **all** of them.
///
/// ### P0 — Base axioms (always required)
///
/// P0 is not a single rule but a bundle of five base axioms:
///
/// | Axiom | Statement | Why it matters |
/// |-------|-----------|----------------|
/// | **Closure** | `add(a, b)` returns `Self` | Type system enforces this. |
/// | **Commutativity** | `add(a, b) == add(b, a)` | Rebalancing merges siblings in arbitrary order — without this, the same merge gives different results depending on rotation direction. |
/// | **Compatibility** | `a <= b` implies `add(a, c) <= add(b, c)` | The V-Tree uses ordering for governance and addition for aggregation.  Compatibility ensures coherence: if `a` is less important than `b`, then `a + c` is still less important than `b + c`. |
/// | **Totality** | `PartialOrd` gives a total order on all values | Required for threshold comparison and violation detection. |
/// | **Well-typing** | `zero()` returns `Self` | The ground element is in the carrier set. |
///
/// P0 alone is enough for governance: compare, aggregate, rebalance,
/// promote, and evict.
///
/// ### P1 — Bounded below
///
/// > There exists a smallest element ⊥ such that `⊥ <= x` for all `x`.
///
/// Enables **proportional sampling** (non-negative ratios give
/// well-defined probability weights) and, together with P5, the
/// **Fibonacci depth bound** on V-Tree height (§IDEA M-18.1).
///
/// ### P2 — Grounded (implies P1)
///
/// > `zero() <= v` for all `v` — the ground element is the minimum.
///
/// Enables **violation-free insertion**: a freshly created node at
/// `zero()` can never outrank an existing one, so catalytic splits
/// never create V-I3 violations (§IDEA M-10.4).
///
/// ### P3 — Idempotent ground
///
/// > `add(zero(), zero()) == zero()`
///
/// Enables **violation-free structural nodes after split**: a new
/// structural V-node initialised to `add(zero(), zero())` carries
/// ground importance, so it cannot violate any uncle constraint.
///
/// ### P4 — Identity element (implies P3)
///
/// > `add(a, zero()) == a` for all `a`.
///
/// The ground element is the additive identity.  Enables:
///
/// - **Ghost eviction fast path**: evicting a node at `zero()`
///   means the parent absorbs nothing, so importance-update + ancestor
///   violation walk can be skipped entirely — O(1) instead of O(hᵥ)
///   per ghost (§IDEA M-12.9.3(B)).
/// - **Sum-propagation early termination**: propagating a zero
///   delta is a no-op, so the walk stops immediately.
///
/// ### P5 — Associativity
///
/// > `add(a, add(b, c)) == add(add(a, b), c)` for all `a, b, c`.
///
/// Structural rearrangements (contraction, promotion) regroup
/// children under a parent.  With associativity, the regrouped sum
/// is algebraically identical — no sum-propagation walk needed after
/// rebalance.
///
/// Together with P1, enables the **Fibonacci depth bound**: V-Tree
/// height is at most $\log_\varphi n$ (§IDEA M-18.1).
///
/// ### Property summary
///
/// | Property | What it says | What it enables |
/// |----------|-------------|-----------------|
/// | **P0** | Closure, commutativity, compatibility, totality, well-typing | Governance: compare, aggregate, rebalance, promote, evict |
/// | **P1** | Bounded below | Proportional sampling; with P5: Fibonacci depth bound |
/// | **P2** | Grounded: `zero() ≤ v` for all `v` (implies P1) | Violation-free insertion |
/// | **P3** | Idempotent ground: `add(zero(), zero()) == zero()` | Violation-free structural nodes after split |
/// | **P4** | Identity: `add(a, zero()) == a` (implies P3) | Ghost eviction fast path; sum-propagation early termination |
/// | **P5** | Associativity | Propagation-free structural rearrangement; with P1: Fibonacci depth bound |
///
/// The Standard configuration requires all of P0–P5.  All built-in
/// unsigned and float types satisfy this: `zero()` is `0` / `0.0`,
/// which is simultaneously the additive identity, the minimum, and
/// the idempotent ground, and ordinary addition is commutative and
/// associative.
///
/// ## Overflow behaviour
///
/// The built-in unsigned impls use `checked_add` / `checked_sub` and
/// **always panic on overflow** — both debug and release builds.
/// This is stricter than native `+` / `-` (which wraps in release)
/// and a deliberate choice: silent wrapping would corrupt the
/// G-Tree's sum invariant (G-I1).  See ADR-M-011 for overflow policy;
/// custom newtype wrappers can opt into saturating or wrapping
/// semantics.
///
/// > **Note (spec divergence — `sub` in core):** The spec
/// > (§IDEA M-8.8) considers subtraction an independent opt-in
/// > capability — the core observation/split/rebalance cycle needs
/// > only `zero` + `add`.  This implementation bundles `sub` into
/// > `Accumulator` for ergonomics: PEWEI refinement,
/// > `Node::refinement()`, and range queries use subtraction
/// > pervasively, and every built-in type supports it.  Custom
/// > types that logically lack subtraction can provide a panicking
/// > stub.
///
/// # Implementations
///
/// Provided for `u8`, `u16`, `u32`, `u64`, `u128`, `f32`, `f64`.
/// All satisfy the Standard configuration (P0–P5); see the
/// [property summary](#property-summary) above.
///
/// Signed integer types (`i8`–`i128`) violate **P2** (zero is not
/// their smallest value) and are therefore excluded — see ADR-M-033
/// for the full analysis, and §IDEA M-1.6 for the general property
/// hierarchy.
///
/// # Examples
///
/// ```
/// use torrust_mudlark::Accumulator;
///
/// fn sum_all<V: Accumulator>(values: &[V]) -> V {
///     values.iter().copied().fold(V::zero(), V::add)
/// }
///
/// assert_eq!(sum_all(&[1u64, 2, 3, 4]), 10);
/// assert_eq!(sum_all::<u64>(&[]), 0);
///
/// // Subtraction for remainder computation.
/// let total = 100u64;
/// let part = 30u64;
/// assert_eq!(total.sub(part), 70);
/// ```
///
/// In context — `Accumulator` is the core bound on every `GvGraph`
/// value type:
///
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
/// // u64 implements all Accumulator sub-traits.
/// let mut g = GvGraph::<u64, u64, 8>::new(cfg);
/// g.observe(42, 10u64);
/// assert_eq!(g.total_sum(), 10);
/// ```
///
/// # References
///
/// - [ADR-M-032] — Three-surface visibility model
///
/// [ADR-M-032]: https://github.com/torrust/torrust-index/blob/develop/packages/mudlark/adr/032-three-surface-model.md
pub trait Accumulator: Copy + PartialOrd + Debug + Default + Send + Sync + 'static {
    /// The additive identity (ground element).
    ///
    /// Under the Standard configuration, this is simultaneously the
    /// minimum value (P2), the identity for `add` (P4), and the
    /// idempotent `add(zero(), zero()) == zero()` (P3).
    fn zero() -> Self;

    /// Same-type addition for internal sum propagation (G-I1).
    ///
    /// Must be commutative (P0) and associative (P5).
    #[must_use]
    fn add(self, other: Self) -> Self;

    /// Same-type subtraction for refinement and remainder computation.
    ///
    /// Built-in unsigned impls use `checked_sub` and always panic on
    /// underflow.  See the note on [overflow behaviour](Accumulator#overflow-behaviour)
    /// above.
    #[must_use]
    fn sub(self, other: Self) -> Self;
}

// ── Unsigned integer implementations ────────────────────────────────

macro_rules! impl_accumulator_uint {
    ($($ty:ty),+) => {$(
        impl Accumulator for $ty {
            #[inline]
            fn zero() -> Self { 0 }

            #[inline]
            fn add(self, other: Self) -> Self {
                self.checked_add(other).expect("attempt to add with overflow")
            }

            #[inline]
            fn sub(self, other: Self) -> Self {
                self.checked_sub(other).expect("attempt to subtract with overflow")
            }
        }
    )+};
}

impl_accumulator_uint!(u8, u16, u32, u64, u128);

// ── Floating-point implementations ──────────────────────────────────

impl Accumulator for f32 {
    #[inline]
    fn zero() -> Self {
        0.0
    }

    #[inline]
    fn add(self, other: Self) -> Self {
        self + other
    }

    #[inline]
    fn sub(self, other: Self) -> Self {
        self - other
    }
}

impl Accumulator for f64 {
    #[inline]
    fn zero() -> Self {
        0.0
    }

    #[inline]
    fn add(self, other: Self) -> Self {
        self + other
    }

    #[inline]
    fn sub(self, other: Self) -> Self {
        self - other
    }
}
