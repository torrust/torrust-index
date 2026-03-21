# ADR-M-017: Dynamic Depth Control

**Status:** Decided  
**Date:** 2026-02-25  
**Updated:** 2026-03-03  
**Superseded in part by:** [ADR-M-018](018-hard-budget-guarantee.md) (dynamic soft limit trigger)  
**API:** [§API M-5.2](../docs/api.md#52-gvgraphc-v-n--the-negative)
(accessors: `depth_evict`, `depth_create`, `depth_buffer`),
[§API M-5.1](../docs/api.md#51-configv) (`alpha_relax`,
`bounded_eviction`)  
**Surface:** 3 (Emulsion)

## Context

The index uses two depth gates — `D_create` (maximum V-depth for
splits) and `D_evict` (minimum V-depth for eviction eligibility).
The invariant **D-I3** requires `D_create < D_evict`, creating a
buffer zone where entries live on borrowed time (§IDEA M-7).

Phase 2 treats these as static `Config` values. Phase 3 introduces
**dynamic depth control** (§IDEA M-7.4): a feedback loop that
adjusts `D_evict` and `D_create` based on memory pressure
(`node_count` vs `budget`).

The spec's pseudocode (§IDEA M-7.4):

```
function adjust_depth_gates():
    if total_nodes > budget:
        D_evict  ← max(1, D_evict − 1)    // maintain D-I4
        D_create ← D_evict − buffer       // maintain D-I3
    else if total_nodes < budget × α_relax:
        D_evict  ← D_evict + 1
        D_create ← D_evict − buffer
```

> **Parameter mapping.** The spec (§IDEA M-7.4) defines _two_ count
> thresholds: `budget` (soft node-count target) and `H` (hard
> ceiling, constrained `H > budget`). In the implementation
> (§API M-5.1), the Config field `budget` corresponds to the spec's
> `H` — a hard ceiling. The spec's soft `budget` role is filled by
> the dynamic `soft_limit = budget − max(headroom, 2*(D_c − 1))`,
> recomputed on every `adjust_depth_gates()` call
> ([ADR-M-018](018-hard-budget-guarantee.md)).
> Throughout this ADR, unqualified "budget" in discussion text
> refers to the Config field (= spec's H) unless otherwise noted.

Seven interrelated parameter and policy choices must be resolved.

**Important structural property (§IDEA M-13.5).** The spec proves that
_tree-level_ oscillation (expand → evict → re-expand → evict …) is
self-dampening regardless of parameter tuning. Each expand-contract
cycle folds absorbed value into the parent's benchmark via
absorption (ADR-M-014). After $k$ cycles at depth $D$, re-reaching
that depth costs $\sim D^2 \theta / 2$ — quadratic in depth. The
tree "remembers" prior over-expansion and demands progressively
stronger evidence before re-investing. This means the feedback
loop's tuning parameters affect **efficiency** (how much wasted
work occurs) rather than **correctness** (whether the system
converges). The system always converges; the question is how
smoothly.

**Spec reference:** §IDEA M-7.4 (dynamic depth control), §IDEA M-7
(depth gates, D-I1, D-I2, D-I3), §IDEA M-12.7 (bottom-up contraction /
tides — how gate changes manifest over successive observations),
§IDEA M-12.9.4 (full contraction to root requires progressive `D_evict`
tightening), §IDEA M-12.10 (semi-internal chain erosion time depends on
gate dynamics), §IDEA M-13.5 (benchmark compounding).

**Related ADRs:**

- [ADR-M-013](013-eviction-eligibility.md): eligibility uses `D_evict`
- [ADR-M-014](014-value-absorption.md): absorption hardens benchmarks
- [ADR-M-015](015-eviction-scan-design.md): eviction scan is budget-
  guarded; `adjust_depth_gates` must fire before the scan
- [ADR-M-018](018-hard-budget-guarantee.md): introduces an internal
  soft limit to convert the budget into a true hard ceiling
  (resolves the overshoot gap identified in this ADR's design)

---

## Analysis

### The adjustment is O(1)

`adjust_depth_gates` performs at most:

- One headroom computation (`max(headroom, convergence_bound)`)
- Two integer comparisons (`node_count` vs `soft_limit`,
  `node_count` vs `soft_limit × α_relax`)
- Two field writes (`depth_evict`, `depth_create`)
- One floor clamp

No arena access, no tree traversal. This is safe to call on every
`observe()` invocation unconditionally (when `budget` is `Some`).

### Current code inventory

| Item                               | Location                         | State                                                     |
| ---------------------------------- | -------------------------------- | --------------------------------------------------------- |
| `Config.depth_create: u32`         | [graph.rs](../src/graph.rs#L30)  | Initial value on Config                                   |
| `Config.depth_evict: u32`          | [graph.rs](../src/graph.rs#L33)  | Initial value on Config                                   |
| `Config.budget: Option<usize>`     | [graph.rs](../src/graph.rs#L36)  | Hard ceiling (= spec's H)                                 |
| `Config.alpha_relax: f64`          | [graph.rs](../src/graph.rs#L40)  | Relaxation fraction                                       |
| `Config.bounded_eviction: bool`    | [graph.rs](../src/graph.rs#L44)  | Bounded eviction flag                                     |
| `GvGraph.node_count: u32`          | [graph.rs](../src/graph.rs#L151) | Maintained by split (+2), evict (−1), legacy promote (+1) |
| `GvGraph.live_depth_evict: u32`    | [graph.rs](../src/graph.rs#L157) | Mutable runtime gate                                      |
| `GvGraph.live_depth_create: u32`   | [graph.rs](../src/graph.rs#L160) | Mutable runtime gate                                      |
| `GvGraph.depth_buffer: u32`        | [graph.rs](../src/graph.rs#L163) | Fixed at construction                                     |
| `Config.validate()`                | [graph.rs](../src/graph.rs#L54)  | Asserts D-I3, D_create ≥ 1, α_relax, budget headroom      |
| `split::attempt_split` depth guard | [split.rs](../src/split.rs#L83)  | Reads `graph.live_depth_create`                           |

`depth_create` and `depth_evict` are mutable runtime state on
`GvGraph` (shadow fields), initialized from `Config` and adjusted
by the feedback loop. `Config` remains immutable — it records the
user's original specification.

### Interaction with the observe() pipeline

Per ADR-M-015, the `observe()` pipeline (core steps only;
plateau-related steps omitted — see ADR-M-026):

```
1.  route_to_receiver         // reads G-tree
2.  accumulate + V-propagate  // writes G-node, V-entry; checks violations
3.  recompute_g_sums          // writes G-tree upward
5.  attempt_split             // reads live_depth_create ← USES LIVE GATE
6.  rebalance                 // mutates V-tree; receives live_depth_evict
                              //   for depth-gated legacy promotion (§IDEA M-11.9)
7.  adjust_depth_gates        // reads node_count, writes gates + soft_limit
8.  check_evictions (if over  // reads live_depth_evict ← USES FRESH GATE
    soft_limit)
```

> **Step numbering.** Steps 4, 9, and 10 (plateau maintenance,
> normalize, P-I4 repair) are omitted — they are feature-gated and
> orthogonal to depth control. The numbers match the implementation
> comments in [observe.rs](../src/observe.rs). ADR-M-015 documents
> the full pipeline.

Adjusting gates between rebalance (step 6) and eviction (step 8)
ensures:

- **Step 5 uses the current `live_depth_create`** — which may have
  been lowered by the _previous_ observation's adjustment. This is
  correct: the split threshold should be as current as possible.
- **Step 6 receives `live_depth_evict`** — rebalance uses it for
  depth-gated legacy promotion (§IDEA M-11.9): when a semi-internal
  entry's violation resolves and the heir's V-depth would exceed
  `D_evict`, the dispatcher uses skip-promote instead, avoiding a
  futile allocate–evict round-trip.
- **Step 8 uses the freshly adjusted `live_depth_evict`** — the
  eviction scan immediately reflects tightening. If `D_evict`
  dropped by 1, a new layer of entries is now eligible.

Adjusting _before_ the scan (not after) means a single observation
can both tighten the threshold and evict newly eligible entries in
one round-trip. There is no one-round delay.

---

## Q1: When to adjust depth gates

### Options

| Option                        | Timing                                     | Latency                                 |
| ----------------------------- | ------------------------------------------ | --------------------------------------- |
| **A. Before check_evictions** | Step 7 position above                      | Zero delay — scan uses fresh gates      |
| **B. After check_evictions**  | After step 8                               | One-round delay — scan uses stale gates |
| **C. Separate method**        | User calls `adjust_depth_gates()` manually | User-controlled timing                  |

### Evaluation

**Option B** creates a one-observation lag between detecting over-
budget and tightening. This means the eviction scan in that same
observation uses the old (higher) `D_evict`, potentially evicting
fewer entries. On the next observation `D_evict` is lower, but
node_count may have grown further from the split in step 5. The
system converges a round late.

**Option C** is maximally flexible but breaks the budget-as-contract
model. A user who sets `budget = 50` expects the library to stay
at or near 50 nodes without manual intervention.

**Option A** is the natural choice. The adjustment is O(1) — it
adds negligible cost to the hot path. It ensures the eviction scan
in the same observation uses the freshest threshold, giving the
tightest possible response.

**Edge case: adjustment + no eviction.** If `adjust_depth_gates`
lowers `D_evict` but `node_count` has not yet exceeded `soft_limit`
(e.g., `α_relax` triggered a relaxation last round, then a burst
of splits pushed count to `soft_limit + 1`), the budget guard still
fires and the scan runs with the new threshold. Correct.

**Edge case: budget is None.** When `budget` is `None`, there is
no memory pressure signal and no basis for adjustment. The depth
gates remain at their initial Config values permanently. Users who
want manual control can call `check_evictions()` directly, but the
gates don't shift. This is consistent with ADR-M-015 (no automatic
eviction when `budget: None`).

### Decision: Option A

Adjust depth gates immediately before the eviction check, within
the `observe()` pipeline. Only when `budget` is `Some`.

---

## Q2: Adjustment rate

### Options

| Option               | Step size              | Convergence        | Risk                        |
| -------------------- | ---------------------- | ------------------ | --------------------------- |
| **A. ±1 per call**   | 1 level                | O(depth) calls     | None                        |
| **B. Proportional**  | `(count − budget) / K` | O(1) calls         | Over-eviction if K is wrong |
| **C. Binary search** | halves gap             | O(log depth) calls | Overshoot, complex          |

### Evaluation

**Option A** is the conservative choice. The feedback loop fires on
every `observe()`, so at worst it takes `depth` observations to
converge from an initial `D_evict` down to the floor. In practice,
each step exposes one new layer of entries; the subsequent eviction
scan removes them. One observation = one layer peeled off. The
convergence rate is tied to the actual tree structure, not an
arbitrary constant.

**Option B** is tempting for burst scenarios (e.g., tree suddenly
doubles in size) but introduces the constant `K`, which must be
tuned. If `K` is too small, `D_evict` drops multiple levels and
mass-evicts aggressively — more entries are removed than necessary,
wasting the accumulated detail. If `K` is too large, it's
equivalent to Option A. A wrong `K` causes churn: tighten too far
→ mass eviction → gate relaxes into headroom → tree slowly refills
→ tighten again. Benchmark compounding (§IDEA M-13.5) prevents this from
being _catastrophic_ — each cycle is more expensive than the last —
but the wasted eviction-and-rebuild work is still undesirable.

**Option C** has the same overshoot problem with added complexity.

**Why ±1 is sufficient:** Each level of the G-tree represents a
power-of-two subdivision. Lowering `D_evict` by 1 exposes entries
at one resolution level. Those entries are the _least globally
significant_ terminal entries (they're deepest in the V-tree). The
number of entries at each V-tree level roughly doubles (it's a
2–3 tree), so each step evicts an exponentially growing batch.
Convergence is geometric, not linear.

**Worked example:** Budget = 100, current count = 200, `D_evict` = 10. Step 1: lower to 9, evict ~50 entries at depth 10. Step 2:
still at 150, lower to 8, evict ~25 entries at depth 9. Step 3:
at 125, lower to 7, evict ~12. Step 4: at 113, lower to 6. After
~5 observations the tree is back to budget. Each observation does
O(evicted_this_round) work, which is bounded by the budget excess
amortized over the convergence steps.

### Decision: Option A — ±1 per call

---

## Q3: α_relax — relaxation threshold

### What α_relax actually controls

Benchmark compounding (§IDEA M-13.5) already prevents the _tree_ from
oscillating — each expand-contract cycle makes re-expansion
quadratically more expensive. The tree self-dampens structurally.

`α_relax` controls something different: **gate-level churn**.
Without a dead zone between the tighten and relax thresholds, the
depth gates toggle on nearly every observation that lands near the
budget boundary. Each toggle is O(1), but a tighten triggers an
eviction scan (O(evictable_tips)), and a relax allows a future
split that may trigger the next tighten. The wasted work is not in
the tree restructuring (which compounding makes cheap to no-op) but
in the **unnecessary eviction scans and their trailing rebalances**.

The interval `[α_relax × S, S]` (where `S` = `soft_limit`) is a
dead zone where neither tightening nor relaxation occurs:

```
     0         α_relax × S              S
     |              |                    |
     |  RELAX zone  |   DEAD ZONE       | TIGHTEN zone
     |  D_evict += 1|   no adjustment   | D_evict -= 1
```

> The analysis below uses `budget` as shorthand for the effective
> threshold. In the final implementation the comparison is against
> `soft_limit` (= `budget − headroom`, per [ADR-M-018](018-hard-budget-guarantee.md)),
> so the effective dead zone is narrower than the raw budget
> fractions stated. The qualitative conclusions are unchanged.

### Options

| α_relax  | Dead zone width | Behavior                                                            |
| -------- | --------------- | ------------------------------------------------------------------- |
| 1.0      | 0% of budget    | Gates toggle every ~2 observations near budget — maximal scan churn |
| 0.9      | 10%             | Narrow band — gates settle briefly, but 5 splits exhaust headroom   |
| **0.75** | **25%**         | Moderate — gates stay settled through typical load variation        |
| 0.5      | 50%             | Wide band — tree stays shallow even with ample headroom             |

### Evaluation

**α_relax = 1.0** causes the gates to toggle on nearly every
observation near the budget boundary. The moment eviction brings
`node_count` to `budget − 1`, the gate relaxes. The next split
pushes to `budget + 1`, tightening fires, an eviction scan runs,
and the cycle repeats every two observations. The _tree_ doesn't
actually oscillate — the hardened benchmarks from §IDEA M-13.5 mean the
relaxed gate permits splits that can't actually happen (the
benchmark bar is too high). But the gate still toggles, and the
eviction scan still fires, doing O(scan) work for zero evictions.
This is wasteful, not catastrophic — but easily avoided.

**α_relax = 0.9** provides a 10% dead zone. For budget = 100,
that's 10 nodes of headroom. A single split adds 2 nodes net, so
5 splits exhaust the band. Under active observation of many
coordinates, the gates start toggling again within a few dozen
observations.

**α_relax = 0.75** gives a 25% dead zone (25 nodes for budget =
100). This requires 12–13 splits to exhaust. After a tightening
episode brings node_count down, the tree has substantial room to
grow back before hitting the budget — meaning the gates stay stable
through typical load variation. Meanwhile, the tree still relaxes
when genuinely under-utilized (below 75% of budget), recovering
resolution where warranted.

**α_relax = 0.5** wastes half the budget as idle headroom. A user
who sets `budget = 100` would see the tree stay at ~50 nodes even
when deeper resolution would improve accuracy.

**Sensitivity analysis:** The dead zone's practical impact depends
on the split rate. A high-throughput system observing millions of
distinct coordinates will split frequently; it benefits from a
wider dead zone to avoid scan churn. A low-throughput system with
sparse observations rarely splits and rarely approaches the budget;
the dead zone width matters less. The 25% default works well for
both — the high-throughput case gets 25 nodes of headroom before
the next tightening, and the low-throughput case rarely enters the
dead zone at all.

### Decision: α_relax = 0.75, configurable

Recommended value 0.75. Exposed in `Config` as `alpha_relax: f64`.
Validated at construction: `0.0 < alpha_relax < 1.0`. Equality with
0.0 is useless (never relax). Equality with 1.0 causes maximal gate
churn (harmless but wasteful). There is no `Default` impl on
`Config` — all fields must be specified explicitly (§API M-5.1).

---

## Q4: Buffer preservation

### How D_create tracks D_evict

The "buffer" is the gap `D_evict − D_create`. It ensures D-I3 and
determines the width of the "borrowed time" zone.

### Options

| Option              | Formula                               | Behavior                         |
| ------------------- | ------------------------------------- | -------------------------------- |
| **A. Fixed gap**    | `D_create = D_evict − buffer_initial` | Gap constant regardless of depth |
| **B. Proportional** | `D_create = D_evict × (1 − ratio)`    | Gap scales with depth            |
| **C. Independent**  | Two variables, two feedback loops     | Maximum flexibility              |

### Evaluation

**Option A** is the simplest and most predictable. `buffer_initial`
is computed once at construction: `config.depth_evict −
config.depth_create`. This is the user's stated intent — "I want
this many levels of buffer zone." Dynamic adjustment shifts both
gates in lockstep, preserving the original gap.

**Option B** changes semantics: at high `D_evict`, the buffer grows
(more borrowed-time levels); at low `D_evict`, it shrinks (less
time to prove). This is unintuitive and hard to reason about. At
`D_evict = buffer + 1` (the floor), a proportional gap would need
special casing.

**Option C** is over-engineered. The spec defines a single feedback
variable (`D_evict`); `D_create` is derived. Two independent loops
would create interaction effects and potential D-I3 violations.

**Edge case:** When `D_evict` is tightened toward the floor,
`D_create = D_evict − buffer` approaches 1. If `buffer = 5` and
`D_evict` reaches 6, `D_create = 1`. This means splits are blocked
for almost all entries (only V-depth ≤ 1 can split, i.e., only the
V-root's children). This is correct behavior: under extreme memory
pressure, the tree should stop expanding entirely.

### Decision: Option A — fixed gap from initial Config

`buffer = config.depth_evict − config.depth_create`, computed once,
stored internally. D_create always equals `D_evict − buffer`.

---

## Q5: Floor and ceiling values

### Constraints

| Bound          | Expression             | Reason                                                                                                                                                                             |
| -------------- | ---------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Hard floor** | `D_evict ≥ buffer + 1` | Ensures `D_create = D_evict − buffer ≥ 1`. At `D_create = 0`, no entry could ever split — not even the root's entry. `D_create ≥ 1` guarantees the root can always subdivide once. |
| **Hard floor** | `D_create ≥ 1`         | (Implied by the above.)                                                                                                                                                            |
| **Ceiling**    | No hard ceiling        | See below.                                                                                                                                                                         |

> **Strengthening D-I4.** The spec's D-I4 requires `D_evict ≥ 1`.
> The floor adopted here (`D_evict ≥ buffer + 1`, i.e. `D_create ≥ 1`)
> is strictly stronger when `buffer ≥ 1` (which D-I3 + `buffer ≥ 1`
> from §IDEA M-7.1 guarantees). This is intentional — `D_create = 0` would
> block all splits including the root's first subdivision, making
> the tree permanently trivial.

### Should D_evict have an upper bound?

| Option                                | Ceiling                          | Effect                                      |
| ------------------------------------- | -------------------------------- | ------------------------------------------- |
| **A. Capped at initial Config value** | `D_evict ≤ config.depth_evict`   | Tree never deeper than originally specified |
| **B. Capped at N**                    | `D_evict ≤ N` (domain bit-width) | Tree can grow to maximum spatial resolution |
| **C. No ceiling**                     | Unbounded (except u32::MAX)      | Maximum freedom                             |

**Option A** is the most conservative. However, it prevents the
tree from recovering resolution after a pressure spike. Imagine:

1. User sets `depth_evict = 20`, `budget = 1000`.
2. Burst traffic pushes `node_count` to 2000. `D_evict` drops to
   15 over 5 observations. Many entries evicted.
3. Traffic subsides. `node_count` falls to 300. But `D_evict` can
   only relax back to 20 (the initial) — which is fine.

Now imagine the user set a conservative initial `depth_evict = 10`
but budget = 10000. The tree has massive headroom but can never go
deeper than 10. Option A would lock the tree into this limitation.

**Option B** caps at N (the domain bit-width, e.g., 64 for a
`u64` coordinate). This is the natural maximum — the G-tree can't
subdivide more than N times (intervals become atomic). In practice,
the V-tree is much deeper than the G-tree (V-depth ≫ G-depth
because structural nodes add levels). So `D_evict = N` is
effectively "no ceiling" for the G-tree, but the V-tree's `D_evict`
could theoretically exceed it.

Wait — `D_evict` and `D_create` are **V-tree depths**, not G-tree
depths. The V-tree can be arbitrarily deep (each split adds 1–2
structural levels; rebalancing also adds levels). V-depth is not
bounded by N. So `N` is not a meaningful ceiling for V-tree depth
gates.

**Option C** imposes no ceiling beyond `u32::MAX`. The feedback loop
naturally limits relaxation: `D_evict` only increases when
`node_count < α_relax × soft_limit`, and it only increases by 1 per
observation. The tree can only grow as fast as observations arrive.
The budget itself is the effective ceiling — once the soft limit is
exceeded, tightening restores the balance.

### Decision: No hard ceiling; hard floor at buffer + 1

- `D_evict` can relax above the initial `config.depth_evict` if
  budget headroom persists. The initial value is a starting point.
- The hard floor `D_evict ≥ buffer + 1` guarantees D-I3 and
  `D_create ≥ 1` at all times.
- In practice, `D_evict` will rarely exceed its initial value by
  more than a few levels, because each relaxation requires ample
  headroom below `α_relax × soft_limit`.

**Interaction with the hard budget guarantee.** Because `D_create`
is unbounded above, the worst-case convergence overshoot —
$2(D_c^{\text{live}} - 1)$ splits during gate tightening — is also
unbounded. [ADR-M-018](018-hard-budget-guarantee.md) resolves this by
making the soft limit _dynamic_: it recomputes
$S = H - \max(M, 2(D_c - 1))$ on every `adjust_depth_gates()` call.
Deep relaxation lowers $S$ to reserve convergence headroom; the
tree can still use the deeper resolution but triggers tightening
earlier. This preserves unbounded relaxation without requiring a
per-split budget guard.

---

## Q6: Configuration surface

### Current Config fields

```rust
pub struct Config<V: Accumulator> {
    pub split_threshold: V,
    pub depth_create: u32,
    pub depth_evict: u32,
    pub budget: Option<usize>,
}
```

### Phase 3 additions

Only one new field is needed: `alpha_relax`.

The buffer is computed, not configured:

```rust
pub struct Config<V: Accumulator> {
    /// θ — minimum intensity required for a G-node to split.
    pub split_threshold: V,
    /// Initial D_create — maximum V-depth at which a split is
    /// authorized. Used to compute the buffer.
    pub depth_create: u32,
    /// Initial D_evict — minimum V-depth for eviction eligibility.
    pub depth_evict: u32,
    /// Optional hard ceiling on live G-node count (§IDEA M-7.4).
    /// When `Some`, dynamic depth control is enabled.
    pub budget: Option<usize>,
    /// α_relax — relaxation threshold as a fraction of soft_limit.
    /// D_evict increases when node_count < soft_limit × alpha_relax.
    /// Recommended: 0.75. Must be in (0.0, 1.0).
    /// Ignored when budget is None.
    pub alpha_relax: f64,
    /// If true, budget-triggered eviction stops early once
    /// node_count ≤ soft_limit. See ADR-M-015.
    pub bounded_eviction: bool,
}
```

### Validation additions

```rust
impl<V: Accumulator> Config<V> {
    fn validate(&self) {
        assert!(
            self.depth_create < self.depth_evict,
            "Config: D_create ({}) must be < D_evict ({}) (idea.md D-I3)",
            self.depth_create, self.depth_evict
        );
        assert!(
            self.depth_create >= 1,
            "Config: D_create ({}) must be >= 1",
            self.depth_create
        );
        assert!(
            self.alpha_relax > 0.0 && self.alpha_relax < 1.0,
            "Config: alpha_relax ({}) must be in (0.0, 1.0)",
            self.alpha_relax
        );
        if let Some(budget) = self.budget {
            let buffer = self.depth_evict - self.depth_create;
            let headroom = 3usize.pow(buffer + 1);
            let convergence = 2 * (self.depth_create as usize)
                .saturating_sub(1);
            let required = headroom.max(convergence);
            assert!(
                budget > required,
                "Config: budget ({budget}) must be > \
                 max(3^(buffer+1), 2*(D_c-1)) = {required} \
                 (ADR-M-018)"
            );
        }
    }
}
```

### Decision: Add alpha_relax and bounded_eviction to Config

`alpha_relax`: recommended value `0.75`. `bounded_eviction`:
recommended value `true` (per ADR-M-015). Both are validated at
construction time. `Config` has no `Default` impl — callers
must specify all fields explicitly (§API M-5.1).

---

## Q7: Runtime state (Config vs Graph)

### The key question

`Config` is currently stored on `GvGraph` and read from directly
during operations (e.g., `graph.config.depth_create` in split.rs).
Dynamic depth control mutates the depth gates. Where do the mutable
values live?

### Options

| Option                     | Config                                  | Graph                                                                      | Mutation                                          |
| -------------------------- | --------------------------------------- | -------------------------------------------------------------------------- | ------------------------------------------------- |
| **A. Mutable Config**      | `depth_create/evict` are mutable        | Graph stores Config                                                        | `graph.config.depth_evict -= 1`                   |
| **B. Shadow fields**       | Config stays immutable (initial values) | Graph has `live_depth_evict: u32`, `live_depth_create: u32`, `buffer: u32` | Graph fields mutated; split.rs reads graph fields |
| **C. Interior mutability** | `Cell<u32>` inside Config               | Graph stores Config                                                        | `config.depth_evict.set(...)`                     |

### Evaluation

**Option A** is tempting for simplicity — just mutate the existing
field. But it conflates "what the user asked for" (initial
specification) with "what the system decided" (runtime state). After
dynamic adjustment, `graph.config().depth_evict` would return the
current live value, not the user's original intent. A caller wanting
to know the initial configuration would have no way to retrieve it.
Additionally, `Config` is `Clone` and may be captured in a snapshot;
mutating it through the graph changes semantics.

**Option B** keeps a clean separation:

- `Config` = immutable specification, set once at `GvGraph::new()`.
- `GvGraph.live_depth_evict`, `live_depth_create`, `buffer` =
  mutable runtime state, initialized from Config.
- `split.rs` reads `graph.live_depth_create` instead of
  `graph.config.depth_create`.
- `graph.config()` always returns the original specification.

This sets a clear precedent: `Config` is a pure input; the graph
owns all mutable state.

**Option C** mixes concerns — `Config` would look immutable from
most call sites but contain hidden mutation. This is confusing and
non-idiomatic Rust.

### Decision: Option B — shadow fields on GvGraph

Add three fields to `GvGraph`:

```rust
pub struct GvGraph<C: Coordinate, V: Accumulator, const N: u32> {
    // ... existing fields ...

    /// Live D_evict — adjusted by dynamic depth control.
    /// Initialized to `config.depth_evict`.
    pub(crate) live_depth_evict: u32,
    /// Live D_create — adjusted by dynamic depth control.
    /// Initialized to `config.depth_create`.
    pub(crate) live_depth_create: u32,
    /// Buffer zone width, computed once:
    /// `config.depth_evict - config.depth_create`.
    pub(crate) depth_buffer: u32,
}
```

Initialize in `GvGraph::new()`:

```rust
live_depth_evict: config.depth_evict,
live_depth_create: config.depth_create,
depth_buffer: config.depth_evict - config.depth_create,
```

Update `split.rs` to read `graph.live_depth_create` instead of
`graph.config.depth_create`. Update eviction to read
`graph.live_depth_evict`.

Expose public accessors:

```rust
impl<C: Coordinate, V: Accumulator, const N: u32> GvGraph<C, V, N> {
    /// Current (dynamic) D_evict.
    pub const fn depth_evict(&self) -> u32 { self.live_depth_evict }
    /// Current (dynamic) D_create.
    pub const fn depth_create(&self) -> u32 { self.live_depth_create }
    /// Original Config specification.
    pub const fn config(&self) -> &Config<V> { &self.config }
}
```

---

## The adjust_depth_gates implementation

Bringing all decisions together:

```rust
/// Adjust D_evict and D_create based on memory pressure.
///
/// Called inside `observe()` after `rebalance()` and before
/// `check_evictions()`. Only fires when `budget` is `Some`.
///
/// Recomputes the dynamic soft limit S = H − max(M, 2(D_c − 1))
/// on every call (ADR-M-018). Tightening / relaxation / dead-zone
/// decisions use the freshly computed S.
///
/// D-I3 is maintained at all times via the fixed buffer.
fn adjust_depth_gates(&mut self) {
    let Some(budget) = self.config.budget else { return };
    let count = self.node_count as usize;

    // Dynamic headroom: max(structural ceiling, convergence bound).
    // ADR-M-018: ensures S + 2(D_c − 1) ≤ H at every step.
    let convergence_bound =
        2 * (self.live_depth_create.saturating_sub(1)) as usize;
    let required_headroom = self.headroom.max(convergence_bound);
    let soft_limit = budget - required_headroom;
    assert!(
        soft_limit >= 1,
        "soft_limit must be >= 1 (budget={budget}, \
         headroom={required_headroom})",
    );
    self.soft_limit = Some(soft_limit);

    if count > soft_limit {
        // Tighten: lower D_evict by 1, floor at buffer + 1.
        let floor = self.depth_buffer + 1;
        if self.live_depth_evict > floor {
            self.live_depth_evict -= 1;
            self.live_depth_create = self.live_depth_evict - self.depth_buffer;
        }
    } else if (count as f64) < soft_limit as f64 * self.config.alpha_relax {
        // Relax: raise D_evict by 1 (no ceiling).
        self.live_depth_evict += 1;
        self.live_depth_create = self.live_depth_evict - self.depth_buffer;
    }
}
```

### Pipeline integration

```rust
pub fn observe<O: Observation<V>>(&mut self, coord: C, delta: O) {
    // Steps 1–6: route, accumulate, G-sums, split, rebalance
    // ...

    // Step 7: Dynamic depth control (§IDEA M-7.4, ADR-M-017)
    //   — also recomputes the dynamic soft limit (ADR-M-018).
    self.adjust_depth_gates();

    // Step 8: Budget-guarded eviction (ADR-M-015, ADR-M-018)
    //   — triggers on soft_limit, not raw budget.
    if let Some(soft_limit) = self.soft_limit {
        if self.node_count as usize > soft_limit {
            if self.config.bounded_eviction {
                self.check_evictions_bounded(
                    self.node_count as usize - soft_limit,
                );
            } else {
                self.check_evictions();
            }
        }
    }
}
```

---

## Decision Summary

| Question               | Decision                                            | Rationale                                                                                                                                 |
| ---------------------- | --------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| **Q1: Timing**         | Before `check_evictions` (step 7 in pipeline)       | O(1) cost; scan uses fresh thresholds; zero delay                                                                                         |
| **Q2: Rate**           | ±1 level per call                                   | Geometric convergence via exponentially growing eviction batches per level; avoids over-eviction; trivial to reason about                 |
| **Q3: α_relax**        | 0.75 (configurable, validated ∈ (0, 1))             | 25% dead zone prevents gate churn and unnecessary scans; tree-level oscillation already prevented by benchmark compounding (§IDEA M-13.5) |
| **Q4: Buffer**         | Fixed gap from initial Config                       | `buffer = config.depth_evict − config.depth_create`, preserved across all adjustments                                                     |
| **Q5: Floor/ceiling**  | Floor: `D_evict ≥ buffer + 1`; no ceiling           | Floor ensures D-I3 + `D_create ≥ 1`; no ceiling allows recovery above initial depth                                                       |
| **Q6: Config surface** | Add `alpha_relax: f64` and `bounded_eviction: bool` | Minimal surface; buffer is derived, not configured                                                                                        |
| **Q7: Mutability**     | Shadow fields on GvGraph; Config stays immutable    | Clean separation of specification (Config) from runtime state (live gates)                                                                |

---

## Consequences

- **`graph.rs`:** Add `live_depth_evict`, `live_depth_create`,
  `depth_buffer` shadow fields. Initialize from Config in `new()`.
  Add `adjust_depth_gates()` method. Add `depth_evict()` and
  `depth_create()` public accessors. Add `alpha_relax: f64` and
  `bounded_eviction: bool` to Config. Extend `validate()`.

- **`split.rs`:** Change `graph.config.depth_create` reads to
  `graph.live_depth_create`. This is a one-line change
  ([split.rs L83](../src/split.rs#L83)).

- **`observe.rs`:** Add steps 7 and 8 to the `observe()` pipeline.

- **`invariants.rs`:** Add a D-I3 runtime check:
  `live_depth_create < live_depth_evict`. Verify
  `live_depth_create = live_depth_evict - depth_buffer`. Verify
  `live_depth_evict ≥ depth_buffer + 1`.

- **Testing:**
  - Unit test: `adjust_depth_gates` tightens when over budget.
  - Unit test: `adjust_depth_gates` relaxes when under α_relax.
  - Unit test: dead zone — no adjustment when in band.
  - Unit test: floor — `D_evict` doesn't drop below `buffer + 1`.
  - Unit test: ceiling — `D_evict` can grow above initial value.
  - Integration test: 500 observations with budget = 30, verify
    `node_count` converges and invariants hold throughout.

- **Semi-internal chain erosion (§IDEA M-12.10).** Dynamic `D_evict`
  tightening is the primary control for eroding semi-internal
  chains. A chain of depth $k$ requires $k$ tides at best. Under
  raw accumulation on a hot chain, erosion may stall without
  further tightening — correct behaviour, as the chain entries
  are genuinely significant. §IDEA M-12.9.4 proves that full contraction
  to the root requires either progressive `D_evict` tightening or
  sustained decay.

- **Precedent:** `Config` is immutable after construction. All
  mutable behavioral state lives on `GvGraph` itself. Future
  parameters (e.g., Phase 4 decay rate) should follow this pattern.

- **Additive change to Config:** Adding `alpha_relax` and
  `bounded_eviction` requires updating every `Config { ... }` literal
  in tests. `Config` has no `Default` impl — explicit construction
  is preferred (no hidden defaults for correctness-critical
  parameters like `split_threshold` and `depth_create`). A builder
  or default helper could be considered in a future ADR.

- **Overshoot gap: resolved by ADR-M-018.** The ±1 adjustment rate
  combined with the pipeline ordering (split before eviction) means
  `node_count` can transiently exceed the soft limit $S$. Two
  sources of overshoot exist: the structural ceiling
  ($M = 3^{\text{buffer}+1}$ entries immune to eviction at the
  floor) and convergence lag ($2(D_c - 1)$ splits during gate
  tightening). [ADR-M-018](018-hard-budget-guarantee.md) resolves
  this with a **dynamic** soft limit
  $S = H - \max(M, 2(D_c^{\text{live}} - 1))$, recomputed on
  every `adjust_depth_gates()` call. This ensures the headroom
  always covers whichever overshoot source dominates, making the
  raw `budget` a true per-call hard ceiling without requiring a
  split guard.

- **`bounded_eviction` is independent of the soft limit.**
  `bounded_eviction` (ADR-M-015 Q3) controls the _stopping policy_
  of an eviction pass — whether to evict all eligible entries or
  stop once pressure is relieved. The soft limit (ADR-M-018) controls
  the _trigger threshold_ — at what `node_count` tightening and
  eviction begin. These are orthogonal concerns:
  - `bounded_eviction = true` + soft limit: stop evicting when
    `node_count` drops to `S`.
  - `bounded_eviction = false` + soft limit: evict all eligible,
    even if `node_count` goes well below `S`.
  - No budget (`None`): `bounded_eviction` only affects the public
    `check_evictions()` call, which is always unbounded.

  The bounded `stop_at` parameter uses the overshoot above the
  dynamic soft limit (`node_count − S`), not the raw budget. Since
  $S$ is recomputed on every `adjust_depth_gates()` call (ADR-M-018),
  the eviction target adapts to the current gate position.
