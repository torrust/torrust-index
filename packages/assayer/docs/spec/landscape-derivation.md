## Part (The Decision Landscape) · `part:spec:decision-landscape`

Part III ended at the risk basis. Part IV specifies what a host does with one: a stateless transform from a risk basis, a declared channel policy and a challenge posterior to a posture-indexed decision landscape — the postures at which the optimal action changes, each with its two-source uncertainty and its dominance probability — together with the exploration signal that says whether resolving one of those boundaries is worth an observation. Nothing in this Part learns. The transform holds no state, reads none, and returns the same landscape for the same three inputs however many times it is called.

The Part specifies what the derivation is: its inputs, its presentation-free output, the exploration quantity read from that output, and the optional rendering layered over it.

### Chapter (The Derivation Function Interface) · `chap:spec:derivation-interface`

The chapter fixes Part IV's shape. It states the derivation's signature and the three inputs that cross into it, the landscape those inputs produce, the purity that makes the transform replayable, the two utilities that read a landscape at a posture, and the recorded decision that keeps presentation outside all of it. The three inputs are specified separately because they stand at different distances from the Core: the risk basis is the Core's own output and crosses unchanged, the channel policy is the host's declaration and is developed in the next chapter, and the challenge posterior arrives from a component the Core does not know about — which is why the boundary it crosses is specified here, where the crossing happens, rather than with the component that maintains it.

**Signature (The derivation function)** · `sig:landscape:derivation-function`

The derivation is a function of exactly three arguments — a risk assessment, a channel policy, and a challenge posterior — returning one decision landscape.

| Argument | Supplied by | Carrying |
| --- | --- | --- |
| Risk assessment | The Core | The risk basis, the axis predictions, the per-Sentinel alarm summaries, and a health snapshot |
| Channel policy | The host | The named action set, the reward parameters, the sensitivity exponents, and the neutral zone |
| Challenge posterior | The Companion Tracker | The challenge-effectiveness distribution, conjugate or moment-matched |

The derivation reads the risk basis (`schema:risk:basis`) and ignores the rest of the assessment; the remaining fields travel in the argument because a host holds one assessment and derives from it, not because the transform consumes them. It is deterministic, stateless and closed form: no iteration, no search, no convergence criterion, and no dependence on the Core's parameter state or on the Companion's pseudo-counts beyond the posterior it is handed. The same three inputs produce the same landscape on every machine and at every time (`pf:landscape:purity`).

Nothing else crosses. A fourth argument carrying display parameters would make the returned object a function of presentation, which is exactly what the landscape is specified not to be (`inv:landscape:presentation-free`). Rendering is a separate call over the resulting decision landscape.

**Signature (The challenge posterior)** · `sig:companion:posterior`

The challenge posterior is the distribution over $q_c$, the probability that a challenge correctly identifies an adverse source. It crosses the derivation boundary in one of two variants.

| Variant | Carrying | Supports |
| --- | --- | --- |
| Conjugate | The Beta pseudo-count pair $(\alpha, \beta)$ | Point estimate, variance, and exact quantiles |
| Moment-matched | The point estimate $\hat{q}_c$ and its variance $\sigma^2_{\hat{q}_c}$ | Point estimate and variance; quantiles only through a moment-matched Beta |

Both variants yield $\hat{q}_c$ and $\sigma^2_{\hat{q}_c}$, which is all the crossover locations and their variances require. The conjugate variant carries strictly more: its quantile function is what credible intervals push through (`def:landscape:credible-intervals`) and what makes the dominance probability exact rather than approximated (`thm:landscape:dominance`). The moment-matched variant exists for hosts whose challenge estimate comes from somewhere else — an external analytics pipeline, a fixed operational override — and it degrades those two consumers to a moment-matched surrogate rather than removing them.

The variant distinction is therefore not a convenience: it is the boundary at which the Companion's conjugate structure either survives into the derivation or does not. A host with no Companion at all supplies the uniform prior (`def:companion:prior`), whose effect is conservative — wide intervals and non-trivial dominance probabilities until evidence accumulates — rather than absent. The posterior itself is maintained and decayed elsewhere (`alg:companion:inference`); what this signature fixes is only what arrives here. The conjugate pair survives in the landscape, while an external mean-and-variance pair enters through the moment-matched variant.

**Signature (The decision landscape)** · `sig:landscape:output`

The landscape is the complete decision content of one assessment on one channel: where the optimal action changes, how certain each boundary is, which evidence source that uncertainty comes from, and how likely each action is to be dominated outright.

| Group | Field | Carrying |
| --- | --- | --- |
| Evidence | Shared term $u$ | The risk-driven crossover location, common to every transition |
| Evidence | Shared uncertainty $\sigma_u$ | The single scalar carrying all risk-driven uncertainty |
| Evidence | Challenge posterior | The posterior as it crossed the boundary |
| Transitions | Crossover vector | One entry per adjacent pair of declared actions |
| Regimes | Regime vector | One entry per declared action |

The crossover vector has fixed shape: exactly $J-1$ entries for $J$ declared actions, whatever the evidence says about dominance. Shape stability is what makes a landscape safe to store, to replay, and to difference across a policy comparison (`ex:architecture:compositional-dividends`) — a Companion update that carries $\hat{q}_c$ across a dominance threshold changes values and never structure. Each crossover reports its transition pair, its location, its offset at the point estimate, its sensitivity to the challenge estimate, its variance and its credible interval (`def:landscape:action-crossover`). Each regime reports its action, its logit width where the action is interior, the width's variance, and its domination probability.

Absent from the landscape: no tags, no posture, no classification probabilities, no display constants. Those belong to the rendering layer (`dec:landscape:rendering-optional`). The presentation-free landscape carries the declared exponent sum alongside those groups so the optimal-action utility can reconstruct the cost curves; the rendered tag profile is a separate object.

**Invariant (The landscape is presentation-free)** · `inv:landscape:presentation-free`

Every field of the landscape is a function of the risk basis, the declared policy and the challenge posterior alone. No kernel bandwidth, compression factor, display floor, spectrum parameter or other presentation constant reaches any of them.

The invariant is this chapter's own claim rather than a general property recorded elsewhere, because it is a claim about this signature: it holds exactly when the derivation's argument list is the three of (`sig:landscape:derivation-function`) and fails the moment a fourth argument carrying display parameters is admitted, whether or not any field happens to read it. Two of the composition dividends rest on it directly (`ex:architecture:compositional-dividends`). An external service deriving under a different policy isolates the cost structure exactly only if the compared outputs share no display parameter; a replayed assessment reproduces its landscape from three stored inputs only if three inputs are all there were.

The landscape call takes the three specified inputs, while the separate rendering call is the only one that takes display configuration (`dec:landscape:rendering-optional`).

**Proof (The purity of the derivation)** · `pf:landscape:purity`

The derivation reads no global state, writes no state, allocates no persistent resource, has no side effect, and returns the same landscape for the same three inputs.

The five clauses are demonstrated by construction rather than by testing, because the signature admits no counterexample. The argument list carries no mutable reference, so nothing reachable through it can be written. It carries no handle to the Core, the Companion, a registry, a clock or a random source, so there is no external state to read and no source of nondeterminism to read it from. The return value is the only channel out, so there is no side effect to observe. Determinism then follows from the first four: a function whose output depends only on its arguments, computed by closed-form arithmetic over them, returns one value per argument triple.

This is what makes replay meaningful (`inv:guarantee:replay`): a host that stored the assessment, the policy and the posterior stored the complete determinant of the landscape, and needs no model state to reproduce it (`inv:guarantee:derivation-purity`).

**Signature (The landscape utilities)** · `sig:landscape:utilities`

Two utilities read a landscape at a posture. Neither requires model state and neither is part of the derivation itself.

| Utility | Given | Returning |
| --- | --- | --- |
| Optimal action | A landscape and a posture | The action whose regime contains the posture's logit |
| Fragility | A landscape and a posture | The decision fragility at that posture (`def:fragility:definition`) |

The split is deliberate. The landscape is computed once per assessment per channel and describes every posture at once; the utilities are read repeatedly, at whatever postures the host cares about, without recomputing anything. A host running one assessment against several postures — a staged response, a comparison across operating points — pays for the landscape once and for each reading almost nothing.

Both utilities are total: every posture in the open unit interval has an optimal action and a fragility, including postures inside a dominated action's inverted region, where the naive reading fails and the specified one does not (`alg:landscape:optimal-action`). Both read the landscape without rendering it.

**Algorithm (The optimal action)** · `alg:landscape:optimal-action`

At posture $\pi$ the optimal action is the one minimising expected cost at $\ell(\pi)$. It is computed from the upper envelope of the per-action cost curves, not by locating the bracketing crossover entries.

1. **Reconstruct** each declared action's expected-cost curve as a function of the logit, from the shared term, the per-transition offsets and the posture-sensitivity functions (`def:landscape:posture-sensitivity`).
2. **Evaluate** each curve at $\ell(\pi)$.
3. **Select** the action of least expected cost, ties broken toward the more permissive action so that the answer is left-continuous in the posture.

The bracketing shortcut is wrong and the reason is structural. When an interior action is dominated at the point estimate its two bounding crossovers are inverted — the lower one lies above the upper — so they bracket nothing, and a reader walking the crossover vector in order would return the dominated action for an interval where it is never optimal. The envelope has no such failure mode: a dominated action simply never attains the minimum, and no special case is needed to exclude it (`thm:landscape:dominance`). For the four declared actions of a typical channel two passes suffice, so correctness here costs nothing worth measuring.

**Decision (Rendering is a layer over the landscape)** · `dec:landscape:rendering-optional`

The resonance rendering is a function of a landscape and a risk basis, taken by hosts that want a smooth visual spectrum over classifications and actions. It is optional, and the landscape is complete without it.

The decision is recorded because the alternative is available and was rejected. A derivation that rendered directly would spare hosts one call and one type, and it would cost the two properties the rest of this chapter is built on. Rendering interposes display constants — kernel bandwidths, compression factors, a magnitude floor — between the evidence and the output (`tab:config:rendering`), so a rendered output is a function of presentation and the presentation-free invariant cannot be stated of it (`inv:landscape:presentation-free`). And rendered tag probabilities are normalised kernel evaluations, not posterior probabilities of any event, so a host reading decision semantics off them is reading a display artefact. All decision semantics are therefore defined on the landscape (`sig:landscape:output`) and all exploration semantics on fragility (`def:fragility:definition`); the rendering carries no decision content the landscape does not already hold (`sig:rendering:contract`).

The primary derivation produces the decision landscape without display configuration, and the optional rendering call takes that landscape, the risk basis and the display configuration.
