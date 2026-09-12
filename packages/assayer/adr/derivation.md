# Derivation · `rec:derivation:pure-decision-transform`

This record fixes the far side of the Core/derivation split. The near side is already decided — what the Core sends across, and that it sends nothing else (`dec:ordering:closed-crossing`). Decided here is what the receiving transform is, what may be added to what it receives, and what it may not reach back into. It realises the derivation choice (`dec:numerics:derivation-purity`).

It is the last record of the set and the only one written after the conformance audit opened. That ordering shows: the decisions below are stated on the specification's authority rather than on the code's (`dec:assayer:spec-authoritative`), and three environments exist so that the distance between the two is carried rather than smoothed over.

**Decision (The derivation is a pure transform and cannot fail)** · `dec:derivation:pure-transform`

The transform reads no Core state, holds none, writes none, consults no clock and no source of randomness, and has no error channel. Every edge its arithmetic can reach has a defined value, so an error channel would carry a failure no caller can receive and every caller must still handle. The specification proves the clauses from the signature rather than asserting them (`pf:landscape:purity`) and registers the result (`inv:guarantee:derivation-purity`).

Purity here is structural rather than behavioural, and that is the whole of the choice. A transform handed the model could be pure, and would be pure for exactly as long as nobody reached; a transform handed a statistic has nothing to reach for. So the input set and the import surface are decided beside the purity rather than after it: neither guards the property, both are what it is made of. What it buys a host is replay (`inv:guarantee:replay`).

**Decision (Channel constants are recomputed on every call)** · `dec:derivation:per-call-constants`

The constants a channel's reward parameters imply are derived on every call and never cached against the policy that produced them, because one of them is not a function of the policy alone: the adversary's benefit from escalation moves with the challenge estimate (`data:channel:challenge-interaction`), and the estimate evolves as labels arrive.

Caching would therefore be a correctness choice wearing the costume of a performance one. A cached differential is right at the instant it is computed and drifts silently after, with nothing on the path able to report that the constant and the estimate have parted company. The genuinely static half is recomputed alongside rather than split out, because two lifetimes in one structure is a bookkeeping obligation bought with a saving no budget here needs (`def:landscape:reward-differentials`).

**Decision (The transform is generic over an ordered action set)** · `dec:derivation:ordered-actions`

A channel declares its own action set, and the declaration may be any subset of the canonical order that preserves that order (`def:channel:actions`). The transform is written against whatever it is handed: costs from a match on the action, differentials from whichever adjacent pairs the declaration contains, and the crossover, bandwidth and placement computations over those pairs without knowing how many there are.

It assumes the declaration is well formed and does not check it. That check is the host's, at policy load rather than per call, and it fails hard rather than degrading: a malformed action set is a caller's defect the caller can fix, which is the side of the partition the package already draws (`dec:degradation:error-partition`). What narrowing the set costs a host is the specification's to say (`cav:channel:action-space-sizing`).

**Decision (The derivation module is the unit the allowlist scopes)** · `dec:derivation:allowlisted-imports`

Purity is guarded by what the derivation module may import, not by a test that watches it behave purely. The module is the scope: every crate-internal root it reaches must stand on an explicit list, and a sibling module added later is prohibited until the list says otherwise.

The choice is between a total guard and a sampling one. A behavioural purity test exercises only the calls somebody thought to write, and passes for every call nobody wrote; an import guard is a statement about the whole module and fails on the first line that would break it. What this record decides is the narrow thing — which module the guard is drawn around. That the boundary is enforced by a check at all is decided elsewhere (`dec:ownership:enforcement`), and the checks are carried once as an asset (`rule:ownership:import-allowlist`).

**Decision (The exploration quantity belongs to this layer)** · `dec:derivation:exploration-layer`

The signal that says whether resolving a decision is worth buying an observation for is computed here and nowhere else. It is a property of where a decision sits against its own boundaries, so it needs the derived output to exist first, which puts it beyond the Core's reach. The Core does not consume it either: no Core surface accepts it, reports it, or ranks by it.

That is a partition rather than an omission. The Core keeps its own learning signal, scored on an uncertainty it already holds (`def:guidance:risk-informative`), and the guidance surface is closed against derivation quantities by decision (`dec:surface:guidance-opacity`). The two together are what the exploration guarantee names (`inv:guarantee:exploration`). Both halves exist in the shipped package: Core guidance supplies the learning signal, and the public landscape fragility utility supplies the decision signal at a caller's posture.

**Rule (What may be added to the crossing)** · `rule:derivation:closed-inputs`

The transform receives three things and the specification fixes which (`sig:landscape:derivation-function`): the Core's risk basis, the host's declared channel policy, and the challenge posterior. This rule governs additions, and it is what a reviewer applies to a proposed field.

A field depending on Core model state — parameters, covariance, the outcome store, identity state, standardisation statistics — is not a convenience but a tightening of the coupling the split exists to prevent, and it is refused unless the sufficiency claim behind the crossing is reopened first. An argument carrying display parameters is refused outright, because it makes the output a function of presentation and the presentation-free invariant cannot then be stated of it at all (`inv:landscape:presentation-free`). Axis predictions are refused on the same reasoning (`def:landscape:outcome-neutrality`).

**Register (The four sizes the corpus gave the crossing)** · `reg:derivation:collapsed-crossing`

Two retiring records stated the size of the crossing five times between them and gave four different answers. The derivation record's opening called it five scalars and a channel policy; its consequences, four scalars and the derived constants; its forward constraint, three scalars plus the Companion's variance. The assessment contract's sufficiency callout named a quadruple, then said in the same environment that the carrier omits the quadruple's second member as recoverable from the other three; its neutrality section named five scalars, one of which the carrier does not hold because it is passed to a different function.

The specification settles this in a way none of the five attempts could: the crossing is counted in arguments, not in scalars (`sig:landscape:derivation-function`). Three arguments cross, and what the transform reads out of the first is the risk basis alone (`schema:risk:basis`). A scalar count is a fact about a struct layout rather than about a boundary, which is why it could drift four ways without contradicting anything either record thought it was saying. The census did not catch this one (`reg:assayer:adr-contradictions-records`).

**Remark (Why the two exploration signals are complements)** · `rem:derivation:exploration-split`

The Core's signal and this layer's look like duplicates and rank the same population differently, which is why the split is a decision rather than an accident of where the code put a function. The Core's asks which stored request the model is least certain about; this layer's asks whether resolving a decision would change what the host does.

They come apart in both directions. A request deep inside a regime can carry large uncertainty and near-zero decision value; a request near a boundary can turn on a small surprise while the model is confident about it. So a host treating them as one signal buys the wrong evidence either way (`def:fragility:definition`).

**Discussion (The rejected enumeration of known action sets)** · `disc:derivation:enumerated-alternative`

The alternative was to enumerate the action sets the corpus knew about and give each its own derivation path. It is the simpler thing to write and it was rejected on what it cannot serve. The specification permits a channel to declare any subset of the canonical order (`def:channel:actions`), so an enumeration is complete only against the declarations somebody anticipated, and a host declaring anything outside the list gets no derivation rather than a worse one.

The rest is the ordinary cost of enumeration: one path per case, each with its own crossover arithmetic and edge handling, each needing the same correction whenever the shared mathematics moves. The generic form has one path over adjacent pairs, so the arithmetic is written once and the declaration is data (`alg:landscape:computation-order`).

**Caveat (The monotonicity guard is definition, not decision)** · `cav:derivation:guard-absorbed`

The retired inventory of numerical constants sent exactly one constant here: the floor a dominated tag's magnitude is set to (`dec:assayer:record-dropped-content`). It is not a decision this record makes. The specification's rewrite has since given the dominated-tag treatment a full home — the three conditions that trigger it, the floor and bandwidth substituted for the tag's own, and why the suppression is presentational with no counterpart in the decision content underneath (`alg:rendering:dominated-treatment`).

So the guard is cited and no figure for it is stated, which is what the disposition's own reasoning asks once another document can answer the question. It is named at all so that a reader who comes here looking for the constant learns where it went rather than reading silence as evidence it was dropped.

**Caveat (The shipped surface now matches the decided split)** · `cav:derivation:shipped-surface`

The former four-gap list has no surviving divergence. The primary call takes the assessment, channel policy and challenge posterior and returns a decision landscape with no display configuration in its arguments (`sig:landscape:derivation-function`); rendering is a separate optional call over that landscape (`dec:landscape:rendering-optional`). The challenge posterior crosses as either the conjugate pseudo-count pair or a moment-matched estimate, preserving the structure its quantile and dominance consumers need (`sig:companion:posterior`). The fourth item was never a gap: the neutral zone is reserved and unread exactly as the specification declares (`def:channel:neutral-zone`).

Three implementation gaps were repaired after the conformance audit measured them, and the neutral-zone item was reclassified as conformance rather than silently dropped. The live surface therefore has the split this record decides: decision content first, optional presentation over it.

**Caveat (The formerly unread tranche is measured)** · `cav:derivation:unmeasured`

The conformance audit no longer leaves this record's module cluster unread (`rep:assayer:code-conformance-campaign`). Re-derivation against the live resonance modules confirms the three-argument decision transform, the separate rendering layer, the two posterior variants and the deliberately inert neutral zone stated above. The record's standing is therefore the same as the rest of the set: its claims have been checked against both the specification and the code.

The old warning still matters as history, not as current qualification. It is why the audit measured the whole cluster rather than treating the record's original gap list as exhaustive, and why the corrected list records the conformant fourth item instead of making it disappear.
