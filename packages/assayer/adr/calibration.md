# Calibration · `rec:calibration:factored-refitting`

This record owns three things that the corpus kept apart and the code does not: fitting the calibration, computing the blend, and the transition function they both use. It realises the calibration choice (`dec:numerics:calibration`), the blend choice (`dec:numerics:blend`), the sharing choice (`dec:numerics:shared-transition`), and the half of the convergence choice that fixes what triggers a refit (`dec:operational:convergence-diagnostic`).

The sharing is a decision here rather than a remark inside one of two records, because the census measured what the alternative costs: one function written out in full in two places, one of which recorded as a consequence that it was defined only once.

**Decision (Fitting is a factored per-regime search)** · `dec:calibration:per-regime-search`

The calibration parameter is fitted by a factored search per regime, on a transformed parameter rather than on the parameter itself, against the objective the specification states (`def:platt:objective`) by the algorithm it fixes (`alg:platt:fitting`).

Two properties of the arrangement are decided rather than inherited. The search is per regime, so a regime with its own score-to-probability relationship gets its own parameter instead of a compromise across all of them. And it is floored: below a minimum sample count no refit happens at all (`req:platt:minimum-samples`), because a parameter fitted from too little evidence is not a weaker estimate but a confident wrong one, and it would propagate into every output until the next refit.

**Decision (The refit is a pure function)** · `dec:calibration:pure-refit`

A refit takes the buffer and the configuration and returns a parameter. It mutates nothing: not the buffer it read, not the trackers that will consume its result, not the model. Installing the result is a separate act performed by the caller (`src/risk/calibration.rs`).

Purity is what makes the fit testable and re-runnable. A refit can be computed against a captured buffer, compared with the installed parameter, and discarded without consequence — which is the property a fit needs most, because it is the one computation in the package whose correctness cannot be checked by inspecting its output alone. The buffer's own shape and bound are the specification's (`constr:platt:buffer`).

**Decision (A large calibration shift resets every drift accumulator)** · `dec:calibration:drift-reset`

When a refit moves the calibration far enough, every drift accumulator is reset rather than adjusted, integrating with the drift mechanism the specification defines (`alg:platt:drift-integration`).

The reset is deliberately conservative and deliberately total. A drift accumulator measures departure from a baseline, and a large calibration shift is precisely the event that invalidates the baseline: continuing to accumulate against it would report drift that is an artefact of the refit rather than a change in the traffic. Resetting all of them rather than the ones judged affected avoids a judgment the package has no evidence to make.

**Decision (No movement bound on the calibration parameter ships)** · `dec:calibration:host-specified-movement-bound`

A host that wants an operational bound on how far the calibration parameter may move between successive refits specifies that bound itself. The package ships no default for it and names no threshold unconditionally. This takes the sensitivity study's recommendation and sharpens it: the only default anywhere is the tolerance the test harness pins, and that figure is a fixture oracle and nothing more.

The refusal is evidential rather than cautious. The two authored stimuli — a small proportional move admitted, a large one rejected — pin an interval and not a point, and every candidate inside that interval carries a per-comparison false-alarm frequency that swings by orders of magnitude across the operating surface the study measured: buffer occupancy, refit cadence, and class balance each move the distribution, all of them within configuration-shaped values. A named constant would therefore promise a discrimination the surface does not deliver, and would promise it in the one voice a host cannot argue with, which is a shipped default. A deployment-grade bound needs two inputs the corpus does not have — a target false-alarm probability and a sampling model — and until a host supplies them, absence is the decision rather than the gap.

Two neighbouring figures do ship and neither is this one, which is the whole reason this decision is written where a reader meets them. The threshold at which a refit resets the drift accumulators (`dec:calibration:drift-reset`) is a production mechanism with a specified value (`alg:platt:drift-integration`), and the movement at which the calibration is called settled (`dec:health:calibration-settled-cuts`) is a third, tighter, and about convergence reporting. All three measure a movement of the same parameter and answer different questions, so the corpus keeps them apart deliberately; reading the harness's scenario tolerance (`tab:assayer:harness-scenario-tolerances`) as any of the others is the confusion this decision refuses, and the reason the refusal is a decision rather than an omission.

**Decision (The regime transition function is defined once and shared)** · `dec:calibration:shared-transition`

The regime-transition function and the constant that shapes it are defined in exactly one place and used by both the fit and the blend (`src/numerics.rs`). They are not two functions that agree, and they are not one function copied.

This is stated as a decision because the sharing is load-bearing and invisible. The fit uses the function to weight evidence by regime; the blend uses it to place a model on the same scale. The regime structure it produces is the specification's (`def:platt:regimes`), as is the discussion of the transition itself (`disc:platt:regime-transition`). Nothing in either call site reveals that the other exists, so the property that keeps them consistent is exactly the kind that decays silently — which is why the code carries a test asserting the sharing rather than leaving it to be noticed.

**Decision (The projected quadratic form is computed by index)** · `dec:calibration:indexed-form`

The blend is restricted to the anchor's subspace (`def:risk:subspace-blend`), and the quadratic form over that subspace is computed by indexing into the full covariance rather than by materialising a projection (`src/risk/blend.rs`).

The restriction therefore costs indexing and not memory. Materialising the projection would allocate a matrix per blend on the assessment path, which is the path with a cost bound, to hold values already present in the one being read from. The covariance it indexes into is the one the posterior record maintains and always refactors rather than extracts (`dec:posterior:always-refactor`). What the restriction buys is stated by the specification as a guarantee rather than argued here (`inv:guarantee:blend-subspace`).

**Decision (The blend weight uses raw forms)** · `dec:calibration:raw-weight-forms`

The weight is computed from raw quadratic forms rather than time-corrected ones. The correction is not omitted as an approximation: it appears in both the numerator and the denominator of the ratio the weight is built from, and cancels exactly.

Applying it anyway would cost two multiplications to arrive at the same number, and would leave a reader of the code unable to tell whether the correction was load-bearing. Stating that it cancels is worth more than performing it, because the next person to add a term to that ratio needs to know which of its factors survive. The variance the blend produces takes the form the specification states (`prop:risk:blend-variance`).

**Decision (Every quadratic form is clamped non-negative)** · `dec:calibration:clamped-forms`

Every quadratic form entering the blend is clamped at zero before use (`src/risk/blend.rs`). A form that rounding has driven slightly negative therefore degrades into zero rather than into a negative variance.

The clamp is placed on the form rather than on the variance it feeds, which is the whole of the decision. A negative variance is not a small error: it is a value with no interpretation, and it propagates into a weight, a ratio, and an output that a consumer has no way to recognise as impossible. Clamping at the form catches it where the sign still means something local. The guard's site is in the code and its value is not transcribed here.

**Decision (Early refit is triggered unconditionally)** · `dec:calibration:unconditional-refit`

Any registration or deregistration — of a Sentinel, an outcome axis, or an identity dimension — sets the early-refit flag. There is no condition on the flag: it is not gated on a blend-weight estimate, a convergence stage, or a measure of how much the event changed the model (`src/owner/lifecycle.rs`).

The trigger is unconditional because the condition it replaced cost more than it saved. Every lifecycle event changes the model's dimensionality and can alter the score-to-probability relationship, so the condition was a non-trivial estimation performed at lifecycle time to decide whether to do work that a refit does correctly and cheaply anyway. Removing it removed a configuration parameter with it.

**Corollary (The blend stays inside the anchor's subspace)** · `cor:calibration:subspace-containment`

Because the form is computed by index over the anchor's coordinates (`dec:calibration:indexed-form`), the blend cannot produce a quantity outside the anchor's subspace: there is no step at which a component off that subspace could enter, since none is ever read.

Containment is structural rather than checked, which is the same relation the substrate record establishes for symmetry. The specification states the containment as a guarantee and the variance result that depends on it (`inv:guarantee:blend-variance`); this record's contribution is that the guarantee is discharged by the shape of the computation and needs no assertion at the call site.

**Register (The supersession this record collapses, and what cannot be read)** · `reg:calibration:collapsed-supersession`

One legacy record conditioned the early refit on a blend-weight check. A second replaced it with an unconditional trigger and said so three times over; a third generalised the trigger to every registration event. The census recorded the sequence with its evidence (`reg:assayer:adr-contradictions-records`).

The condition itself cannot be stated in full, and that is the finding rather than an omission here. No record in the corpus ever wrote it out: each supersession names the check and none reproduces it, so the set held a supersession whose object could not be read. What is recoverable is its shape — a blend-weight estimate evaluated at lifecycle time — and its fate. The trigger is now (`dec:calibration:unconditional-refit`), unconditional, stated once.

**Remark (Why two agreeing copies are not sharing)** · `rem:calibration:sharing-rationale`

Two copies of the transition function that currently agree are not the same as one function used twice, even while every value they produce matches. The property that matters is not present agreement but what happens to the next edit: a constant tuned in one copy leaves the other behind, and nothing in either call site would show it.

What that would produce is specific rather than general. The fit would weight evidence by one regime boundary and the blend would place models against another, so the calibration would be fitted on a partition its consumer does not use — an artefact that appears as a systematically miscalibrated output and not as a failure.

**Caveat (Outputs before the first refit are uncalibrated)** · `cav:calibration:pre-calibration`

Until the first refit meets the sample floor, the calibration parameter is at its initial value (`def:platt:initial-value`) and the outputs it scales are not calibrated. They are ordered — a higher score still means higher risk — but the probability attached to one is not one a consumer may read as a frequency.

The specification states this rather than promising otherwise (`cav:limitation:pre-calibration`) and fixes the stages a reader can use to tell where the system is (`tab:warmup:stages`). It is repeated here because it is the direct cost of the sample floor this record chose, and a reader who has just been told that fitting is floored is owed the sentence saying what the floor leaves uncovered.
