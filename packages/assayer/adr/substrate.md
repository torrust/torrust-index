# Substrate · `rec:substrate:dense-symmetric-model`

This record fixes what the posterior is carried in: the shape the model has, the type its matrices are, and the dependency underneath them. It realises three layer choices — the model shape (`dec:structural:model-shape`), the matrix substrate (`dec:representation:matrix-substrate`) and the isolation of the backend behind it (`dec:representation:backend-isolation`) — and it owns the mirror direction, which the census found two records specifying oppositely while each attributed the choice to the other.

The specification states the two Gaussian results and the guarantees that rest on them; this record decides the carrier that lets them be applied, and cites rather than restates.

**Decision (The model is dense and dynamically dimensioned)** · `dec:substrate:dense-dynamic`

The model is carried densely and its dimension is a runtime property, not a compile-time one. A lifecycle change does not grow the structure in place: it reallocates a fresh one at the new dimension and moves the retained content into it, which is what makes a structural operation exact on the structure it produces rather than approximately right on a structure it patched (`inv:guarantee:structural-exactness`).

Reallocation is the expensive half and is chosen anyway. Growing in place would spread the cost across the operations that follow it and would leave the exactness guarantee resting on the correctness of every patch, where reallocation leaves it resting on one construction.

**Decision (The anchor model is never extended and never marginalised)** · `dec:substrate:anchor-invariant`

The anchor model (`def:risk:anchor-model`) has an invariant dimension. No lifecycle event extends it and none marginalises it: it is the fixed member of the triple (`def:risk:model-triple`), and every other member may change shape around it while it does not.

Fixity is what makes it an anchor rather than a third model. A quantity compared against something that also moves measures nothing stable, so the value of the anchor to every path that reads it is exactly the value of this decision. An axis registration therefore preserves the posterior it extends without the anchor participating (`inv:guarantee:axis-lifecycle`).

**Decision (The assessment path reads the covariance only)** · `dec:substrate:covariance-only`

The assessment path reads the covariance and never the precision matrix. Both are tracked, for the reason the specification records (`rem:gaussian:dual-tracking`), but only one of them is on the read path, and the separation is what lets the retention record exclude the other from what is published.

This is a decision about which quantity answers a request, not about which is authoritative. Nothing here says the precision matrix is secondary; it says the path that answers a request has no use for it, and that a structure carried for the benefit of a path that never reads it is bulk.

**Decision (Symmetry is a type invariant, not a caller's duty)** · `dec:substrate:symmetry-invariant`

The matrices are carried in a wrapper whose symmetry is an invariant restored after every mutating operation. It is not a property callers preserve by convention and it is not checked at the sites that consume it: each named operation writes and then restores, so the invariant holds at every point a caller can observe (`src/linalg/symmetric.rs`).

The alternative is symmetry as a documented obligation. It fails in the ordinary way — one mutation site added later that does not restore, and every consumer downstream reads an asymmetric matrix as though it were symmetric, with no error anywhere and a wrong answer everywhere.

**Decision (No raw mutable access is exposed)** · `dec:substrate:no-raw-access`

The wrapper exposes no raw mutable view of its contents. Every mutation goes through a named operation, and the mirroring step that restores the invariant is private to the type. A caller cannot reach past the operations to write an entry directly, which is what makes the previous decision an invariant rather than a habit.

The two decisions are one mechanism seen twice. Restoring after every mutation guarantees nothing if a caller can mutate without going through an operation that restores, so closing the raw path is not a further precaution but the premise of the invariant.

**Decision (Two constructor trust levels)** · `dec:substrate:trust-levels`

Construction is split by provenance. A matrix the package computed is wrapped without validation, on the reasoning that rounding asymmetry from its own arithmetic is expected and is repaired by mirroring. A matrix restored from storage is validated against a tolerance and may be refused, because nothing vouches for what a stored byte range contains.

The split is the useful part. One permissive constructor would silently accept a corrupt checkpoint; one strict constructor would make every internal computation pay a validation pass for asymmetry it created itself and immediately repairs.

**Decision (A single linear-algebra backend serves the whole workspace)** · `dec:substrate:single-backend`

There is one linear-algebra backend and no second one behind a feature flag, a platform condition, or a fallback path. Every matrix operation in the package resolves to the same dependency (`packages/assayer/Cargo.toml`).

One backend means one set of numerical behaviours. Two would mean the results of a factorisation depend on which build produced them, and every guarantee stated about conditioning or exactness would quietly acquire a qualifier naming the build.

**Decision (The wrapper isolates the backend from every caller)** · `dec:substrate:backend-isolation`

No caller names the backend's types. The wrapper is the only place they appear, so replacing the dependency is a change to one module rather than to every site that touches a matrix. What the callers name is the package's own type and the operations this record decides.

The isolation answers a different question from the substrate invariant — what the package depends on, rather than what it guarantees about its own type — and the two are separated for that reason rather than because either is large.

**Decision (Serialisation is written by hand)** · `dec:substrate:hand-serialisation`

The persisted form is written by hand rather than derived from the backend's own types (`src/linalg/serde_support.rs`). The package therefore owns its stored layout, and that layout does not change because a dependency changed its representation.

This is what makes the previous decision affordable in the presence of durable state. A derived encoding would tie the format of every checkpoint on disk to the internal shape of a third-party type, so replacing the backend would be a change to one module and an invalidation of every checkpoint ever written.

**Convention (Mutation writes one triangle and mirrors into the other)** · `conv:substrate:mirror-direction`

The lower triangle is authoritative. Every mutating operation writes it and then mirrors lower into upper; the reverse direction appears nowhere. Stated here and nowhere else, so that no second statement of it can drift from this one.

The direction itself is arbitrary and its being fixed is not. Either triangle would have served, and the cost of leaving the choice unstated is the divergence recorded below — two records specifying opposite directions, and a helper specified at length for a direction the code does not use.

**Register (The two mirror directions this record collapses)** · `reg:substrate:collapsed-mirror`

The corpus carried both directions. One record wrote the lower triangle and mirrored lower to upper; another specified the upper mirrored into the lower and attributed that choice to the first, and specified a helper for it. Its own review table then conceded that the first governs, that the implementation writes lower, and that the helper was never implemented — and the body was never corrected, so that record asserted both. The census recorded the divergence with its evidence (`reg:assayer:adr-contradictions-records`).

The code adjudicates for the lower triangle, and the convention above is now the one statement. The conceding review table is the shape worth naming: a record can know it is wrong, say so in a table, and go on being wrong in its body.

**Corollary (The invariant lets the Gaussian results stand as premises)** · `cor:substrate:premise-standing`

Because symmetry holds at every observable point, the extension and marginalisation results (`thm:gaussian:extension`) and (`thm:gaussian:marginalisation`) can be applied without revalidating their premise at each call site. Their premise is a property of the type rather than of the argument, so there is nothing per-call to check and no per-call check to forget.

The precision guarantee rests on the same footing (`inv:guarantee:precision`). This is why the record is a substrate rather than a utility: what it decides is consumed as an assumption by every numerical decision downstream, none of which restates it.

**Remark (The isolation is what makes the tabulated costs meaningful)** · `rem:substrate:cost-ownership`

The specification tabulates the cost of each matrix operation (`tab:gaussian:operation-costs`). Those figures are properties of the algorithm only because the backend is isolated and singular: with a second backend, or with callers reaching the dependency directly, the table would describe whichever implementation happened to be linked, and would need a column naming it.

The costs are therefore owned by the specification and are not restated here. What this record contributes is the condition under which they mean anything at all.

**Discussion (The rejected sparse representation)** · `disc:substrate:sparse-alternative`

The alternative was a sparse representation. It was available and rejected on the feature vector rather than on the matrices: the vector is dense by construction, so there are no structural zeros for a sparse form to decline to store. What it would buy is indirection on every access and a fill-in problem on every update, paid for a saving the density of the input does not offer.

Sparsity would become worth reconsidering if the vector ever acquired large blocks that are structurally absent rather than merely small. Nothing in the current composition does.
