// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

use super::SpatialRead;

/// Subband-adaptive temporal scaling of accumulated intensities.
///
/// # Summary
///
/// Requires `V: Attenuatable`.  Types that cannot be meaningfully
/// scaled (e.g. ordinal rankings) simply omit the
/// [`Attenuatable`](super::Attenuatable) impl — calling `decay()`
/// on them is a compile error.  Split from
/// [`SpatialWrite`](super::SpatialWrite) (ADR-M-009 Addendum 3)
/// because temporal scaling needs a stricter bound than plain
/// observation.
///
/// # Analogy
///
/// In the silver halide model ([ADR-M-032]), heat drives latent
/// image regression: sub-critical silver atom clusters thermally
/// disperse and the grain re-sensitizes.  `TemporalDecay` is the
/// thermal regressor instrument — the controlled heat source that
/// the user applies to age (or intensify) the negative.
///
/// # Contract
///
/// ## Three regimes
///
/// The spec (§IDEA M-1.3, §IDEA M-14) calls this operation `modulate` and
/// defines three regimes depending on the scaling factor:
///
/// | Factor | Regime | Silver halide analog | Effect |
/// |--------|--------|---------------------|--------|
/// | `0.0` | Annihilation | Complete bleaching | Hard reset — all competitive standing lost |
/// | `(0, 1)` | Attenuation | Thermal regression | Clusters disperse, recency wins |
/// | `1.0` | Identity | No heat | No change |
/// | `> 1.0` | Amplification | Chemical intensification | Fine-scale structure reinforced |
///
/// The parameter is named `attenuation` (matching the most common
/// regime), but the architecture supports all three — and the
/// underlying `Attenuatable::attenuate` method accepts any
/// non-negative factor.
///
/// ## Subband adaptivity
///
/// The `q` selectivity parameter controls how the scaling rate
/// varies across spatial depths, so coarser and finer resolution
/// regions can age at different rates (§IDEA M-14.3):
///
/// - `q = 0.0` — uniform: all subbands scale at the same rate.
/// - `q > 0.0` — selective: coarse structure persists, fine
///   structure fades (or is amplified) faster.
/// - `q = 1.0` — maximum selectivity.
///
/// ## Object safety
///
/// `TemporalDecay` could be made object-safe (no generics on
/// `decay()`), but is kept alongside the other instruments as a
/// static-dispatch capability bound.
///
/// # Implementations
///
/// [`GvGraph<C, V, N>`](crate::GvGraph) is the primary implementor.
///
/// # Examples
///
/// Accept any scalable structure via a trait bound:
///
/// ```
/// use torrust_mudlark::{SpatialRead, TemporalDecay, GNodeId};
///
/// fn half_life<S: TemporalDecay>(s: &mut S, root: GNodeId) {
///     s.decay(root, 0.5, 0.0);
/// }
///
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
/// g.observe(42, 100u64);
/// let before = g.total_sum();
/// let root = g.g_root();
/// half_life(&mut g, root);
/// assert!(g.total_sum() < before);
/// ```
///
/// # References
///
/// - [ADR-M-032] — Three-surface visibility model
///
/// [ADR-M-032]: https://github.com/torrust/torrust-index/blob/develop/packages/mudlark/adr/032-three-surface-model.md
pub trait TemporalDecay: SpatialRead {
    /// Apply temporal scaling to a G-subtree.
    ///
    /// - `root` — the G-node whose subtree receives scaling.  Pass
    ///   [`GvGraph::g_root()`](crate::GvGraph::g_root) to scale the
    ///   entire domain.
    /// - `attenuation` — base multiplicative factor at the midpoint
    ///   depth.  Values in `(0, 1)` attenuate; `> 1.0` amplifies;
    ///   `0.0` annihilates.
    /// - `q` — depth selectivity in `[0.0, 1.0]`: `0.0` applies
    ///   uniform scaling at every depth; higher values make shallow
    ///   and deep nodes scale at different rates.
    ///
    /// # Examples
    ///
    /// ```
    /// # use torrust_mudlark::{Config, GvGraph};
    /// use torrust_mudlark::TemporalDecay;
    /// # let cfg = Config {
    /// #     split_threshold: 5u64,
    /// #     depth_create: 3,
    /// #     depth_evict: 6,
    /// #     budget: None,
    /// #     alpha_relax: 0.75,
    /// #     bounded_eviction: true,
    /// # };
    /// # let mut g = GvGraph::<u64, u64, 8>::new(cfg);
    /// g.observe(42, 100u64);
    /// let root = g.g_root();
    ///
    /// // Attenuation: 50% regression.
    /// TemporalDecay::decay(&mut g, root, 0.5, 0.0);
    /// assert!(g.total_sum() < 100);
    /// ```
    fn decay(&mut self, root: crate::handle::GNodeId, attenuation: f64, q: f64);
}
