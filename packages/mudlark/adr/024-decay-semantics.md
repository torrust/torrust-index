# ADR-M-024: Decay Semantics

**Status:** Decided  
**Date:** 2026-02-26  
**Phase:** 4 (Output)  
**Relates to:** [ADR-M-010](010-observation-generics.md) (Observation
generics), [ADR-M-012](012-g-sum-recomputation.md) (G-sum recomputation),
[ADR-M-023](023-pewei-reconstruction.md) (PEWEI reconstruction)  
**Spec:** §IDEA M-14 (temporal semantics), §IDEA M-14.5 (filter bank),
§IDEA M-14.3 (three-parameter decay)  
**API:** [§API M-5.2](../docs/api.md#temporal-decay) (decay)  
**Surface:** 2 (Film)

---

## Context

### The G-V Graph as a spatiotemporal filter bank

The G-V Graph is a **spatiotemporal filter bank** with three stages:

| Stage          | Operation                                                                                                      | Axis            | Module          |
| -------------- | -------------------------------------------------------------------------------------------------------------- | --------------- | --------------- |
| **Analysis**   | Each `observe` decomposes incoming energy into spatial subbands defined by the G-Tree's plateaus (§IDEA M-5.6) | Spatial → Scale | `observe()`     |
| **Processing** | Decay attenuates inactive entries; rebalance promotes active ones — a time-varying gain per subband            | Time            | `decay()`       |
| **Synthesis**  | PEWEI reconstruction reassembles the filtered subbands into a spatial intensity function                       | Scale → Spatial | `reconstruct()` |

This is the same architecture as a QMF filter bank in audio, a
subband coder in video, or a wavelet-domain Wiener filter in
denoising. The only unusual property is that the analysis bank is
_adaptive_ — the G-Tree materialises subbands only where the signal
has structure (§PEWEI M-5, main §IDEA M-6/§IDEA M-11).

The G-Tree's depth axis is a **logarithmic frequency axis**: each
depth doubles the spatial frequency (halves the cell width). Depth
$k$ corresponds to spatial frequency $f_k \propto 2^k$, identical
to octave bands in audio or dyadic levels in a standard wavelet
decomposition.

Each G-node's `g.own` is a **subband coefficient**. At internal
nodes it is a **scaling coefficient** — the frozen pre-refinement
measurement. At terminals it is the **leaf measurement**. The
V-Tree ranks these coefficients by significance. PEWEI extraction
reads them out in significance order.

### The coefficient model

Under accumulation and decay, each subband coefficient follows a
first-order IIR (exponential moving average):

$$c_k(t+1) = \lambda_k \cdot c_k(t) + \delta_k(t)$$

where $\delta_k(t)$ is the observation increment at time $t$ and
$\lambda_k$ is the decay factor for subband $k$ (depth $k$ in the
G-Tree). The pole $\lambda_k$ determines the effective memory
window:

$$\tau_k = \frac{-1}{\ln \lambda_k} \qquad \text{(time constant in decay steps)}$$

### Decay as a three-parameter temporal filter

A temporal filter on the subband axis has three natural parameters,
analogous to a parametric EQ band:

| Parameter       | Meaning                                 | Axis                          |
| --------------- | --------------------------------------- | ----------------------------- |
| **Root**        | Which G-subtree to decay                | Spatial (which region)        |
| **Attenuation** | How much to decay at the midpoint depth | Temporal gain                 |
| **Q**           | How depth-selective the decay is        | Scale (frequency selectivity) |

The **attenuation** is the base decay factor, applied at the
midpoint of the target subtree's depth range. **Q** controls how
the factor varies across the depth axis: at $q = 0$, all subbands
within the target receive the same factor (uniform / extreme
wideband); at $q > 0$, coarse subbands persist while fine subbands
fade faster (selective / narrowband).

The per-depth factor within the target subtree:

$$\ln\lambda(d) = \ln(\text{att}) \cdot \left(1 + q \cdot \left(\frac{2d_\text{local}}{D} - 1\right)\right)$$

where $d_\text{local}$ is the depth relative to the subtree root
and $D$ is the local depth range ($N - d_\text{root}$). This gives:

| Depth within subtree              | Factor             |
| --------------------------------- | ------------------ |
| Root ($d_\text{local} = 0$)       | $\text{att}^{1-q}$ |
| Midpoint ($d_\text{local} = D/2$) | $\text{att}$       |
| Deepest ($d_\text{local} = D$)    | $\text{att}^{1+q}$ |

The midpoint always decays at exactly `attenuation`, regardless of
Q. Q tilts the gain line around this pivot.

### Selectivity continuum

```
q = 0.0    uniform         λ(d) = att for all d
q = 0.3    gentle tilt     λ_coarse = att^0.7,  λ_fine = att^1.3
q = 0.5    moderate        λ_coarse = att^0.5,  λ_fine = att^1.5
q = 1.0    maximum         λ_coarse = 1 (no decay),  λ_fine = att²
```

There is no qualitative discontinuity. The $q = 0$ case is the
zero-selectivity limit, and the implementation detects it to
take the uniform fast path.

### Composition under repeated application

$k$ steps of $(\text{att}, q)$ = one step of $(\text{att}^k, q)$.

**Q is invariant under repetition.** Only the attenuation deepens.
The selectivity is a structural property of the filter, not a
magnitude. This follows from the log-linear envelope:

$$\left[\text{att}^{1 + q(2d/D - 1)}\right]^k = (\text{att}^k)^{1 + q(2d/D - 1)}$$

This composability fails for non-log-linear profiles (bell curves,
step functions, arbitrary user closures). The log-linear envelope
is the unique monotonic profile that is a fixed-point under the
composition operator "apply the same decay $k$ times." This is the
signal-processing reason it is the correct default.

### Subtree targeting

When root is an interior node at global depth $d_r$, the local
depth is $d_\text{local} = d_\text{global} - d_r$ and the local
range is $D = N - d_r$. The selectivity envelope operates on local
depth — the subtree root's `g.own` gets $\text{att}^{1-q}$ (coarsest
within this region) and the deepest terminals get $\text{att}^{1+q}$
(finest within this region).

Use cases for subtree-targeted decay:

- **Region-specific staleness.** Traffic stopped in region $[a, b)$
  but the rest of the domain is active. Decay only the stale region.
- **Heterogeneous temporal policies.** Different sensors report at
  different rates. Each region decays at its own rate.
- **Targeted cleanup.** A noise burst contaminated a specific region.
  Decay it aggressively without disturbing confirmed structure
  elsewhere.
- **Cost.** Decaying a subtree of size $S$ costs $O(S)$, not $O(G)$.

### SNR evolution

Per-subband SNR (§PEWEI M-6), where $N_k$ is the frozen
benchmark at depth $k$:

At $q = 0$: $\text{SNR}'_d = \frac{\text{att} \cdot I_d}{\text{att} \cdot N_k} = \text{SNR}_d$. Invariant. Stale measurements never lose confidence.

At $q > 0$: $\text{SNR}'_d = \frac{\text{att}^{1+q'} \cdot I_d}{\text{att}^{1-q'} \cdot N_k} = \text{att}^{2q'} \cdot \text{SNR}_d < \text{SNR}_d$ (where $q'$ depends on the relative depths). The SNR _decreases_ over time for stale measurements. Terminals that aren't refreshed gradually drop below their noise floor — the tree suppresses stale fine-scale coefficients. This is a **temporally adaptive Wiener filter** driven by the Q parameter.

### Analogues in other domains

| Field  | Technique                                       | Same principle                                            |
| ------ | ----------------------------------------------- | --------------------------------------------------------- |
| Audio  | Multiband compression (per-band attack/release) | Low bands: slow release. High bands: fast release.        |
| Video  | Motion-adaptive temporal NR                     | Low spatial freq: many-frame average. High: fewer frames. |
| Radar  | Subband integration / MTI                       | Low-Doppler: long dwell. High-Doppler: short integration. |
| Speech | Per-band spectral subtraction                   | Wiener filter with frequency-dependent gain.              |
| Image  | Wavelet-domain denoising (BayesShrink)          | Threshold estimated per subband.                          |

---

## Questions

### Q1: API shape (DC-024-1)

```rust
/// Apply temporal decay to a G-subtree.
///
/// - `root`: G-node whose subtree is decayed. Use `g_root()` for the
///   entire tree.
/// - `attenuation`: base decay factor, applied at the midpoint depth
///   of the subtree. Values in (0, 1) cause exponential decay.
/// - `q`: selectivity in [0, 1].
///   - `0.0` = uniform: all subbands in the subtree decay at the same
///     rate.
///   - `> 0.0` = selective: coarse subbands persist, fine subbands
///     fade faster.
///   - `1.0` = maximum: the subtree root's depth doesn't decay at all;
///     the deepest terminals decay at `attenuation²`.
pub fn decay(
    &mut self,
    root: GNodeId,
    attenuation: f64,
    q: f64,
);
```

`attenuation` is `f64` rather than `O: Observation<V>`. Decay
factors are inherently floating-point ratios. At $q > 0$ the
per-depth factors are derived via `ln`/`exp` in f64 — the
`Observation` generic would be misleading since the actual scaling
always operates in f64 precision regardless of what `O` is.
Internally, each node is scaled via
`V::from_f64(value.to_f64() * factor)` — the `Accumulator` trait's
`to_f64`/`from_f64` conversion pair, the same arithmetic that
the blanket `Observation<V> for V` impl's `scale` path performs.

Three parameters, one method. Q = 0 is defined as uniform — the
implementation detects this and takes the uniform fast path
(same factor for every node, but still bottom-up recompute and
trailing rebalance — see Q2).

**Considered alternatives:**

| Alternative                                         | Why not                                                                                                                                      |
| --------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------- |
| `Decay<O>` enum with `Uniform`/`Selective` variants | Binary choice for a continuous parameter. Q = 0.01 is closer to uniform than to "selective mode." The enum forces a false dichotomy.         |
| Separate `decay()` and `decay_subtree()` methods    | Unnecessary split. `decay(g_root, att, q)` is the global case.                                                                               |
| Per-node callback `f(node) → factor`                | Unstructured. No composability guarantee. Unbounded V-I3 violations.                                                                         |
| `(coarse, fine)` instead of `(att, q)`              | Less intuitive. Doesn't make the midpoint-preservation property visible. Harder to reason about what repeated application does to the shape. |

**Validation:**

| Condition              | Behaviour                                                                                                                                 |
| ---------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| `q == 0.0`             | Uniform fast path. Same factor for every node.                                                                                            |
| `q` in `(0.0, 1.0]`    | Selective path. Per-depth factors via precomputed table.                                                                                  |
| `q < 0.0`              | Panic. Negative selectivity is not meaningful.                                                                                            |
| `q > 1.0`              | Panic. At $q > 1$, coarse subbands would _grow_, violating decay semantics.                                                               |
| `attenuation == 0.0`   | Annihilation (§THEORY M-7.3). At $q < 1$: entire subtree zeroed. At $q = 1$: detail flush ($0^0 = 1$ preserves root, descendants zeroed). |
| `attenuation < 0.0`    | Panic. Negative factors are not meaningful for multiplicative decay.                                                                      |
| `attenuation.is_nan()` | Panic. NaN attenuation is a programmer error.                                                                                             |
| `attenuation == 1.0`   | No-op. Early return.                                                                                                                      |
| `attenuation > 1.0`    | Allowed (amplification). Documented.                                                                                                      |

### Q2: G-sum propagation strategy (DC-024-2)

After scaling `g.own`, G-sums must be updated. In principle, for
$q = 0$ a single-pass strategy could scale both `g.own` and `g.sum`
by the same factor, since $\lambda(a+b) = \lambda a + \lambda b$.
However, for integer accumulator types, `floor(sum × λ)` ≠
`floor(own × λ) + floor(child.sum × λ)` due to independent
truncation at each node. The implementation therefore uses
**bottom-up recompute for both paths**:

| Q       | Strategy                                                                                                                                      | Cost            |
| ------- | --------------------------------------------------------------------------------------------------------------------------------------------- | --------------- |
| $q = 0$ | Scale `g.own` by the uniform factor, then recompute `g.sum = own + left.sum + right.sum` bottom-up. Same as $q > 0$ but with a single factor. | $O(S)$ two-pass |
| $q > 0$ | Scale `g.own` by per-depth factors from the precomputed table, then recompute `g.sum` bottom-up identically.                                  | $O(S)$ two-pass |

Then in both cases:

- Propagate `g.sum` from subtree root upward to G-root: $O(d_r)$.
- Sync V-entry intensities from `g.own` for all entries in the
  subtree: $O(E_S)$.
- Recompute V-structural intensities (global): $O(V)$.
- Detect V-I3 violations + rebalance: $O(K \log K)$.

**Why always rebalance.** Even at $q = 0$, integer truncation from
`V::from_f64` introduces per-node rounding errors that accumulate
across repeated decay calls. These can cause cumulative V-I3 drift.
The implementation always runs `find_violated_nodes` + `rebalance`
after decay — the scan is $O(V)$ and returns empty when no
violations exist, so the overhead is negligible when invariants
hold.

### Q3: Decay triggers eviction? (DC-024-3)

| Option | Behaviour                                            |
| ------ | ---------------------------------------------------- |
| **A**  | Yes — `decay()` calls `check_evictions()` internally |
| **B**  | No — `decay()` only applies the temporal filter      |

Composability: the `decay → extract → evict` pipeline is a key
workflow:

```rust
graph.decay(root, 0.95, 0.3);
let snapshot = graph.extract();   // PEWEI of decayed-but-alive state
graph.check_evictions();          // now contract
```

Auto-eviction would preclude inspecting the post-decay state. For
selective decay specifically, the user may want to extract the PEWEI
to see how per-subband SNR evolved _before_ eviction removes
low-confidence coefficients.

`check_evictions()` already exists as a public method. Easier to add
`decay_and_evict()` later (additive) than to un-couple (breaking).

### Q4: Interpolation function (DC-024-4)

The log-linear envelope $\ln\lambda(d) = \ln(\text{att}) \cdot (1 + q \cdot (2d/D - 1))$ is the unique monotonic profile that:

1. Preserves the midpoint under all Q values.
2. Is closed under repeated application (Q invariant).
3. Gives constant dB-per-octave rolloff.

**Implementation:** precompute a table of $D+1$ factors. Cost: $D+1$
calls to `exp`, amortised over the subtree walk. For $N \leq 64$,
the table fits on the stack.

**Considered alternatives:**

| Alternative                    | Why not                                                                          |
| ------------------------------ | -------------------------------------------------------------------------------- |
| Linear interpolation           | Additive interpolation of a multiplicative parameter. Q drifts under repetition. |
| User-supplied `fn(u32) -> f64` | Loses composability. Hard to serialise.                                          |
| Bell/bandpass shape            | Sharpens toward a notch under repetition. Q is not invariant.                    |

### Q5: Depth axis (DC-024-5)

G-Tree depth (structural depth of the G-node), not V-Tree depth
(significance rank).

G-Tree depth is the spatial frequency axis. It is stable (does not
change under V-Tree rebalancing) and corresponds to the subband
decomposition that PEWEI reconstruction pro-rates along.

`GNode` does not store depth. Derivable from `(lo, hi)`:
$d = N - \lfloor\log_2(\text{hi} - \text{lo})\rfloor$.
Computed per-node during the decay walk via
`gnode_depth_from_interval` (see §ARCH M-4.4).

---

## Decision

| ID       | Decision                                                                                                                                   | Option |
| -------- | ------------------------------------------------------------------------------------------------------------------------------------------ | ------ |
| DC-024-1 | `fn decay(&mut self, root: GNodeId, attenuation: f64, q: f64)` — three parameters, one method, `f64` attenuation (not `O: Observation<V>`) | See Q1 |
| DC-024-2 | Bottom-up G-sum recompute for both $q = 0$ and $q > 0$ (integer truncation prevents single-pass scale)                                     | See Q2 |
| DC-024-3 | No auto-eviction — `decay()` only applies the temporal filter                                                                              | B      |
| DC-024-4 | Log-linear interpolation (Q-invariant under repetition)                                                                                    | A      |
| DC-024-5 | G-Tree depth (spatial frequency axis)                                                                                                      | A      |

### DC-024-1 rationale: `f64` over `O: Observation<V>`

The `Observation<V>` generic (ADR-M-010) decouples arithmetic
precision from storage for additive `observe()` — the caller
chooses whether to accumulate at f32, f64, or same-type precision.
This makes sense for deltas: a u16-stored tree can observe an f32
increment.

Decay factors are fundamentally different. They are multiplicative
ratios in (0, 1). At $q = 0$ every node is scaled by the same
factor; at $q > 0$ the per-depth factors are computed via
`ln`/`exp` in f64 regardless of what `O` would be. The generic
would be ceremony without benefit — nobody decays by a `u16`
factor. Using `f64` directly is honest about what the arithmetic
actually does.

---

## Consequences

- `decay()` is a single method with three parameters: root
  (spatial target), attenuation (temporal gain), Q (frequency
  selectivity). No enum, no mode switch — a continuous
  parameterisation of the temporal filter.

- Both $q = 0$ and $q > 0$ use the same two-pass structure:
  DFS walk to scale `g.own`, then reverse-order (bottom-up)
  recompute of `g.sum`. The uniform path applies a single factor;
  the selective path indexes a precomputed per-depth table.
  Cost: $O(S + V + K \log K)$ in both cases, where $K$ is the
  number of V-I3 violations (typically zero for uniform decay
  on float types, small for integer types under truncation drift).

- Q is invariant under repeated application. $k$ decay steps of
  $(\text{att}, q)$ = one step of $(\text{att}^k, q)$. The
  selectivity is a structural property; only attenuation compounds.

- Subtree targeting is native: `decay(some_node, att, q)` decays
  only that spatial region at cost $O(S)$.
  `decay(g_root(), att, q)` is the global case.

- Eviction is not triggered by `decay()`. Users compose
  `decay()` → `extract()` → `check_evictions()` as needed.

- SNR evolves under $q > 0$: stale fine-scale terminals lose
  confidence relative to persistent coarse benchmarks. The
  denoising pass (§PEWEI M-11) becomes temporally adaptive —
  a Wiener filter driven by Q.

- No changes to `reconstruct()` are needed. G-I1 is maintained
  after decay. `Accumulator::sub` (ADR-M-023) handles remainder
  computation on decayed values transparently.

- Factor application uses `Accumulator::to_f64` and
  `Accumulator::from_f64` directly — the same arithmetic as the
  blanket `Observation<V> for V` impl's `scale` path, but without
  routing through the trait. No new trait methods are required.

- V-I3 violations after decay are handled by the existing Phase 3
  rebalance infrastructure. At $q = 0$ on float types: typically
  zero violations. At $q = 0$ on integer types: occasional
  violations from cumulative truncation drift. At $q > 0$:
  violations concentrated at cross-depth boundaries, count
  monotonic in Q.

- Plateau mirror maintenance: after G-sum and V-intensity updates,
  plateau sums are recomputed, `normalize_plateaus` removes any
  degenerate plateaus, and P-I4 repair runs. This is a no-op when
  the `dynamic-contour-tracking` feature is disabled. (Note:
  ADR-M-031's sorted-placement elimination applies to `observe()`,
  not to `decay()` — decay still runs the full normalize pass.)

### Cost summary

| Q       | Steps                                                                                                                                                                                                         | Total cost            |
| ------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------- |
| $q = 0$ | Scale `g.own` uniformly; recompute `g.sum` bottom-up; propagate to G-root; sync V-entries; recompute V-structural sums; detect + rebalance violations; plateau maintenance                                    | $O(S + V + K \log K)$ |
| $q > 0$ | Precompute per-depth factor table; scale `g.own` per-depth; recompute `g.sum` bottom-up; propagate to G-root; sync V-entries; recompute V-structural sums; detect + rebalance violations; plateau maintenance | $O(S + V + K \log K)$ |
