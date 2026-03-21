# ADR-M-003: Violation Tracking Strategy

**Status:** Decided (revised 2026-03-07)  
**Date:** 2026-02-24  
**Relates to:** [ADR-M-002](002-vtree-node-enum.md) (V-Tree node
representation — `VNode<V>`, `PackedChildren<V>`)  
**Spec:** §IDEA M-11 (V-Tree Rebalancing), §IDEA M-11.8 (rebalance loop),
§IDEA M-11.11 (side-effect violations), §IDEA M-11.12 (violation source catalogue),
§IDEA M-10.1 (split preprocessing)  
**API:** [§API M-5.2](../docs/api.md#52-gvgraphc-v-n--the-negative)
(observe pipeline — violation check step),
[§API M-6](../docs/api.md#6-surface-3--emulsion)
(`rebalance` module)  
**Surface:** 3 (Emulsion)

## Context

The V-Tree's max-uncle constraint (V-I3) may be violated when an
entry's intensity grows past its max uncle. The rebalance loop
(§IDEA M-11.8) must find and resolve all violations. Two strategies:

- **Eager:** Check for violations at the point of mutation, push
  violators onto a work queue immediately.
- **Lazy:** Scan the V-Tree during `rebalance()` to find violations.

## Decision

**Eager tracking with a `Vec<VNodeId>` work queue.**

### Rationale

Every intensity mutation already touches the node and its parent
during sum propagation. The uncle check reads the grandparent's
`PackedChildren` — which is in the same cache line, already loaded.
The violation check is **free in cache terms**.

Pushing a `VNodeId` onto a `Vec` is a single write. Scanning the
entire V-Tree to find the same violations would re-read nodes
already in hand.

> Write the check where the mutation happens. Let the compiler
> elide what it can.

### Design

```rust
struct GvGraph<C, V, const N: u32> {
    // ... arenas, roots, config ...

    /// Work queue — scoped to one mutation batch (observe, evict, decay).
    /// Built during propagation, drained by rebalance(), empty on return.
    violations: Vec<VNodeId>,
}
```

### Push sites

There are ten violation sources (§IDEA M-11.12), numbered 1–10, served
by dedicated push functions. Sources 1–5 were identified during
initial development; sources 6–9 were discovered while implementing
eviction when `debug_assert!` full-tree audits caught residual
violations; source 10 was added when g-contraction during promotion
created uncovered grandchild violations. Source 5b is a special
sub-case of source 5 — post-split violations under non-standard
configurations — not a separate source.

The ten sources arise in three contexts, served by eight push
functions (§IDEA M-11.11):

| Context      | Sources     | Push functions                                                                                                              | Cost   |
| ------------ | ----------- | --------------------------------------------------------------------------------------------------------------------------- | ------ |
| Observation  | 1–2         | Ancestor walk (inline in `observe()`)                                                                                       | O(h_V) |
| Rebalancing  | 3–5, 5b, 10 | `push_side_effect_violations`, `push_promoted_violations`, `push_contraction_child_violations`, `push_source_10_violations` | O(1)   |
| Leaf removal | 6–9         | `push_leaf_removal_violations`, `push_collapse_violations`, `push_remaining_sibling_violations`, `push_cousin_violations`   | O(h_V) |

Source 5 (split preprocessing, §IDEA M-10.1) reuses the same
`push_side_effect_violations` and `push_promoted_violations` calls
as sources 3–4. `push_contraction_child_violations` is
resolve-dispatcher bookkeeping (§IDEA M-11.9) — a child-level check that
skips the target node `c` to avoid duplicate work.

---

#### Sources 1–2. Ancestor walk after intensity propagation (§IDEA M-11.11.2)

```rust
let mut check_id = Some(entry_id);
while let Some(id) = check_id {
    if is_violated(id) { violations.push(id); }
    check_id = vnodes.get(id.index()).parent;
}
```

**Why the full walk:** `propagate_v_sums` increases the intensity
of every structural ancestor on the path to the root. A structural
node at depth 4 might now exceed _its_ uncle at depth 3 — a
violation that has nothing to do with the entry. Only checking the
entry (the original design) missed these ancestor violations.

Cost: O(V-depth), matching `propagate_v_sums` itself.

---

#### Source 3. Grandchild side effects after restructuring (`push_side_effect_violations`) (§IDEA M-11.11.1)

After contraction at `p` creating merged node `m`:

- Grandchildren of `p` (at most 6): uncle shield may have weakened.
- Grandchildren of `m` (at most 6): uncle context changed — their
  uncle is now their former sibling's intensity, not the
  pre-contraction uncle.

After promotion at `g`:

- Grandchildren of `g` (at most 9).

---

#### Source 4. Child-level side effects after restructuring (`push_promoted_violations`) (§IDEA M-11.11.1)

After promotion or contraction changes `p`'s children, the _direct
children_ of `p` have new uncles at the grandparent level. A child
that was safe before the restructuring might now be violated against
its new uncle context.

**Why distinct from source 3:** Grandchild checks (source 3) look
two levels below the restructured node. Child checks look one level
below. After contraction makes `p` a 2-node `[isolate, merged]`,
either child might be violated against its uncle at `p`'s parent.
The grandchild check misses this because it examines nodes _below_
the children, not _at_ them.

Called after: every `standard_promote`, `skip_promote`, and split
preprocessing contraction.

---

#### Source 5. Split preprocessing (§IDEA M-10.1)

Split preprocessing contracts the receiver's parent before
subdividing. This contraction creates the same effects as sources
3–4: grandchildren face weakened uncles and children face new uncle
contexts. The same `push_side_effect_violations` and
`push_promoted_violations` functions are called on the preprocessed
parent.

---

#### Source 5b. Post-split violations (§IDEA M-10.1)

After a catalytic or bootstrap split, the new entries or the
structural node `s` may exceed their uncles when P2 (minimum
newcomer) or P3 (idempotent addition) does not hold. Under
standard configurations (P2 + P3) catalytic splits are
violation-free (§IDEA M-10.4), but non-standard configurations can
produce violations. `push_side_effect_violations` and
`push_promoted_violations` are called at `p`; a conditional
ancestor walk fires when `ν ⊕ ν ≠ ν`.

Source 5b reuses the same push functions as sources 3–4/5; no
dedicated push function is needed.

---

#### Resolve bookkeeping: `push_contraction_child_violations` (§IDEA M-11.11.1)

During `resolve(c)`, Phase 1 contracts parent `p`, changing the
uncle context for all of `p`'s children. But `c` itself is about to
be handled by Phase 2 — re-enqueuing it would cause duplicate work.
`push_contraction_child_violations(p, skip = c)` checks `p`'other
children without re-enqueuing `c`.

This is not a distinct violation source (it covers the same class of
violations as source 4) but an implementation detail of the resolve
dispatcher (§IDEA M-11.9) that avoids redundant queue entries.

---

#### Escalation after standard promote (`escalate_after_promote`) (§IDEA M-11.10)

After `standard_promote` makes `p` a 3-node `{c₁, c₂, sib}`:

1. Find heaviest child `h`.
2. Check: is `h` violated at grandparent (direct), OR is any child
   of `h` violated at `p` (indirect)?
3. If neither → return (no cycle risk).
4. Contract `p` (isolating `h`); push side-effect violations.
5. Re-check the trigger condition; if contraction resolved it →
   return.
6. Ensure grandparent `g` is a 2-node (contract if 3-node);
   re-check again.
7. Skip-promote `h` to `g` — breaking the cycle.

**Why needed:** Without escalation, the following infinite cycle
occurs:

```
standard_promote(c) → p is 3-node {c₁, c₂, sib}
c₁ violated → resolve(c₁): contract p → 2-node {c₁, merged}
c₁ still violated → standard_promote(c₁) → p is 3-node again → ∞
```

Even if `c₁` is not directly violated, a _child of c₁_ may be
violated against its new uncle (the indirect variant). Resolving
that child would contract `p`, then standard_promote recreates the
3-node.

Escalation detects this before it loops and takes a different
path — skip_promote — that advances `h` one level higher instead
of oscillating.

---

#### Source 6. Leaf-removal ancestor walk (`push_leaf_removal_violations`) (§IDEA M-11.11.3)

Removing a V-entry calls `propagate_v_sums_from`, which decreases
the intensity of every structural ancestor from the change point to
the root. At each ancestor level, the decreased node may be a weaker
uncle to its siblings' children. Walk from the structural change
point to the root, checking siblings' children at each level.

Cost: O(h_V), matching `propagate_v_sums_from`.

---

#### Source 7. Collapse: promoted node's children (`push_collapse_violations`) (§IDEA M-11.11.3)

When a 2-node parent collapses, the surviving sibling (`sole`) is
re-parented to the grandparent. Children of `sole` now see entirely
different uncles — the grandparent's other children, rather than the
removed entry.

**Why distinct from source 6:** The ancestor walk walks _upward_
from the change point, checking siblings' children. But `sole` is
now a _child_ of the grandparent, not a sibling — the walk doesn't
examine its children.

---

#### Source 8. 3→2 transition: remaining siblings' children (`push_remaining_sibling_violations`) (§IDEA M-11.11.3)

When an entry is removed from a 3-node parent (making it a 2-node),
the remaining siblings' children lose the removed entry as a
potential uncle. If the removed entry was the heaviest uncle,
`max_uncle` decreases, potentially violating the remaining
siblings' children.

---

#### Source 9. Collapse: cousins' children (`push_cousin_violations`) (§IDEA M-11.11.3)

When a 2-node collapses, the grandparent's other children
("cousins") see `sole` replace the collapsed parent as uncle. Since
`sole.int < parent.int` (the removed entry's intensity is gone), the
shield weakened.

---

#### Source 10. g-contraction + promotion: intermediate-level grandchildren (`push_source_10_violations`) (§IDEA M-11.12)

When `resolve(c)` or `escalate_after_promote` contracts the
grandparent `g` before a skip/legacy promote, the contraction
creates a merged intermediate node. Promotion then disperses the
uncle aggregate below this intermediate structural node.
Grandchildren of the merged node face a new uncle context —
their uncle is now the importance of the other merged child rather
than the pre-contraction aggregate.

`push_source_10_violations(merged_g)` checks grandchildren of the
merged intermediate node — the same pattern as source 3, but at
the grandparent level rather than the parent level. Called from
the resolve dispatcher (§IDEA M-11.9) and escalation (§IDEA M-11.10)
after g-contraction.

---

### Rebalance drains the queue

```rust
fn rebalance(&mut self) {
    while let Some(c) = self.violations.pop() {
        if !self.vnodes.is_occupied(c.index()) { continue; }
        if !self.is_violated(c) { continue; }
        self.resolve(c);
        // resolve() may push new side-effect violations
    }
}
```

The guard checks handle three cases:

- Node was destroyed by a prior resolution (stale handle).
- Violation was resolved as a side effect of another resolution.
- Duplicate entries in the queue (harmless, skipped).

A `debug_assert!` full-tree audit runs after the loop in debug
builds, confirming no residual violations were missed by the
queue-based approach. A pre-resolve and post-resolve audit
(`debug_audit_violation_queue`) also runs at each iteration in debug
builds to catch push-site omissions early.

The implementation includes a safety-net iteration limit proportional
to arena size (`20 × node_count`, floor 10 000). In debug builds
the limit panics; in release builds it breaks out with an error log.

`rebalance()` returns `Vec<GNodeId>` — the G-nodes created by
legacy promotions (§IDEA M-11.6) during the drain, so the caller can handle
plateau and depth-control bookkeeping.

### Decay uses full-tree scan

Decay modifies intensities across many nodes simultaneously
(per-depth attenuation factors). Rather than tracking individual
violations during the bulk mutation, `decay()` calls
`find_violated_nodes` — an O(V) scan — to seed the violations
queue, then drains it through the same `rebalance()` loop. This
is correct because decay is a batch operation, not a hot-path
per-observation step.

### Testing: `ViolationSources` configuration

A `ViolationSources` struct allows selectively enabling/disabling
individual sources for testing. Unit tests prove each source is
necessary by disabling it and demonstrating that `debug_assert!`
full-tree audits catch the resulting residual violations.

The struct covers sources 3, 4, 6, 7, 8, 9, 10 — the seven
sources that have dedicated push functions in the `rebalance`
module. Sources 1–2 (ancestor walk) are inline in `observe()` and
always active. Source 5 (split preprocessing) and source 5b
(post-split) reuse the same push functions as sources 3–4.

```rust
pub struct ViolationSources {
    pub source_3_contraction_grandchildren: bool,
    pub source_4_promotion_children: bool,
    pub source_6_leaf_removal_ancestors: bool,
    pub source_7_collapse_children: bool,
    pub source_8_three_to_two_siblings: bool,
    pub source_9_collapse_cousins: bool,
    pub source_10_g_contraction_promotion: bool,
}
```

## Consequences

- Zero-cost violation detection: uncle check reads already-cached
  `PackedChildren` data.
- Bounded queue growth: each restructuring pushes O(1) candidates;
  the ancestor walk pushes O(h_V) per observation; the
  leaf-removal walk pushes O(h_V) per eviction.
- No `HashSet` needed: duplicates are harmless (guard skips them).
- Queue is always empty when a mutation batch returns — no leaked
  state across `observe()`, `check_evictions()`, or `decay()` calls.
- No full-tree scan required in release builds for observe/evict;
  debug-only audit catches any push-site omissions during
  development. Decay uses the full-tree scan by design.
- Escalation (§IDEA M-11.10) prevents infinite cycles without limiting the
  rebalance loop's ability to make forward progress.
- Ten violation sources (§IDEA M-11.12) cover three contexts:
  observation (sources 1–2), rebalancing (sources 3–5, 10),
  and leaf removal (sources 6–9). Source 5b (post-split under
  non-standard configurations) is a special sub-case of source 5.
