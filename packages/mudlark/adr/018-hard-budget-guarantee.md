# ADR-M-018: Hard Budget Guarantee via Dynamic Soft Limit

**Status:** Decided  
**Date:** 2026-02-25  
**Updated:** 2026-03-03  
**Spec:** §§IDEA M-7.4–7.5 (dynamic depth control, hard budget
guarantee), §IDEA M-12.1.1 (Plateau–Node-Count Bound: $P \leq |G| \leq H$),
§IDEA M-12.6 (eviction scan — applies the §IDEA M-7.5.1 Semi-Internal Consumption
Lemma to bound trailing-rebalance promotions), §IDEA M-12.9.3 (finite-time
contraction bound:
$O(|G|_0 \cdot h_V)$ total work)  
**Spec divergence:** Yes — see [below](#spec-divergence)  
**API:** [§API M-5.1](../docs/api.md#51-configv) (budget validation),
[§API M-5.2 Accessors](../docs/api.md#accessors) (`headroom`,
`soft_limit`, hard budget guarantee)  
**Surface:** 3 (Emulsion)

## Context

ADR-M-017 introduced dynamic depth control: when `node_count` exceeds
`budget`, the system tightens `D_evict` by 1 per `observe()` and
evicts newly eligible terminals. The documentation (§§IDEA M-7.4–7.5,
architecture.md, api.md) describes this budget as a **hard
ceiling**, but the implementation cannot enforce it as a per-call
invariant.

### Spec divergence

§§IDEA M-7.4–7.5 define **two** user-facing parameters: `budget`
(soft node-count target, triggers depth adjustment) and $H$
(hard ceiling, $H > \text{budget}$). The hard guarantee is
achieved via semi-internal count tracking: $|G| + S + 2 \leq H$,
where $S$ is the live semi-internal G-node count.

This ADR **collapses** `budget` and $H$ into a single
`Config.budget` that serves as the hard ceiling. The soft trigger
is derived internally as `soft_limit = budget − required_headroom`.
The hard guarantee is achieved via structural/convergence headroom
analysis rather than semi-internal tracking. This simplifies the
user-facing API (one parameter instead of two) and avoids
maintaining the semi-internal counter.

Notation: this ADR uses $S$ for the soft limit (not the
semi-internal count as in §IDEA M-7.5). The $\alpha_{\text{relax}}$
threshold also tracks the dynamic $S$ (i.e.,
`node_count < soft_limit × α_relax`) rather than the raw budget
as in the spec pseudocode.

### The overshoot problem

A single `observe()` can split (+2 nodes) *before* the eviction
step fires. Gate tightening proceeds at ±1 level per call. Eviction
can only remove *terminal* G-nodes. These three constraints create
a structural gap between the user-specified budget and the worst-
case actual node count.

### The V-tree height bound

The V-tree is a 2-3 tree. V-entries can occupy varying depths
(promotion moves entries to shallower positions), but every entry
occupies a child slot of a structural node. An entry at depth $d$
consumes a slot that could otherwise root a subtree holding up to
$3^{h-d}$ entries at the deepest level. The maximum total entry
count in a 2-3 subtree of height $h$ is therefore $3^h$, achieved
when all entries sit at the deepest level.

Eviction requires `depth > D_evict`. When `D_evict` reaches its
floor (`buffer + 1`), the maximum number of entries that can exist
at depth $\leq$ floor is bounded by:

$$n_{\max} = 3^{\,\text{buffer} + 1}$$

This is the theoretical hard wall: beyond this count, the V-tree
must contain entries deeper than the floor, and those become
eviction-eligible. There is no configuration that avoids this
structural ceiling.

| `depth_create` | `depth_evict` | buffer | floor | $n_{\max}$ |
|:--------------:|:-------------:|:------:|:-----:|:----------:|
| 1 | 2 | 1 | 2 | **9** |
| 2 | 4 | 2 | 3 | **27** |
| 3 | 6 | 3 | 4 | **81** |
| 4 | 8 | 4 | 5 | **243** |

If `budget < n_max`, the system converges to `budget` under
sustained load with only transient overshoot. If `budget ≥ n_max`,
there is no overshoot problem.

The danger zone is `budget` values that are small relative to
`n_max` — for example, `budget = 30` with `buffer = 3` gives
`n_max = 81`, meaning up to 51 nodes of overshoot while the gates
converge to the floor.

### What the user expects

A user who writes `budget: Some(50)` expects the library to
**never** allocate more than 50 G-nodes. The current implementation
cannot guarantee this because it uses `budget` as both the hard
ceiling promise and the soft trigger for depth control.

## Analysis

### Separate the hard limit from the soft trigger

The key insight: if we trigger depth tightening and eviction
*earlier* — at a count below the user's hard limit — the gates have
room to converge before the hard wall is reached.

Define:

- **Hard limit** $H$: the user-specified `budget`. Node count must
  *never* exceed this value.
- **Structural headroom** $M = 3^{\text{buffer}+1}$: the worst-case
  number of entries that can exist while immune to eviction at the
  floor.
- **Convergence headroom** $K = 2(D_c^{\text{live}} - 1)$: the
  worst-case overshoot above the soft limit while the gates tighten
  from the current $D_c$ to the floor, at one split (+2 nodes)
  per `observe()` call.
- **Required headroom** $R = \max(M, K)$: whichever dominates.
- **Soft limit** $S = H - R$: the internal trigger point for depth
  tightening and eviction.

### Two sources of overshoot

The soft limit must absorb overshoot from two independent sources:

1. **Structural immunity.** At the depth floor, up to $M$
   eviction-immune entries can exist (all at V-depth $\leq$ floor).
   The system cannot evict them regardless of gate position.

2. **Convergence lag.** Gate tightening proceeds at ±1 per call
   (ADR-M-017 Q2). From $D_c^{\text{live}}$ to the floor takes
   $D_c - 1$ steps. Each step may admit one split (+2 nodes)
   before the eviction scan fires: worst-case overshoot above $S$
   is $2(D_c - 1)$.

A static soft limit using only $M$ (the structural term) works
when $M \geq K$, which holds for typical configurations where
`buffer ≈ depth_create`. But ADR-M-017 Q5 decided that $D_c$ has
**no ceiling** — relaxation can raise $D_c$ above its initial
value when budget headroom persists. This means $K$ can grow
unboundedly at runtime, eventually exceeding $M$.

**Example.** $D_c^{\text{init}} = 3$, buffer = 1, $M = 9$. After
a quiet period, $D_c$ relaxes to 30. $K = 2 \times 29 = 58$.
A static soft limit $S = H - 9$ leaves only 9 nodes of headroom,
but convergence could produce 58 nodes of overshoot. The budget
would be violated.

### The dynamic soft limit

The solution is to recompute $S$ on every `adjust_depth_gates()`
call, tracking the current $D_c$:

$$S = H - \max\bigl(M,\; 2(D_c^{\text{live}} - 1)\bigr)$$

This is self-consistent because the proof is inductive:

1. At any moment, $S_{\text{now}} = H - \max(M, 2(D_c^{\text{now}} - 1))$.
2. Maximum further overshoot from this moment $= 2(D_c^{\text{now}} - 1)$.
3. Peak $\leq S_{\text{now}} + 2(D_c^{\text{now}} - 1) \leq S_{\text{now}} + R_{\text{now}} = H$.  $\checkmark$

During tightening, $D_c$ falls → $R$ drops → $S$ rises → the gap
closes from both sides. The system never needs more headroom than
it has reserved.

### Worked example

$D_c^{\text{init}} = 3$, buffer = 1, $M = 9$, $H = 100$.

| Phase | $D_c^{\text{live}}$ | $K = 2(D_c{-}1)$ | $R = \max(M, K)$ | $S = H - R$ |
|-------|:---:|:---:|:---:|:---:|
| Startup | 3 | 4 | 9 | **91** |
| Quiet (relaxes to 10) | 10 | 18 | 18 | **82** |
| Quiet (relaxes to 30) | 30 | 58 | 58 | **42** |
| Burst starts | 30 | 58 | 58 | **42** |
| Tightening (5 calls) | 25 | 48 | 48 | **52** |
| Near floor | 3 | 4 | 9 | **91** |

At the burst onset, tightening triggers at 42. Worst-case peak:
$42 + 58 = 100 = H$. Hard limit held. As $D_c$ drops during
convergence, $S$ rises — the system recovers effective capacity
as the threat recedes.

### Why this works

1. **Tightening starts early.** When `node_count` crosses $S$, the
   gates start dropping. The tree begins contracting with $R$
   nodes of headroom remaining.

2. **The headroom matches both threats.** The required headroom
   $R = \max(M, K)$ covers whichever term dominates at the current
   gate position. Since $S + R = H$, the total never exceeds $H$.

3. **No behavioral change for typical configs.** When
   $M \geq 2(D_c - 1)$ (the common case: buffer is comparable to
   $D_c$), the convergence term never dominates and $S$ stays at
   $H - M$ — identical to a static soft limit.

4. **Preserves unbounded relaxation.** ADR-M-017 Q5 allows $D_c$ to
   grow without a ceiling. The dynamic soft limit accommodates this
   without restricting adaptivity: deep relaxation simply lowers
   $S$ to reserve more headroom. The tree *can* use the deeper
   resolution, but starts tightening earlier to ensure it can
   converge in time.

5. **No split guard needed.** Because $S$ is always set to ensure
   $S + 2(D_c - 1) \leq H$, the soft limit mechanism alone is
   sufficient. There is no configuration where transient overshoot
   from splits can breach $H$. The guarantee is purely structural
   — no per-split check required.

6. **Validation: $H > R_{\text{init}}$.** At construction,
   $R_{\text{init}} = \max(M, 2(D_c^{\text{init}} - 1))$.
   `Config::validate()` rejects budgets that cannot accommodate the
   initial required headroom.

### Impact on α_relax

The relaxation threshold tracks the dynamic $S$: relax fires when
`node_count < S × α_relax`. After deep relaxation, $S$ is lower,
so the relaxation zone narrows. This is the correct behaviour: when
the system has committed to deep resolution (large $D_c$), it needs
more reserved headroom and should be more conservative about
relaxing further. When $D_c$ tightens back down, $S$ rises and the
relaxation zone widens again.

### After deep relaxation the tree operates in a smaller range

When $D_c$ has relaxed far above its initial value, $S$ drops
significantly (e.g., from 91 to 42 in the worked example). The
tree's effective operating range shrinks to `[0, S]`. This is the
inherent trade-off of unbounded relaxation: deeper resolution
requires reserving more headroom to ensure safe convergence. Under
the alternative of capping $D_c$ at its initial value, $S$ would
remain static — but the tree could never discover finer resolution
than the initial config intended. The dynamic soft limit makes this
trade-off explicit and automatic.

### Implementation

`adjust_depth_gates()` recomputes $S$ on every call:

```rust
fn adjust_depth_gates(&mut self) {
    let Some(budget) = self.config.budget else {
        return;
    };
    let count = self.node_count as usize;

    // Dynamic headroom: max(structural ceiling, convergence bound).
    // ADR-M-018: ensures S + 2(D_c − 1) ≤ H at every step.
    let convergence_bound =
        2 * (self.live_depth_create as usize).saturating_sub(1);
    let required_headroom = self.headroom.max(convergence_bound);
    let soft_limit = budget - required_headroom;
    assert!(
        soft_limit >= 1,
        "soft_limit must be >= 1 (budget={budget}, \
         headroom={required_headroom}, \
         D_c={}, buffer={})",
        self.live_depth_create,
        self.depth_buffer
    );
    self.soft_limit = Some(soft_limit);

    if count > soft_limit {
        // Tighten: lower D_evict by 1, floor at buffer + 1.
        let floor = self.depth_buffer + 1;
        if self.live_depth_evict > floor {
            self.live_depth_evict -= 1;
            self.live_depth_create =
                self.live_depth_evict - self.depth_buffer;
        }
    } else {
        let threshold = soft_limit as f64 * self.config.alpha_relax;
        let count_f = count as f64;
        if count_f < threshold {
            // Relax: raise D_evict by 1 (no ceiling).
            self.live_depth_evict += 1;
            self.live_depth_create =
                self.live_depth_evict - self.depth_buffer;
        }
    }
}
```

The structural headroom `M = 3^(buffer+1)` is still computed once
in `GvGraph::new()` and stored as `self.headroom`. The soft limit
is recomputed each call — O(1) additional cost (one subtraction,
one comparison).

## Decision

**Introduce a dynamic soft limit
$S = H - \max(M, 2(D_c^{\text{live}} - 1))$, recomputed on every
`adjust_depth_gates()` call.**

1. `Config::validate()` rejects
   `budget ≤ max(3^(buffer+1), 2*(depth_create - 1))` when
   `budget` is `Some`.

2. `GvGraph::new()` computes `headroom = 3^(buffer+1)` and the
   initial soft limit using the initial `depth_create`.

3. `adjust_depth_gates()` recomputes
   `soft_limit = budget - max(headroom, 2*(D_c_live - 1))`
   on every call, then triggers tightening or relaxation based
   on `node_count` vs `soft_limit`.

4. The budget guard in `observe()` step 8 triggers on
   `node_count > soft_limit`.

5. The `Config.budget` doc comment remains "hard ceiling on live
   G-node count" — because it now *is* a hard ceiling.

6. A public accessor `soft_limit() -> Option<usize>` is provided
   for observability. The returned value may differ between
   consecutive `observe()` calls as $D_c$ changes.

7. **No split guard.** The dynamic headroom reservation is
   provably sufficient: $S + 2(D_c - 1) \leq H$ holds at every
   step by construction. No per-split budget check is needed in
   `attempt_split()`.

## Consequences

- **Budget is a true hard ceiling.** `node_count` will never
  exceed `budget` under any observation sequence, proven by
  induction on the dynamic headroom invariant.

- **Plateau count bounded by the hard ceiling (§IDEA M-12.1.1).** The
  Plateau–Node-Count Bound $P \leq |G| \leq H$ is a direct
  corollary: the hard budget limits not only node count but also
  the maximum number of plateaus — and therefore structural
  complexity — under all conditions.

- **Minimum budget validation.** Users must set `budget >
  max(3^(buffer+1), 2*(depth_create - 1))`. For example, with
  buffer=3 and depth_create=3, the minimum budget is 82 (the
  structural term dominates). This is documented and enforced at
  construction time.

- **Earlier tightening under deep relaxation.** When $D_c$ has
  relaxed well above its initial value, the soft limit drops to
  reserve convergence headroom. The tree triggers tightening
  sooner, giving it room to converge before hitting $H$. This
  is the cost of unbounded relaxation — deeper resolution requires
  more reserved headroom.

- **No behavioral change for typical configs.** When the buffer
  is comparable to $D_c$ (the common case), $M$ dominates $K$ and
  the soft limit is effectively static at $H - M$.

- **Clean proof of correctness.** The hard guarantee follows from
  a single inductive step: $S_{\text{now}} + 2(D_c^{\text{now}} - 1) \leq H$.
  No split guard, no per-call emergency check. The budget contract
  is upheld by the feedback loop's own headroom reservation.

- **No API change.** The user-facing `Config` is unchanged. The
  dynamic soft limit is an internal implementation detail.

- **Split semantics preserved.** Because the hard guarantee is
  structural (not enforced by refusing splits), the V-tree's
  competitive tournament governs all splits. No region is
  artificially suppressed at the budget boundary. This avoids
  benchmark inflation from suppressed splits and preserves normal
  §IDEA M-13.5 compounding dynamics.

- **Test updates.** Tests that set very small budgets (e.g.,
  `budget: Some(3)`) will need to ensure the budget exceeds the
  minimum. Integration tests should verify `node_count ≤ budget`
  as a hard invariant after every `observe()`.

## References

- §§IDEA M-7.4–7.5 (dynamic depth control, hard budget guarantee,
  semi-internal tracking — this ADR diverges on mechanism;
  see [Spec divergence](#spec-divergence))
- §IDEA M-19 (spray resistance — §IDEA M-19.3 relies on the hard
  budget from §IDEA M-7.5 for the resource bound)
- ADR-M-017 (depth gate mechanics, ±1 rate, floor, buffer,
  unbounded relaxation)
- ADR-M-015 (eviction scan design, budget guard)
- §IDEA M-13.5 (benchmark compounding, self-dampening)
