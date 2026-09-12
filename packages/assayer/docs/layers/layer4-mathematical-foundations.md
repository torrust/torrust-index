# Layer 4: Mathematical Foundations · `spec:numerics:model-mathematics`

This document fixes the conceptual choices of the numerics layer. It is one of the five layer outlines that supply the middle term of the projection (`dec:assayer:projection-principle`): the specification fixes the concepts, the layer outlines fix the choices those concepts leave open, and the decision-record set is derived from the choices. It is not a summary of records; the record census found the layer documents had only ever been that (`obs:assayer:adr-layers-not-outlines`).

A *conceptual choice* here is a question the specification poses but does not answer, where the answer binds every implementation of the layer and could have gone another way. The distinction does real work at this layer, because a derivation is not a choice: the census classified the mathematical derivations, their cost accountings, and their step enumerations as material defining what the system is, and directed them to the specification (`data:assayer:adr-disposition-totals`). What remains here is what the numerics *decide* — which identity to compute with, what to do when it fails, and what the failure is allowed to cost.

Each choice below is stated exactly once. Each names, in prose, the record cluster expected to own its decisions (`plan:assayer:decision-record-architecture`). Most of those clusters have since landed, and a choice whose cluster has landed now carries a citation of the landed record's identity decision alongside the naming; a choice whose cluster remains unlanded is still named only. The tracking relation over every landed record is carried once, by the master register (`docs/adrs.md`), and is not repeated here: this outline connects to its records by citing their identity decisions rather than by a tracking table of its own.

## Scope · `sec:numerics:scope`

**Summary (What this layer fixes)** · `summ:numerics:scope`

The numerics layer fixes how the posterior is maintained: the incremental update and its bound, the single factorisation whose refusal is a verdict and the floor that keeps the matrix factorisable, the correction applied when a dimension leaves, the calibration and the blend that turn a posterior into a probability, the purity of the derivation that reads it, and the conjugate model behind the challenge estimate. It operates on the substrate the representation layer fixes (`summ:representation:scope`) at the points in the orderings the operational layer fixes (`summ:operational:scope`).

Nine choices. The census derived seven for this layer (`tab:assayer:adr-disposition-numerics`); this outline splits one and adds one, on the reasoning recorded at each.

**Convention (No result is restated here)** · `conv:numerics:results-cited`

Every theorem, identity, bound, and cost this layer relies on is owned by the specification and cited. None is reproduced, and none is re-derived. Where a choice exists only because a result holds, the choice names the result and stops.

## The conceptual choices · `sec:numerics:choices`

**Decision (Read, decide, mutate — in three separated steps)** · `dec:numerics:incremental-update`

The incremental update is separated into three steps: read the quantities the policy needs, decide the update the policy permits, and only then mutate. The mutation is the specification's leverage-bounded update (`alg:update:sherman-morrison`), whose bound is what makes a single observation unable to dominate the posterior (`prop:update:leverage-bound`), and which is exact per observation rather than approximate (`inv:guarantee:per-observation-exactness`). Time decay and label decay combine into one factor computed once per model per label, not applied twice in sequence. Leverage is bounded by policy *before* the update is applied, never repaired afterwards, and the interaction between that bound and the importance weighting is the specification's (`disc:weighting:leverage-interaction`).

The alternative — mutate then inspect and revert — is the arrangement the separation exists to prevent, because a revert must reconstruct state the mutation destroyed. Expected owner: the *posterior maintenance* record (`dec:posterior:three-step-update`).

**Decision (A factorisation is attempted once, and its refusal is a verdict)** · `dec:numerics:repair-cascade`

A factorisation is attempted plainly and once. Its refusal is not retried under a diagonal shift: the refusal says that the matrix in hand is not positive definite in the working arithmetic, and it is the answer rather than a condition to repair. What a refusal costs depends on where the matrix came from. On the posterior maintenance path it retains the maintained covariance and flags it, which is the numerics-layer face of the structural failure posture (`dec:structural:failure-posture`); on a path restoring a model from a persisted artefact it fails the call, because there the refusal is a defect of whoever wrote the artefact and one that can be acted on. The retired second phase shifted the matrix, factored the shifted one and returned that matrix's inverse, which is an answer about a matrix nobody supplied; what keeps the precision guarantee a statement about the model is a floor on the spectrum, applied where the covariance is being rebuilt from the precision matrix anyway (`inv:guarantee:precision`), and what keeps a rebuilt covariance honest is the drift measured after it rather than the factorisation's own verdict. One shared inversion utility serves every call site; there is no second implementation with its own tolerances, and the utility takes no numerical configuration at all. The specification records that the posterior can become ill-conditioned (`cav:limitation:conditioning`), and refusing is what the system does about a matrix it cannot factor rather than a claim that it will not meet one.

The census found this utility's interface specified incompatibly by the two records that shared it, each certifying its own side as verified (`reg:assayer:adr-contradictions-records`). There is one utility with one interface, owned once, and what a call site decides is the disposition of a refusal rather than the arithmetic that produced it.

Expected owner: the *posterior maintenance* record (`dec:posterior:three-step-update`), which carries the refusal at (`dec:posterior:repair-cascade`), the floor at (`dec:posterior:spectral-floor`) and the adoption test at (`dec:posterior:measured-adoption`).

**Decision (Recomputation fires on a counter whose interval conditioning sets)** · `dec:numerics:recomputation-trigger`

Recomputation is decided per model and fires on an elapsed-update counter whose interval conditioning sets, on growth of the cheap ratio away from the value the last rebuild recorded, or at once on a precision diagonal entry that is not finite or not positive (`alg:gaussian:condition-adaptive-recompute`), reading the estimate the specification defines (`def:monitoring:condition-number`). The conditioning arms are relative to the last rebuild's own record rather than absolute, because the replenishment floor pins the cheap ratio's denominator and a fixed threshold on a quantity the rebuild cannot move is satisfied permanently rather than triggered. Poor outcomes halve the interval and sustained clean operation restores it, so the cadence adapts downward fast and upward slowly; what makes an outcome poor is the drift residual left when the part the prior put there is taken out, never the whole reading, since that part scales with the interval and no rebuild removes it. Divergence between the maintained factorisation and a fresh one is monitored (`alg:gaussian:synchronisation-monitor`) against the measure the specification fixes (`def:monitoring:synchronisation-error`).

The census grouped the trigger with the cascade. It is separated here because the two answer different questions — *what to do when the factorisation is bad* against *when to look* — and because the trigger is the layer's only adaptive policy, where the cascade is a fixed procedure. Expected owner: the *posterior maintenance* record (`dec:posterior:three-step-update`).

**Decision (Marginalisation corrects by half-solve, and may decline)** · `dec:numerics:marginalisation-correction`

When a dimension leaves, the correction is computed through a half-solve Gram matrix rather than a full solve, over the Schur complement the specification defines (`def:gaussian:schur-complement`) and under the positive-definiteness it requires (`req:gaussian:positive-definiteness`), realising the marginalisation result (`thm:gaussian:marginalisation`). The operation is infallible: a conditioning guard skips the correction and falls back to the uncorrected block rather than attempting a computation it expects to fail. The covariance is always refactored afterwards and never taken from the extracted block. A full solve is retained as a debug-only cross-check oracle, so the cheap path has something to be wrong against. What the operation must preserve exactly is structural (`inv:guarantee:structural-exactness`).

Expected owner: the *posterior maintenance* record (`dec:posterior:three-step-update`).

**Decision (The derivation is pure, infallible, and closed over its inputs)** · `dec:numerics:derivation-purity`

The derivation is a pure function: no Core state, no interior mutability, no clock, and no failure mode (`inv:guarantee:derivation-purity`), which the specification proves rather than asserts (`pf:landscape:purity`). Its output carries no presentation (`inv:landscape:presentation-free`), so choosing an action from it (`alg:landscape:optimal-action`) and rendering it are separate concerns and separately optional. The function is generic over an ordered action set rather than fixed to one. Channel constants are recomputed per call because one of them depends on evolving evidence, so caching them would silently freeze that dependence. Its imports are restricted by an allowlist enforced as a test, which is the boundary enforcement of (`dec:structural:boundary-enforcement`) applied inward. Exploration quantities are derivation-layer quantities that the Core neither computes nor consumes.

Expected owner: the *derivation* record (`dec:derivation:pure-transform`). The Core-side half of the split is owned by the *runtime paths* record (`dec:ordering:assessment-function`), which realises the split named at (`dec:operational:core-derivation-split`); the derivation cluster waited on the Part IV ruling and was scoped once that ruling landed.

**Decision (Calibration is a factored per-regime search with no side effects)** · `dec:numerics:calibration`

Fitting the calibration parameter is a factored per-regime search on a transformed parameter (`alg:platt:fitting`) against the specification's objective (`def:platt:objective`), under a minimum-sample floor below which no refit happens (`req:platt:minimum-samples`). The refit is a pure function of the buffer and the configuration: it mutates nothing and returns a parameter, so a refit can be recomputed, tested, and discarded. A large calibration shift conservatively resets every drift accumulator (`alg:platt:drift-integration`), on the reasoning that a shift large enough to matter invalidates the baseline the accumulators measure against. Outputs before the first refit are uncalibrated and the specification says so (`cav:limitation:pre-calibration`).

Expected owner: the *calibration and blending* record (`dec:calibration:per-regime-search`).

**Decision (The blend computes by index and clamps every form)** · `dec:numerics:blend`

The blend is restricted to the anchor's subspace (`def:risk:subspace-blend`) and stays there (`inv:guarantee:blend-subspace`). The projected quadratic form is computed by index without allocating a projection, so the restriction costs indexing rather than memory. The blend weight uses raw forms rather than time-corrected ones, because the correction cancels in the ratio and applying it would cost two multiplications to reach the same number. Every quadratic form is clamped non-negative before use, so a form driven slightly negative by rounding degrades into zero rather than into a negative variance. The resulting variance takes the form the specification states (`prop:risk:blend-variance`) and guarantees (`inv:guarantee:blend-variance`).

Expected owner: the *calibration and blending* record (`dec:calibration:per-regime-search`).

**Decision (The regime transition function is defined once and shared)** · `dec:numerics:shared-transition`

The regime-transition function and its constant are defined in exactly one place and used by both the calibration and the blend. They are not two functions that happen to agree, and they are not one function copied.

This choice exists because the census found the alternative's cost measured: the function was written out in full in two records, one of which recorded as a consequence that it was defined once and warned that inconsistency between two copies would produce artefacts — the consequence was right and both record bodies transcribed it anyway (`reg:assayer:adr-contradictions-records`). The sharing is promoted to a choice of its own here so that the outline states it where a derived record must carry it, rather than leaving it as a remark inside one of two records. The regime structure it produces is the specification's (`disc:platt:regime-transition`) and (`def:platt:regimes`).

Expected owner: the *calibration and blending* record (`dec:calibration:per-regime-search`).

**Decision (The challenge estimate is conjugate, decaying toward its prior)** · `dec:numerics:challenge-model`

The challenge estimate is a conjugate model updated in closed form (`alg:companion:update`) and guaranteed conjugate rather than approximated (`inv:guarantee:conjugacy`), with lazy decay toward the prior (`alg:companion:decay`) applied by blending rather than by scaling the counts — a form the specification chose deliberately (`dec:companion:decay-form`). A host override does not pause accumulation underneath it, so clearing an override recovers the state that evolved while it stood (`alg:companion:override`). Injected evidence is indistinguishable from observed evidence and is capped (`alg:companion:injection`), so a host cannot use injection to overwhelm observation. Convergence is bounded (`bound:companion:convergence`), and the specification records that it converges only as labels arrive (`cav:limitation:challenge-convergence`) and that contributing labels are thin in realistic deployments (`cav:limitation:challenge-thin`).

Expected owner: the Companion cluster, landed as (`dec:challenge:conjugate-model`); the unwired gap between the model and its caller is stated at (`cav:numerics:challenge-unwired`).

## Unsettled at this layer · `sec:numerics:unsettled`

**Caveat (The challenge mathematics shipped before anything consumed it)** · `cav:numerics:challenge-unwired`

Every decision of (`dec:numerics:challenge-model`) is implemented and exported, and nothing on any runtime path calls it: the constructors appear only in tests (`obs:assayer:adr-challenge-unwired`). The model is live — it binds a shipped library type — while the contract that would give it a caller is not (`dec:contracts:companion-boundary`). Whichever way that is ruled, the derived record owes its reader this fact rather than an arrangement that reads as complete.

**Entry (Numerics deferrals, resolved)** · `entry:numerics:diagnostics-open`

Two registered deferrals hung off this layer's choices. The marginalisation ledger folds every model's diagnostics and exposes its lifetime aggregate through health; the pushed completion event and lifecycle result now carry the same removal's event-local aggregate, retiring (`entry:assayer:defer-marginalise-event`). Synchronisation error now reaches health for every live model and in aggregate, alongside the part of each reading the replenishment clamp put there and the residual left when that part is taken out, and a clean rebuild whose residual is high halves the effective interval down to the adaptive floor, retiring (`entry:assayer:defer-sync-error-visibility`).

The correction skip counter that makes (`dec:numerics:marginalisation-correction`) observable when it declines is complete and retired at (`entry:assayer:defer-schur-skip-counter`). Each item is held by the record that owns the decision it qualifies; neither remains in the open register.
