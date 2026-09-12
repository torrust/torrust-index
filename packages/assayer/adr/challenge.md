# Challenge · `rec:challenge:companion-evidence-model`

This record owns the challenge estimate: the conjugate model behind it, the boundary that keeps it host-owned, the sufficiency floor its health report reads, and the fact that it shipped before anything consumed it. It realises the challenge-model choice (`dec:numerics:challenge-model`) and the Companion boundary choice (`dec:contracts:companion-boundary`).

It carries a finished model, a contract describing a different arrangement, and no connection between the two. Rather than presenting a coherent design that does not exist, the record states both halves and the gap between them; two of its caveats exist for no other purpose.

**Decision (The estimate is a conjugate model with lazy decay toward its prior)** · `dec:challenge:conjugate-model`

The challenge estimate is a conjugate model updated in closed form (`alg:companion:update`), guaranteed conjugate rather than approximated (`inv:guarantee:conjugacy`), with decay applied lazily toward the prior (`alg:companion:decay`).

Conjugacy is the decision and cheapness is its consequence. A closed-form update means the estimate is maintained by arithmetic on a pair of counts rather than by an iterative fit, so it costs the same whether it is updated once or a thousand times, and it has no convergence behaviour of its own to go wrong. Laziness follows for the same reason: with the update in closed form there is nothing gained by decaying state nobody is reading.

**Decision (An override does not pause accumulation underneath it)** · `dec:challenge:override-accumulates`

While a host override stands, the underlying estimate keeps accumulating evidence, so clearing the override recovers the state that evolved while it was in force (`alg:companion:override`).

The alternative — freezing the model under an override — makes the override a trap. A host that set one during an incident and cleared it a week later would get back a week-old estimate, at exactly the moment it most wanted a current one, and would have no way to distinguish stale state from converged state. Continuing to accumulate means an override costs what it should cost — control of the output while it stands — and nothing more.

**Decision (Injected evidence is indistinguishable and capped)** · `dec:challenge:capped-injection`

Evidence a host injects enters the same counts as observed evidence and is capped, so injection cannot overwhelm observation (`alg:companion:injection`).

The two halves answer each other. Indistinguishability is what makes injection useful — seeded evidence must behave like the real thing, or the model would need two update rules and two interpretations of its own counts. The cap is what makes indistinguishability safe: without it, a host could write any posterior it liked by injecting enough, and the estimate would report the host's belief back to the host as though it were a measurement.

**Decision (Companion state is host-owned and reaches no Core structure)** · `dec:challenge:host-ownership`

The state lives with the host. It is absent from the working copy and from the published snapshot alike, and the boundary is the specification's (`inv:companion:boundary`). The exclusion from what the package publishes is the retention record's (`dec:retention:companion-excluded`).

Host ownership is what keeps the challenge estimate from acquiring a vote in assessment. The estimate is about the host's own interventions, so a Core that read it would be learning from its own downstream effects — and the feed-forward direction the package holds everywhere else (`dec:ownership:feed-forward`) exists precisely to make that impossible rather than merely discouraged.

**Decision (Estimation is infallible)** · `dec:challenge:infallible-estimation`

An estimate query always returns a value: the prior, an override, or a posterior (`src/risk/challenge.rs`). There is no error channel and no absent case a caller must handle.

The three outcomes are total by construction, which is what makes infallibility honest rather than a swallowed failure. With no evidence the prior is the correct answer and not a placeholder standing in for one; with an override the override is; with evidence the posterior is. A fallible surface would have to invent a fourth condition to report, and its caller would have to invent a policy for a case the mathematics does not produce.

**Decision (The sufficiency floor is host-owned and lives here)** · `dec:challenge:sufficiency-floor-owned-here`

The effective-sample floor at which a channel's estimate is reported as sufficiently evidenced belongs to the Companion and stands beside the tracker whose health report reads it (`src/risk/challenge.rs`), not in the Core convergence construction configuration where it shipped.

The move is the boundary applied to a figure rather than a new judgment about it. A threshold that decides what a host's own health report says about a host's own channel is the host's, exactly as the state it reads is (`dec:challenge:host-ownership`), and the source that carried it said so itself: its dated notice of 2026-05-17 asked for this relocation and was still asking when the figure moved. What Core lost with it was nothing it used — the composite took Companion sufficiency as a deliberately unused argument and the label path handed it a hard-coded placeholder, so removing both changes no stage the package computes. The study that sized the move is [the convergence-thresholds decisions](../docs/reports/convergence-thresholds/decisions.md), and it rejected the alternative of connecting the figure to the composite as contrary to this record's own boundary and to report-only health (`dec:health:reports-never-gates`).

The second Companion figure that stood beside it, a minimum-samples count, is deleted rather than moved. It named no operation: the tracker has one sufficiency comparison and no second floor, and no constant, parameter, or name anywhere in the package answered to it. A figure with no operation has no home to be moved to.

**Corollary (The independence is structural rather than asserted)** · `cor:challenge:structural-independence`

Because the state is absent from the working copy and from the published snapshot (`dec:challenge:host-ownership`), Core independence from the Companion is a property of what exists rather than a rule anyone must follow (`inv:guarantee:companion-independence`).

The distinction is the same one the ownership record draws about the feed-forward direction: a guarantee discharged by structure cannot be broken by an edit that forgets it, because there is no field to read. Were the state merely *ignored* by Core rather than absent from it, independence would hold only as long as every future author remembered — and this record could promise it no further than the next change.

**Remark (Why decay blends toward the prior rather than scaling the counts)** · `rem:challenge:decay-by-blending`

Decay moves the state toward the prior by blending, not by scaling its counts down. The form is the specification's deliberate choice (`dec:companion:decay-form`), and the reason is what the two forms do at their limits.

Scaling shrinks the evidence toward nothing, so a long-idle estimate approaches a state of no information — which is not what an idle entity means. Blending moves it toward the prior, so idleness returns the estimate to the default belief rather than to an absence of belief, and the bound on how fast that happens is the specification's (`bound:companion:convergence`). The distinction matters most exactly where the data is thinnest, which is the common case.

**Caveat (Every decision here is implemented, and nothing calls it)** · `cav:challenge:unwired`

The mathematics above is implemented and exported, and no runtime path invokes it. The model's constructors appear only in tests and test support; the code says so itself, in a labelled notice at the constructor recording that only tests construct the tracker (`src/risk/challenge.rs`).

This is stated first among the caveats because it changes how everything above should be read. The decisions are real — they bind a shipped library type that the rewrite must carry — but none of them is load-bearing for any behaviour the package exhibits today. The census recorded the finding rather than resolving it (`obs:assayer:adr-challenge-unwired`), and a reader who took this record for a description of running code would be wrong about every decision in it.

**Caveat (The arrangement the contract describes does not exist)** · `cav:challenge:arrangement-open`

At the dated finding, three decisions of the legacy Companion contract described an arrangement the package did not implement: a tracker replaceable through a published trait, a standalone Companion health surface, and a metrics mapper under its own prefix. The mapper still does not exist, so the arrangement as a whole remains incomplete.

The health surface was the first exception, and this record used to deny it. A standalone report does exist: the tracker builds one at the moment of reading, its envelope and per-channel detail are public, and both are exported from the crate root (`src/risk/challenge.rs`, `src/lib.rs`), and the specification enumerates its fields as a table of the chapter's own (`tab:companion:health`). The replacement trait is the second exception (`claim:risk:the-shipped-tracker-answers-the-replacement-surface-with-its-exact-posterior-counts`). What remains absent is the mapper limb, so this record keeps the caveat and names that surviving gap rather than letting the dated three-part denial stand (`cav:contracts:companion-arrangement`).

**Caveat (The sufficiency floor is twenty, and one declared tier can never reach it)** · `cav:challenge:sufficiency-threshold-open`

The floor is twenty effective samples beyond the prior. The label is older than this description and is kept as it stands, because the study and the register cite it and a name outlives what it once described; what changed is that the record now states a decision where it used to state a conflict.

The floor arrived here at the fifty it shipped with, and fifty is not a figure this corpus can derive. The specification's own target for Companion convergence is twenty effective observations (`bound:companion:convergence`), and its warm-up reading aid distinguishes about ten contributing labels for a first useful estimate from about twenty for practically useful variance (`tab:warmup:milestones`). Fifty answered to neither anchor. Twenty is taken on those grounds, over the commissioned sensitivity analysis, which also found fifty unsustainable below moderate targeting: at the specified contributing rates a weakly targeted channel would first cross fifty after about nine hundred and sixty-seven days and would never stay above it between one contribution and the next ([the sensitivity analysis](../docs/reports/convergence-thresholds/sufficiency-floor-sensitivity.md)).

What twenty means in operation is stated here rather than left for a reader to discover, because it is not free. Under the shipped hourly retention a channel fed at a regular rate settles at a ceiling in effective samples, and only a channel whose ceiling clears the floor can be sufficient at all (`alg:companion:decay`). Weakly, moderately and well targeted channels settle at about fifty and a half, a hundred and a half, and a hundred and sixty-seven, and all three sustain twenty. An untargeted channel, at the specified eight hundredths of a contributing label a day, settles at about seventeen and a sixth and never reaches twenty — not late, not eventually, never. That residue is accepted rather than hidden: it is the true report of a stream too thin for the quality this floor names, and a host that wants the Companion at that tier must target better, inject defensible evidence, or set an override (`alg:companion:injection`).

Two things keep the cost of a wrong reading small and are worth stating so that the next reader does not overestimate it. The floor decides a reported boolean and gates nothing (`dec:health:reports-never-gates`), so a wrong value misleads a reader and changes no output. And no runtime path reads it: an exact-name search finds exactly one reader outside the declaration, a crate test that pins the value and hands it to the health report, and no production caller (`cav:challenge:unwired`).

**Caveat (The evidence is thin where it matters)** · `cav:challenge:thin-evidence`

The estimate converges only as labels arrive (`cav:limitation:challenge-convergence`), and the labels that contribute to it are thin in realistic deployments (`cav:limitation:challenge-thin`).

The two facts compound rather than merely coexist. A conjugate model with little evidence sits near its prior, which is the correct behaviour and also the least informative one, so the estimate is closest to useless exactly where a host would most want it. Stated plainly, this bounds what the decisions above are worth: the mathematics is sound, and soundness is not the same as having enough data for the answer to say anything.
