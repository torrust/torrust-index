# The Assayer Specification: Labeled Outline · `plan:assayer:specification-rewrite-contract`

This is the labeled outline of the rewritten Assayer specification, produced under the campaign backlog's outline entry (`plan:assayer:specification-rewrite-contract`) and bound by the audit's verdict (`req:assayer:spec-outline-inputs`). It says, Part by Part and chapter by chapter, what environments the rewritten specification will carry, in order: the kind and label of each future head, what that environment is for in one line, whether it binds the implementation or explains it, and what the rewritten text must state or resolve where the audit and the section registers found the present text wrong, thin, or unimplemented. It is the specification's contract. The rewrite may refine wording freely; it adds or removes an environment only by amending this outline first.

It has been through the adversarial review its own open questions were addressed to (`plan:assayer:specification-rewrite-contract`), and this revision is the amended one. The review re-derived every count mechanically, opened a sample of the Part IV status flags against the code, checked the kind and force calls against the section registers and the deferral register, and adjudicated the fourteen open questions. What it changed, it changed here rather than reporting elsewhere; the places where a claim was corrected say so in the passage that carried it, so that a reader who remembers the earlier figure can see why it moved. Three questions remain open and await a ruling, and they are marked as such where they stand.

The outline takes structure and discards numbering. The Part skeleton, the chapter sequence, and the chapter subjects of the audit's inventory survive intact; every section number in the present document is thrown away, and the three numbering holes the audit found inside a single revision (`obs:assayer:spec-numbering-holes`) are the argument that they must be. What replaces the numbers is the label at each head, minted here.

The document practices the discipline it serves. The label at each heading or environment head is that environment's mint; a parenthesized label in running text is a same-owner citation; material in double-backtick spans — including every future label this outline plans, every legacy tag form it derives from, and every legacy section locator — is displayed without participating. Every label this document mints has area `assayer`, and every kind is drawn from a fixed registry of environment kinds, under the grammar of the label calculus. The labels planned for the rewritten specification are data here and mint nothing until the rewrite carries them at a head.

## Method and derivation · `sec:assayer:spec-outline-method`

**Convention (How future labels are derived)** · `conv:assayer:spec-outline-derivation`

Future labels are minted from the existing identity register, not invented. The audit's census counts 463 distinct tags of the superseded scheme (`dec:assayer:tags-superseded`) in 1,727 occurrences, of which 456 carry a mint site (`data:assayer:spec-tag-resolution`); each of those already names a concept an author judged citable, and the outline's task is to give it a kind and a conformant name rather than to invent an inventory. Four rules govern the derivation.

1. **Namespace becomes area.** The old scheme's first segment is a subject namespace; the calculus's area is the same idea under a different grammar, so each namespace maps to one area word, expanded from its abbreviation and chosen for readability. The mapping is (`tab:assayer:spec-outline-areas`) and it is exhaustive over the 44 namespaces the census finds. Areas are words over lowercase letters and digits with no hyphen, which is why every area below is a single noun.
2. **Slug becomes name, grammar-adjusted.** The old slug becomes the label's name segment, hyphenated where the grammar requires it and spelled out where the abbreviation would be opaque to a reader who never saw the old scheme — a slug reading `q-depth` becomes `question-depth`, a slug reading `vs-id` becomes `versus-identity`. The adjustment never changes which concept is named; where it did, the row would be a new mint and marked so.
3. **Kind is chosen, never inherited.** The old scheme had no kinds. Each row's kind is chosen from the registry against what the material actually is, taking the section registers' *kinds yielded* columns as the starting call and the audit's distribution (`tab:assayer:spec-implicit-kinds`) as the starting budget. Where this outline departs from a register's call, the chapter entry says so and why.
4. **Folding, and what does not earn a head.** An identity whose material is one row of a table, one clause of an enumeration, or one bullet of a list folds into the environment carrying that table, enumeration, or list, and does not become an environment of its own. This is the restraint of (`cav:assayer:spec-implicit-overreach`) made operational: the 343 bold-lead paragraphs are not 343 environments, the 110 display-math blocks want the equation treatment of a display rather than a head each, and Appendix H's material wants imported citations rather than Assayer-owned environments. The folded identities are recorded per chapter and totalled in (`data:assayer:spec-outline-budget`).

New environments — material the present specification needs and does not carry, chiefly the missing rendering-configuration table (`obs:assayer:drift-rendering-config`) and the environments that resolve the dangling identities (`reg:assayer:spec-dangling-tags`) — are minted deliberately and flagged **NEW** in their row. Every new mint is listed in (`reg:assayer:spec-outline-new`).

**Table (Namespace to area)** · `tab:assayer:spec-outline-areas`

The 44 namespaces of the census, their future areas, and the three that mint no area at all. Areas are checked against the ratified record naming schema (`dec:assayer:record-naming-schema-text`), whose second clause makes an area unique across the owner: where the natural noun is a likely record area — `concurrency`, `calibration`, `health`, `identity` — this outline takes the specification's own material noun instead. There are four such avoidances and the table marks each; a fifth row once carried the same mark for a choice that was readability rather than avoidance, and the review corrected it. A residual collision with an area the record set later claims was left as a finding for (`plan:assayer:specification-rewrite-contract`); the review has now made it, and the policy that settles it is recorded at the twelfth open question, which the review closed. The short form is that these four avoidances were aimed at the wrong targets and the areas stand anyway.

| Namespace | Area | Note |
| --- | --- | --- |
| ``arch`` | ``architecture`` | the component split and its drawing |
| ``ax`` | ``axis`` | outcome-axis prediction models |
| ``bayes`` | ``gaussian`` | the Gaussian algebra, named for its object rather than its school; a readability choice, not an avoidance |
| ``bg`` | — | Appendix H; imported material, no Assayer area |
| ``cal`` | ``platt`` | the calibration parameter; avoids ``calibration`` |
| ``cfg`` | ``config`` | the configuration surface |
| ``cns`` | ``constraint`` | the five structural constraints |
| ``comp`` | ``companion`` | the Companion Tracker |
| ``conc`` | ``publication`` | published state and staleness; avoids ``concurrency`` |
| ``det`` | ``detection`` | detection analysis |
| ``dn`` | — | design notes; dissolved into their subject areas |
| ``elig`` | ``eligibility`` | the training-eligibility contract |
| ``enc`` | ``encoding`` | the encoding contract |
| ``ex`` | ``landscape`` | the worked landscapes belong to their chapter's area |
| ``ext`` | ``extraction`` | per-Sentinel structured extraction |
| ``fmap`` | ``boundary`` | the interface map and its verification |
| ``frag`` | ``fragility`` | decision fragility |
| ``guide`` | ``guidance`` | Core label guidance |
| ``hm`` | ``monitoring`` | health monitoring; avoids ``health`` |
| ``host`` | ``host`` | the host contract |
| ``id`` | ``keyspace`` | the key space identity layer; avoids ``identity`` |
| ``imp`` | ``weighting`` | importance weighting |
| ``inv`` | ``guarantee`` | the formal properties register |
| ``land`` | ``landscape`` | the decision landscape |
| ``led`` | ``ledger`` | the Outcome Ledger |
| ``lim`` | ``limitation`` | the known-limitations register |
| ``map`` | ``dimension`` | the dimension map |
| ``model`` | ``risk`` | the core risk models |
| ``mot`` | — | the old scheme's mottos; retired with the scheme |
| ``out`` | ``output`` | the output structures |
| ``phi`` | ``feature`` | the feature vector and its templates |
| ``pol`` | ``channel`` | channel policy |
| ``prin`` | ``principle`` | the three design principles |
| ``reg`` | ``registry`` | the three registries and their lifecycles |
| ``rend`` | ``rendering`` | the resonance rendering layer |
| ``res`` | ``resource`` | convergence and resource bounds |
| ``run`` | ``runtime`` | the operational cycle |
| ``sig`` | ``signal`` | host signals |
| ``sm`` | ``update`` | the model update path |
| ``std`` | ``standardisation`` | online standardisation |
| ``time`` | ``temporal`` | temporal governance |
| ``val`` | ``valence`` | the valence asymmetry |
| ``warm`` | ``warmup`` | initialisation and warm-up |
| ``xterm`` | ``terminology`` | cross-layer terminology mapping |

One further area is new and has no namespace behind it: ``spec``, which carries the document's own skeleton — its Parts, its chapters, and its appendices. Those heads are structure, and the structure is what survives.

**Convention (Reading the outline tables, and why they are not tracking tables)** · `conv:assayer:spec-outline-columns`

Five columns per row. *Kind* is the registry kind the future head will carry. *Label* is the full label that head will mint. *Derived from* names the identity the label is minted from, as a displayed legacy form; it reads NEW where the environment is created here and has no material in the present document, and *unminted* where the material is present but was never given an identity — a division head, a record the old scheme left anonymous, a procedure stated inside another section's prose. An unminted row is not a new environment: it is an existing one the old scheme failed to name, which is itself evidence for the supersession. *Scope* is one line saying what the environment is for — the sentence the rewrite writes to. *Flags* carries the binding force, the Part IV status where one applies, and the drift the rewritten text must state or resolve, citing the register that found it.

These tables are deliberately **not** the tracking tables of the outline tracking tool. That tool reads a table as a tracking declaration exactly when its header cells are Entry, Head and Document, and it would then verify every declared head against the tracked document as it stands today — which mints nothing, being carried as a pre-calculus document (`obs:assayer:spec-nonparticipation`). Every row here would fail, correctly and uselessly. Conversion to live tracking tables happens at the concatenation entry (`spec:spec:measurement-judgement-interpreter`), in the same commit that removes the specification from the pre-calculus list and makes its heads real. Until then the header above keeps the tables inert, and the labels sit in double-backtick spans so that no cell mints.

**Convention (Binding force is declared at the head)** · `conv:assayer:spec-outline-force`

The present specification marks binding force nowhere: it uses no capitalised requirement keyword at all (`obs:assayer:spec-no-modal-vocabulary`), and what binds is inferred from position. The rewrite ends that, and the kind at the head is where it ends: an ``inv``, a ``req``, a ``pre``, a ``def``, a ``schema``, an ``alg``, a ``conv`` and a ``tab`` of defaults bind the implementation; a ``mot``, a ``rem``, a ``disc``, an ``ex``, a ``data``, a ``bound``, a ``cav`` and a ``scenario`` do not. The Flags column states the force in words anyway, because the audit's partition (`tab:assayer:spec-partition`) is a reading and this outline is where it becomes a decision, and a decision should be legible without decoding the kind. Where this outline's force differs from the audit's chapter-level partition, the chapter entry says so.

Two forces need naming beyond binds and explains. **Discloses** is the force of the limitation register: a limitation constrains no implementation but the specification is answerable for stating it, so removing one is a substantive edit and not a tidy-up. **Obliges** is the force of an analytic property: six of the twenty-nine formal properties are settled by neither code nor test (`summ:assayer:spec-sections-c`), and their force is a proof obligation on the document, not a constraint on the crate.

**Convention (Part IV status, per environment)** · `conv:assayer:spec-outline-partiv`

The audit's fourth requirement is that the outline record, per environment, whether it describes the shipped system or unbuilt work, and it applies wherever a register found the question live — Part IV throughout, and four places outside it. The specification is authoritative and the shipped surface is the legacy party (`dec:assayer:spec-authoritative`), so the status is never a criticism of the environment. Three values, taken from the Part III–IV register's vocabulary (`conv:assayer:s34-verdicts`): **shipped**, the implementation carries this surface; **unbuilt**, the specification requires a surface the implementation does not carry; **shared-shape**, the implementation answers the same need under a different shape or name, so the migration re-forms rather than builds.

**Data (What this outline plans)** · `data:assayer:spec-outline-budget`

Counted over the outline tables below, recomputed at the review, and true at this revision. **478 environments**, every label distinct — 475 as first drawn, plus the computation order the review restored into Chapter 15 and the two background overviews restored for independent shareability. By Part: 48 in Part I, 97 in Part II, 61 in Part III, 52 in Part IV, 16 in Part V, 47 in Part VI, 48 in Part VII, 83 in Part VIII, and 26 across the front matter and the appendices. The largest chapters are the guarantees and limitations register at 65, the feature vector at 38, the core risk models at 35, health monitoring at 29 and the registries at 27; the smallest chapters are host signals at 3 and the valence asymmetry at 4, though two appendices and the front matter are smaller still.

By kind, the ten most used: ``tab`` 86, ``def`` 84, ``alg`` 56, ``cav`` 45, ``inv`` 35, ``req`` 26, ``rem`` 16, ``sec`` 16, ``prop`` 11, ``ex`` 10 — and twenty-one further kinds below ten, including two ``preview`` environments and down to one each for ``pre``, ``constr``, ``just`` and ``pf``, thirty-one kinds in all. By force: 318 bind, 93 explain, 45 disclose, 6 oblige, and 16 are structural division heads carrying no material. Every kind used is drawn from the registry of environment kinds; the review checked all thirty-one against it and found no departure.

By provenance: 389 rows derive from an existing identity, 77 name material the old scheme left unminted, and 12 are new material. Thirteen environments are flagged NEW in their rows; one is the rendering configuration table, which derives from a dangling identity and carries material the document does not have. Forty-one areas are claimed, forty from the namespace mapping and one for the document skeleton. By Part IV status, counted over every row that carries one: 214 shipped, 74 unbuilt, 21 shared-shape. The review moved two of these: the coordinate lift left *shipped* because the code contradicts it, and the restored computation order entered *shared-shape*. Those three figures overlap by design — thirteen rows carry two of them — so 296 distinct rows carry a status at all and 180 carry none.

The folding total that (`conv:assayer:spec-outline-derivation`) promises here, supplied at the review. The 389 deriving rows name 392 legacy forms, two of which are ranges — the five compositional dividends and the six warm-up stages — so the rows between them consume **400 of the census's 456 minted identities**. The remaining **56 minted identities earn no head**: thirty are Appendix H's, disposed as imported material, and the rest fold into the environment carrying their table, enumeration or list, or belong to the chapter and three appendices that mint nothing at all (`reg:assayer:spec-outline-noenv`). Folding is recorded in the chapter entries only where it changed a structural call — Chapters 1, 7, 24 and 28 — and the review found no chapter where an unrecorded fold swallowed material a section register marked binding.

The total exceeds the audit's estimate of roughly two hundred, and the excess is accounted for rather than apologised for: the estimate counted the environment-shaped prose and did not count three registers that are already environments in table clothing — twenty-nine formal properties, forty-two limitations, and sixteen configuration tables — nor the sixteen structural heads, nor the appendices. Net of those, the prose environments number close to the estimate. The configuration figure read eighteen before the review, which was the row count of Chapter 28 rather than its count of tables; the chapter mints sixteen tables beside a division head and a convention.

## Part I — Foundations · `sec:assayer:spec-outline-part-1`

The Part head mints ``part:spec:foundations``. Four chapters, of which the audit's partition reads two as normative, one as mixed and one as expository throughout — a distribution the review restated here, the earlier wording having counted only one chapter as binding. Twenty-five of the Part's forty-eight environments bind. Part I is where the outline's hardest editorial call falls, and it is not in the mathematics: Chapter 2 is the specification's most emphatic host requirement and has no implementation surface at all (`summ:assayer:spec-sections-a`), so its central environment is recorded here as unbuilt rather than quietly restated.

Structure. The heads below carry the Part itself and its chapters, and are claimed here because the outline tracks the specification's skeleton as well as its contents.

| Head | Document |
| --- | --- |
| ``spec:spec:measurement-judgement-interpreter`` | ``packages/assayer/docs/spec/foundations-purpose.md`` |
| ``part:spec:foundations`` | ``packages/assayer/docs/spec/foundations-purpose.md`` |
| ``chap:spec:purpose-and-principles`` | ``packages/assayer/docs/spec/foundations-purpose.md`` |
| ``chap:spec:encoding-contract`` | ``packages/assayer/docs/spec/foundations-encoding.md`` |
| ``chap:spec:valence-asymmetry`` | ``packages/assayer/docs/spec/foundations-valence.md`` |
| ``chap:spec:mathematical-foundation`` | ``packages/assayer/docs/spec/foundations-gaussian.md`` |

**Entry (Chapter 1, Purpose, Scope, and Principles)** · `entry:assayer:spec-outline-ch01`

The chapter head mints ``chap:spec:purpose-and-principles``. The chapter is the document's front door and reads as prose, but its binding content is larger than its prose suggests: ten of its seventeen environments bind, and one of them is the document's only unconditional structural guarantee. The register reaches the same place by a different route — it marks three of six sections normative or mixed, and those three decompose into the ten — so the difference is granularity and not disagreement. The earlier revision of this entry said two, counting sections it had already decided to split; the review corrected it, because a chapter whose force is understated by a factor of five is the one place a rewrite would take the wrong tone.

Three departures from the section register (`entry:assayer:spec-sections-ch01`). First, the register reads the assumed-properties table as one ``tab`` plus a ``pre``; this outline splits it, because two of the eight properties are enforced at ingestion with named errors and are therefore preconditions the Core checks rather than assumptions it makes, and a single table cannot carry both forces. Second, the five compositional dividends fold into one ``ex``: they are one argument in five movements, and five heads would be five citation targets for a passage nothing cites into. Third — recorded at the review, having been taken silently before — the ownership table is promoted. The register reads section ``1.4`` as expository throughout, and this outline makes ``tab:architecture:ownership`` bind as a boundary statement: it is the only enumeration of which component owns which state, the boundary appendix's verification table cites it, and a boundary that explains rather than binds is the defect the appendix exists to prevent. The drawing and the dividends beside it stay expository, so the promotion is of one row and not of the section.

The valence sign convention is minted here, at its first statement, and cited from the host contract and the risk target rather than restated there.

| Kind | Head | Derived from | Scope | Flags | Document |
| --- | --- | --- | --- | --- | --- |
| ``mot`` | ``mot:architecture:three-questions`` | ``arch:three`` | the three questions a host has and the three components that answer them | explains; the passage names the unbuilt derivation vocabulary — (`warn:assayer:drift-landscape-unimplemented`) | ``packages/assayer/docs/spec/foundations-purpose.md`` |
| ``prin`` | ``prin:principle:identity-preservation`` | ``prin:identity`` | structure is preserved rather than summarised away | explains | ``packages/assayer/docs/spec/foundations-purpose.md`` |
| ``prin`` | ``prin:principle:multi-level-decomposition`` | ``prin:decompose`` | evidence is decomposed across levels rather than pooled | explains | ``packages/assayer/docs/spec/foundations-purpose.md`` |
| ``prin`` | ``prin:principle:measure-not-decide`` | ``prin:measure`` | the Core measures and the host decides | explains | ``packages/assayer/docs/spec/foundations-purpose.md`` |
| ``inv`` | ``inv:guarantee:feed-forward`` | ``inv:feedforward`` | no Core state reaches a Sentinel; the one cycle runs through the host | binds; shipped, held by construction — the report crosses by value and no Sentinel handle exists in the Core; the review found the guard is a build-time lint rather than a check at reception, and the one test of it covers the identity path | ``packages/assayer/docs/spec/foundations-purpose.md`` |
| ``conv`` | ``conv:valence:sign`` | ``host:valence`` | positive valence is the adverse direction, and the sign enters only the risk target | binds; shipped; cited by the host contract and the risk target rather than restated | ``packages/assayer/docs/spec/foundations-purpose.md`` |
| ``fig`` | ``fig:architecture:overview`` | ``fig:arch:overview`` | the component drawing, with its configuration input restored | explains; the pre-rewrite drawing's configuration input was lost with the output rename | ``packages/assayer/docs/spec/foundations-purpose.md`` |
| ``tab`` | ``tab:architecture:ownership`` | ``arch:overview`` | which component owns which state, in seven rows | binds as a boundary statement | ``packages/assayer/docs/spec/foundations-purpose.md`` |
| ``ex`` | ``ex:architecture:compositional-dividends`` | ``arch:div-*`` | the five things the split buys, worked in one place | explains; folds five identities; two dividends rest on the presentation-free separation — (`obs:assayer:drift-rendering-not-optional`) | ``packages/assayer/docs/spec/foundations-purpose.md`` |
| ``assum`` | ``assum:constraint:dynamic-registries`` | ``cns:dyn-registry`` | registries change at runtime and the algebra must absorb it | binds as a standing assumption | ``packages/assayer/docs/spec/foundations-purpose.md`` |
| ``assum`` | ``assum:constraint:valence-asymmetry`` | ``cns:valence`` | outcomes are observed on one side of the action only | binds as a standing assumption | ``packages/assayer/docs/spec/foundations-purpose.md`` |
| ``assum`` | ``assum:constraint:crossover-structure`` | ``cns:crossover`` | the decision surface is a crossover structure, not a threshold | binds as a standing assumption | ``packages/assayer/docs/spec/foundations-purpose.md`` |
| ``assum`` | ``assum:constraint:encoding-dependence`` | ``cns:encoding`` | everything downstream depends on the host's encoding | binds as a standing assumption | ``packages/assayer/docs/spec/foundations-purpose.md`` |
| ``assum`` | ``assum:constraint:outcome-axes`` | ``cns:axes`` | outcome axes are declared by the host and uninterpreted by the Core | binds as a standing assumption | ``packages/assayer/docs/spec/foundations-purpose.md`` |
| ``tab`` | ``tab:architecture:sentinel-properties`` | ``arch:sentinel-props`` | the six properties of the batch-report interface the Core assumes | binds as preconditions on the Sentinel | ``packages/assayer/docs/spec/foundations-purpose.md`` |
| ``pre`` | ``pre:architecture:report-validation`` | NEW | the two properties the Core enforces at ingestion, with their named errors | binds; shipped; splits the assumed-properties table, per (`summ:assayer:spec-sections-a`) | ``packages/assayer/docs/spec/foundations-purpose.md`` |
| ``rem`` | ``rem:architecture:inherited-guarantees`` | NEW | the Sentinel-internal guarantees the Core inherits through the report without referencing them | explains; restores a paragraph the rewrite deleted, the restoration confirmed at the review — see (`reg:assayer:spec-outline-open`) | ``packages/assayer/docs/spec/foundations-purpose.md`` |

Citation spine. The feed-forward invariant is cited by the boundary verification, by the report-reception environment, and by the Companion's boundary statement; the valence convention by the host contract, the risk target, and the axis training target; the five constraints each by the chapter that develops them — dynamic registries by the registry chapter, valence by the asymmetry chapter, crossover by the landscape chapter, encoding by the encoding contract, axes by the axis registry. The ownership table cites nothing and is cited by the boundary appendix.

**Entry (Chapter 2, The Encoding Contract)** · `entry:assayer:spec-outline-ch02`

The chapter head mints ``chap:spec:encoding-contract``. The register's verdict governs the whole chapter: this is the specification's strongest statement of host responsibility and its most thoroughly unimplemented one, since Sentinel registration accepts a name and an identifier and nothing else (`entry:assayer:spec-sections-ch02`). The contract environment is therefore recorded as unbuilt, deliberately and at full strength, rather than softened to match the code — which the authoritative-specification ruling (`dec:assayer:spec-authoritative`) requires and the register asks for by name. The five guidance sections stay guidance and are not promoted: they are the clearest expository block in Part I.

| Kind | Head | Derived from | Scope | Flags | Document |
| --- | --- | --- | --- | --- | --- |
| ``req`` | ``req:encoding:host-contract`` | ``enc:contract`` | the host owns the encoding, and the two partial compensations that do not replace it | binds; **unbuilt** — no registration surface carries the declarations | ``packages/assayer/docs/spec/foundations-encoding.md`` |
| ``def`` | ``def:encoding:question-hierarchy`` | ``enc:q-hierarchy`` | what a prefix hierarchy means for this source | explains | ``packages/assayer/docs/spec/foundations-encoding.md`` |
| ``def`` | ``def:encoding:question-meaning`` | ``enc:q-meaning`` | what a shared prefix asserts about two keys | explains | ``packages/assayer/docs/spec/foundations-encoding.md`` |
| ``def`` | ``def:encoding:question-depth`` | ``enc:q-depth`` | how deep the meaningful structure runs | explains | ``packages/assayer/docs/spec/foundations-encoding.md`` |
| ``tab`` | ``tab:encoding:suitable-sources`` | ``tbl:enc:suitable`` | sources whose prefixes carry meaning, and what they carry | explains | ``packages/assayer/docs/spec/foundations-encoding.md`` |
| ``tab`` | ``tab:encoding:unsuitable-sources`` | ``tbl:enc:unsuitable`` | sources whose prefixes carry none, and why | explains | ``packages/assayer/docs/spec/foundations-encoding.md`` |
| ``tab`` | ``tab:encoding:curve-guidance`` | ``enc:curves`` | space-filling curve choices for multi-dimensional data | explains | ``packages/assayer/docs/spec/foundations-encoding.md`` |
| ``rem`` | ``rem:encoding:downstream-fallback`` | ``enc:fallback`` | downstream learning as a slow fallback, and the remedy that is not one | explains; its early-warning heuristic named a value that did not exist, and now names the shipped mean-slot-weight reading — (`entry:assayer:defer-sentinel-informativeness`) | ``packages/assayer/docs/spec/foundations-encoding.md`` |
| ``def`` | ``def:encoding:domain-width`` | ``enc:width`` | the declared per-Sentinel domain width and the fixed internal width | binds; the stated rejection rule is stated of the wrong registry — the range check exists only on the identity path | ``packages/assayer/docs/spec/foundations-encoding.md`` |
| ``alg`` | ``alg:encoding:coordinate-lift`` | ``enc:lift`` | lifting a declared-width coordinate into the internal representation | binds on the host; **carried by no Core surface** — the review's code pass found no lift routine in the crate, and the specification itself assigns the lift to the integration layer, so the register's *shipped* reading was wrong; the declared width reaches only a range check on the identity path | ``packages/assayer/docs/spec/foundations-encoding.md`` |
| ``dec`` | ``dec:encoding:fixed-internal-width`` | ``dn:fixed-width`` | why the internal width is fixed rather than generic | binds as a recorded decision | ``packages/assayer/docs/spec/foundations-encoding.md`` |
| ``rem`` | ``rem:encoding:unchecked-width`` | NEW | the declared width is an unchecked assertion; the correspondence is the host's | explains; restores a deleted sentence the register calls the one a reader most needed | ``packages/assayer/docs/spec/foundations-encoding.md`` |

Citation spine. The contract is cited by the identity chapter's key-space application, by the host-duties environment, and by the encoding constraint of Chapter 1; the domain width by the report-reception environment and by the registration protocol; the coordinate lift by the Ledger's routing. The three question definitions are cited by nothing and cite the contract, which is the correct shape for guidance.

**Entry (Chapter 3, The Valence Asymmetry)** · `entry:assayer:spec-outline-ch03`

The chapter head mints ``chap:spec:valence-asymmetry``. Expository throughout, exactly as both the audit's partition and the section register read it: the chapter argues for the design rather than constraining it, and the outline does not promote any of it. Four environments, one per section, and no folding is needed because the chapter's own structure is already one argument per section. The chapter carries the two numeric losses the register found — a coverage figure changed with no stated basis, and a falsifiable half-life replaced by an unfalsifiable statement — and both are carried into the review rather than inherited silently.

| Kind | Head | Derived from | Scope | Flags | Document |
| --- | --- | --- | --- | --- | --- |
| ``prin`` | ``prin:valence:censored-asymmetry`` | ``val:asym`` | outcomes are censored on one side, structurally and irreducibly | explains | ``packages/assayer/docs/spec/foundations-valence.md`` |
| ``disc`` | ``disc:valence:mitigation-layers`` | ``val:layers`` | the five architectural layers that manage the asymmetry | explains; two layers are written in the unbuilt fragility vocabulary; the anchor coverage figure wants a basis — see (`reg:assayer:spec-outline-open`) | ``packages/assayer/docs/spec/foundations-valence.md`` |
| ``alg`` | ``alg:valence:contamination-loop`` | ``val:loop`` | the eight-step contamination loop and its four mitigations | explains; the loop's half-life figure was replaced by an unfalsifiable statement — see (`reg:assayer:spec-outline-open`) | ``packages/assayer/docs/spec/foundations-valence.md`` |
| ``disc`` | ``disc:valence:axis-pathway`` | ``val:loop-axis`` | the same loop through spatially-enabled outcome axes | explains | ``packages/assayer/docs/spec/foundations-valence.md`` |

Citation spine. All four cite forward and are cited backward: the asymmetry is cited by the eligibility contract and by the Ledger's decay rationale, the mitigation layers by the guidance interface and the anchor model, the contamination loop by the Ledger's time-indexed decay and by the laundering limitation, the axis pathway by the axis registry's spatial policy.

**Entry (Chapter 4, Mathematical Foundation)** · `entry:assayer:spec-outline-ch04`

The chapter head mints ``chap:spec:mathematical-foundation``. The best-implemented chapter in Part I, and the outline's job here is mostly to give its results heads so they can be cited as results. Two departures from the register. The regularised Schur complement is split into the procedure and its positive-definiteness requirement, because the register found the code testing the diagonal where the specification means the spectrum, and a requirement stated in its own environment is a requirement a checker can be pointed at. The synchronisation error's *definition* moves to the monitoring chapter, where its threshold and its report live, and this chapter's monitoring procedure cites it — the register found the code computing a diagonal maximum where the specification means a Frobenius norm, and one definition site is how that stops recurring. The two formal properties this chapter states — structural exactness and per-observation exactness — are minted in the guarantees register of the last chapter and cited here.

| Kind | Head | Derived from | Scope | Flags | Document |
| --- | --- | --- | --- | --- | --- |
| ``thm`` | ``thm:gaussian:extension`` | ``bayes:ext`` | extension of a Gaussian posterior is exact | binds; shipped | ``packages/assayer/docs/spec/foundations-gaussian.md`` |
| ``thm`` | ``thm:gaussian:marginalisation`` | ``bayes:marg`` | marginalisation of a Gaussian posterior is exact | binds; shipped | ``packages/assayer/docs/spec/foundations-gaussian.md`` |
| ``def`` | ``def:gaussian:gathered-partition`` | ``bayes:gather`` | the gathered removed partition, for non-contiguous removals | binds; shipped | ``packages/assayer/docs/spec/foundations-gaussian.md`` |
| ``rem`` | ``rem:gaussian:dual-tracking`` | ``bayes:dual-note`` | why precision and covariance are tracked together, numerically | explains | ``packages/assayer/docs/spec/foundations-gaussian.md`` |
| ``def`` | ``def:gaussian:schur-complement`` | ``bayes:schur`` | what the Schur complement means for this posterior | binds | ``packages/assayer/docs/spec/foundations-gaussian.md`` |
| ``alg`` | ``alg:gaussian:regularised-schur`` | ``bayes:schur-reg`` | the regularisation delta, the host's posture guard and the package's solvability guard | binds; shipped, constants exact | ``packages/assayer/docs/spec/foundations-gaussian.md`` |
| ``req`` | ``req:gaussian:positive-definiteness`` | NEW | the marginalised precision is verified positive definite by its minimum eigenvalue | binds; shipped on the general path, which reads the spectrum; the rank-one path is covered by the head below rather than by this one | ``packages/assayer/docs/spec/foundations-gaussian.md`` |
| ``req`` | ``req:gaussian:rank-one-verification`` | NEW | the rank-one path verifies the B' it formed at the general path's boundary, by an outward-rounded Gershgorin certificate or by the spectrum, never by the diagonal alone, and reports which certificate decided | binds; shipped — closes the exception the Schur epsilon audit left open, against a represented-SPD parent whose implemented result carries a negative eigenvalue behind a positive diagonal | ``packages/assayer/docs/spec/foundations-gaussian.md`` |
| ``req`` | ``req:gaussian:marginalisation-error-reported`` | NEW | every marginalisation computes and reports the approximation it committed, labelled as a measurement or a bound, and the per-event figures fold into a cumulative monitor | binds; shipped | ``packages/assayer/docs/spec/foundations-gaussian.md`` |
| ``tab`` | ``tab:gaussian:operation-costs`` | ``tbl:bayes:costs`` | costs of extension, marginalisation and the per-observation update, with the per-matrix breakdown of the worked total | explains; the breakdown that let a reader check the total was lost in the rewrite and the review restored it, the arithmetic being implied by the rows above it | ``packages/assayer/docs/spec/foundations-gaussian.md`` |
| ``thm`` | ``thm:gaussian:two-level-guarantee`` | ``bayes:two-level`` | the two-level posterior guarantee, stated as a result | binds; cites the two exactness properties | ``packages/assayer/docs/spec/foundations-gaussian.md`` |
| ``cav`` | ``cav:gaussian:fixed-model-departure`` | ``bayes:no-fixed-model`` | where the fixed-model reading departs from the running system | discloses | ``packages/assayer/docs/spec/foundations-gaussian.md`` |
| ``prop`` | ``prop:gaussian:conservatism`` | ``bayes:conservative`` | the departure is conservative in the stated direction | binds as an analytic claim | ``packages/assayer/docs/spec/foundations-gaussian.md`` |
| ``data`` | ``data:gaussian:binding-frequency`` | ``bayes:bind-freq`` | how often the guarantee's condition actually binds | explains | ``packages/assayer/docs/spec/foundations-gaussian.md`` |
| ``alg`` | ``alg:gaussian:synchronisation-monitor`` | ``bayes:sync`` | monitoring the precision-covariance product against its definition | binds; shipped and reported per model and in aggregate, alongside the part of the reading the replenishment clamp put there and the residual left when that part is taken out; a clean rebuild whose residual is high shortens the effective interval, and the rebuild's own before-and-after readings decide whether its covariance is adopted | ``packages/assayer/docs/spec/foundations-gaussian.md`` |
| ``alg`` | ``alg:gaussian:condition-adaptive-recompute`` | ``bayes:cond-adaptive`` | recomputation triggered by the condition estimate | binds; shipped | ``packages/assayer/docs/spec/foundations-gaussian.md`` |
| ``req`` | ``req:gaussian:prior-replenishment-floor`` | ``bayes:floor`` | the floor below which prior precision is replenished | binds; shipped and reported | ``packages/assayer/docs/spec/foundations-gaussian.md`` |

Citation spine. Extension and marginalisation are the most-cited environments in the document: every lifecycle protocol of the registry chapter cites one or both, as do the axis lifecycle and the dimension-registry protocols. The gathered partition is cited by the identity chapter's exit path and by the dimension map's whole-dimension removal. The two-level guarantee is cited by the update path and by the guarantees register. The regularised Schur is cited by every deregistration protocol; its positive-definiteness requirement is cited by the precision-health environments of the monitoring chapter.

## Part II — The Data Model · `sec:assayer:spec-outline-part-2`

The Part head mints ``part:spec:data-model``. Five chapters and the largest share of the outline's environments, because this is where the specification is a data model rather than an argument: registries, extraction widths, feature blocks, the dimension map, and the standardisation procedure are all things a tool could check against constants it mirrors, and the section register argues for exactly that (`summ:assayer:spec-sections-a`). Two chapters here are sound in every section and two are the most drifted in the document; the outline marks the difference per environment rather than per chapter.

Structure. The heads below carry the Part itself and its chapters, and are claimed here because the outline tracks the specification's skeleton as well as its contents.

| Head | Document |
| --- | --- |
| ``part:spec:data-model`` | ``packages/assayer/docs/spec/data-model-registries.md`` |
| ``chap:spec:registries-and-lifecycle`` | ``packages/assayer/docs/spec/data-model-registries.md`` |
| ``chap:spec:structured-extraction`` | ``packages/assayer/docs/spec/data-model-extraction.md`` |
| ``chap:spec:key-space-identity`` | ``packages/assayer/docs/spec/data-model-identity.md`` |
| ``chap:spec:host-signals`` | ``packages/assayer/docs/spec/data-model-signals.md`` |
| ``chap:spec:feature-vector`` | ``packages/assayer/docs/spec/data-model-features.md`` |

**Entry (Chapter 5, Registries and Lifecycle)** · `entry:assayer:spec-outline-ch05`

The chapter head mints ``chap:spec:registries-and-lifecycle``. Four division heads survive as ``sec`` environments, because the chapter genuinely has four registries' worth of material and a flat chapter of twenty-five heads would be unreadable; they carry no material of their own and the outline says so. The Sentinel registration record is the chapter's decision point: the specification declares five fields of which three carry the encoding contract, the implementation has two, and every downstream claim fails with it (`summ:assayer:spec-sections-a`). Under the authoritative-specification ruling the record stands as specified and is recorded unbuilt. The lifecycle algebra around it is sound in every direction the register checked.

| Kind | Head | Derived from | Scope | Flags | Document |
| --- | --- | --- | --- | --- | --- |
| ``sec`` | ``sec:registry:sentinel`` | unminted | division: the Sentinel registry | structural | ``packages/assayer/docs/spec/data-model-registries.md`` |
| ``schema`` | ``schema:registry:sentinel-record`` | ``reg:snt`` | the five-field registration record and its coordinate semantics | binds; **unbuilt** — two fields ship, and the three carrying the encoding contract do not | ``packages/assayer/docs/spec/data-model-registries.md`` |
| ``tab`` | ``tab:registry:sentinel-operations`` | ``tbl:reg:snt-ops`` | add, remove and offline against interpreter, estimator, Ledger and map | binds; shipped | ``packages/assayer/docs/spec/data-model-registries.md`` |
| ``rem`` | ``rem:registry:anomaly-dilution`` | ``reg:snt-dilution`` | why adding a Sentinel dilutes rather than distorts | explains | ``packages/assayer/docs/spec/data-model-registries.md`` |
| ``alg`` | ``alg:registry:sentinel-registration`` | ``reg:snt-add`` | the registration protocol, step by step | binds; **unbuilt** in its first and fourth steps, which read fields that do not exist | ``packages/assayer/docs/spec/data-model-registries.md`` |
| ``alg`` | ``alg:registry:sentinel-deregistration`` | ``reg:snt-remove`` | the deregistration protocol and its marginalisation | binds; shipped, the hibernation archive included — (`entry:construction:hibernation`) | ``packages/assayer/docs/spec/data-model-registries.md`` |
| ``tab`` | ``tab:registry:coverage-states`` | ``reg:partial`` | the four coverage states, their occupancy and their feature values | binds; shipped | ``packages/assayer/docs/spec/data-model-registries.md`` |
| ``alg`` | ``alg:registry:hibernation`` | ``reg:hibernate`` | hibernation as an optional lifecycle state, and what it preserves | binds; shipped, under the archive contract at (`entry:construction:hibernation`) | ``packages/assayer/docs/spec/data-model-registries.md`` |
| ``def`` | ``def:registry:no-online-sentinel`` | ``reg:zero`` | the zero-Sentinel case and its reported flag | binds; shipped | ``packages/assayer/docs/spec/data-model-registries.md`` |
| ``tab`` | ``tab:registry:sentinel-scale`` | ``reg:no-max`` | no architectural ceiling, and comfortable counts by label rate | explains | ``packages/assayer/docs/spec/data-model-registries.md`` |
| ``sec`` | ``sec:registry:outcome-axis`` | unminted | division: the outcome axis registry | structural | ``packages/assayer/docs/spec/data-model-registries.md`` |
| ``def`` | ``def:registry:outcome-axis`` | ``reg:axis`` | what an outcome axis is, and the Core's refusal to interpret one | binds; shipped | ``packages/assayer/docs/spec/data-model-registries.md`` |
| ``schema`` | ``schema:registry:axis-record`` | unminted | the axis registration record and its registration order | binds; **unbuilt** in its description field — the completed identity metadata entry (`entry:memory:audit-metadata`) did not cover the axis record | ``packages/assayer/docs/spec/data-model-registries.md`` |
| ``tab`` | ``tab:registry:axis-operations`` | unminted | add, remove and absent-value across the three surfaces | binds; shipped | ``packages/assayer/docs/spec/data-model-registries.md`` |
| ``alg`` | ``alg:registry:axis-registration`` | ``reg:axis-add`` | the axis registration protocol and its dimension count | binds; **unbuilt as specified** — the handler under-extends the standardisation vectors past one identity dimension, breaking the map's version invariant on the reference shape | ``packages/assayer/docs/spec/data-model-registries.md`` |
| ``alg`` | ``alg:registry:axis-deregistration`` | ``reg:axis-remove`` | the axis deregistration protocol | binds; shipped | ``packages/assayer/docs/spec/data-model-registries.md`` |
| ``req`` | ``req:registry:partial-outcome-reporting`` | ``reg:axis-partial`` | what a partial outcome report does and does not update | binds; shipped | ``packages/assayer/docs/spec/data-model-registries.md`` |
| ``tab`` | ``tab:registry:axis-scaling`` | ``tbl:reg:axis-scale`` | per-axis feature counts and per-label cost by axis count | explains | ``packages/assayer/docs/spec/data-model-registries.md`` |
| ``rem`` | ``rem:registry:axis-hibernation`` | unminted | axis hibernation, by reference to Sentinel hibernation | binds by reference; shipped | ``packages/assayer/docs/spec/data-model-registries.md`` |
| ``disc`` | ``disc:registry:spatial-policy`` | ``reg:axis-spatial`` | when to enable and disable spatial features on an axis | explains | ``packages/assayer/docs/spec/data-model-registries.md`` |
| ``sec`` | ``sec:registry:identity-dimension`` | unminted | division: the identity dimension registry | structural | ``packages/assayer/docs/spec/data-model-registries.md`` |
| ``def`` | ``def:registry:identity-dimension`` | ``reg:iddim`` | what an identity dimension is | explains | ``packages/assayer/docs/spec/data-model-registries.md`` |
| ``alg`` | ``alg:registry:dimension-registration`` | ``reg:iddim-add`` | the dimension registration protocol, with its anchor projection recompute | binds; shipped | ``packages/assayer/docs/spec/data-model-registries.md`` |
| ``alg`` | ``alg:registry:dimension-deregistration`` | ``reg:iddim-remove`` | the dimension deregistration protocol and its gathered partition | binds; shipped | ``packages/assayer/docs/spec/data-model-registries.md`` |
| ``sec`` | ``sec:registry:lifecycle-interaction`` | unminted | division: how the three registries interact | structural | ``packages/assayer/docs/spec/data-model-registries.md`` |
| ``inv`` | ``inv:registry:model-set-serialisation`` | ``reg:serial`` | which registry event touches which model set, under one publication protocol | binds; shipped | ``packages/assayer/docs/spec/data-model-registries.md`` |
| ``req`` | ``req:registry:compound-events`` | ``reg:compound`` | compound events process sequentially, and commute | binds; nothing in the code answers the commutation claim | ``packages/assayer/docs/spec/data-model-registries.md`` |

Citation spine. Every protocol here cites the Gaussian chapter's extension, marginalisation or regularised Schur; the Sentinel record cites the encoding contract and the domain width; the coverage states are cited by the extraction chapter's slot and by the alarm summary; the serialisation invariant is cited by the publication chapter's lifecycle guarantee; hibernation is cited by the limitation register's hibernation row and by the axis hibernation remark. The registration protocols are cited from the dimension map's rebuild triggers.

**Entry (Chapter 6, Per-Sentinel Structured Extraction)** · `entry:assayer:spec-outline-ch06`

The chapter head mints ``chap:spec:structured-extraction``. The soundest chapter in the register: every width the chapter states is the width the code produces, in the order the chapter states it (`entry:assayer:spec-sections-ch06`). The outline's job is to mint the tables so they stay checkable — six tables of independent width claims that a tool could verify against the constants they mirror, which is the standing rule (`goal:assayer:tools-over-discipline`) pointing at a specific check. One new environment: the extraction is computed and stored in single precision while every model consuming it is double, and the specification never says so.

| Kind | Head | Derived from | Scope | Flags | Document |
| --- | --- | --- | --- | --- | --- |
| ``setup`` | ``setup:extraction:from-batch-report`` | unminted | what a batch report offers and what the extraction compresses it to | binds; shipped | ``packages/assayer/docs/spec/data-model-extraction.md`` |
| ``tab`` | ``tab:extraction:chain-z-scores`` | ``ext:chain-z`` | the six chain z-score views over the per-axis maxima | binds; shipped | ``packages/assayer/docs/spec/data-model-extraction.md`` |
| ``rem`` | ``rem:extraction:order-statistic-bias`` | ``ext:max-bias`` | why the maximum view carries an order-statistics bias | explains | ``packages/assayer/docs/spec/data-model-extraction.md`` |
| ``rem`` | ``rem:extraction:batch-mean-extension`` | ``ext:batch-mean-opt`` | the optional batch-mean extension | explains | ``packages/assayer/docs/spec/data-model-extraction.md`` |
| ``tab`` | ``tab:extraction:chain-cusums`` | ``ext:chain-cusum`` | the three chain CUSUM views | binds; shipped | ``packages/assayer/docs/spec/data-model-extraction.md`` |
| ``tab`` | ``tab:extraction:chain-structure`` | ``ext:chain-struct`` | the eight chain structural features | binds; shipped | ``packages/assayer/docs/spec/data-model-extraction.md`` |
| ``def`` | ``def:extraction:report-staleness`` | ``ext:staleness`` | report staleness and its throughput proxy | binds; shipped | ``packages/assayer/docs/spec/data-model-extraction.md`` |
| ``tab`` | ``tab:extraction:coordination`` | ``ext:coord`` | the twelve coordination features and their absent-value defaults | binds; shipped | ``packages/assayer/docs/spec/data-model-extraction.md`` |
| ``tab`` | ``tab:extraction:ledger-features`` | ``ext:ledger-feats`` | the base and per-axis Ledger features and their conditions | binds; shipped | ``packages/assayer/docs/spec/data-model-extraction.md`` |
| ``tab`` | ``tab:extraction:batch-context`` | ``ext:batch-ctx`` | the batch context feature | binds; shipped | ``packages/assayer/docs/spec/data-model-extraction.md`` |
| ``def`` | ``def:extraction:slot`` | ``ext:slot`` | the total extraction width and the slot with its occupancy prefix | binds; the code's width-decomposition summary mislabels which group is which, though the total and the order are right | ``packages/assayer/docs/spec/data-model-extraction.md`` |
| ``conv`` | ``conv:extraction:offsets`` | ``ext:offsets`` | the offset convention, slot-relative and vector-relative | binds; shipped | ``packages/assayer/docs/spec/data-model-extraction.md`` |
| ``rem`` | ``rem:extraction:lean-extraction`` | ``dn:lean-extraction`` | why the extraction stays lean rather than carrying the report | explains | ``packages/assayer/docs/spec/data-model-extraction.md`` |
| ``rem`` | ``rem:extraction:single-precision`` | NEW | the extraction is single precision and every model consuming it is double | binds as a stated split; shipped and undocumented | ``packages/assayer/docs/spec/data-model-extraction.md`` |

Citation spine. The slot is the chapter's hub: it is cited by the feature vector's block layout, by the dimension map's per-Sentinel width, by the extraction reference appendix, and by the dimensional summary. The Ledger features table cites the Ledger's per-entry state and its attenuation analysis. The coverage states of the registry chapter are cited by the slot's occupancy prefix.

**Entry (Chapter 7, The Key Space Identity Layer)** · `entry:assayer:spec-outline-ch07`

The chapter head mints ``chap:spec:key-space-identity``. Sound in its data structures and unsound in its registration record: two per-cell state types match field for field, all six cross-dimension anchor positions hold, and three of the five declared budget parameters are absent from the code (`entry:assayer:spec-sections-ch07`). One folding decision: the chapter's lifecycle section points into the registry chapter rather than repeating it, and the outline keeps that — a pointer is a citation and names nothing new, so the section yields no environment. The competitive set's five non-invariants stay inside the set's definition rather than becoming five environments; they are what the definition is careful not to promise.

| Kind | Head | Derived from | Scope | Flags | Document |
| --- | --- | --- | --- | --- | --- |
| ``mot`` | ``mot:keyspace:purpose`` | ``id:purpose`` | what the identity layer does, and the three things it refuses to do | explains | ``packages/assayer/docs/spec/data-model-identity.md`` |
| ``schema`` | ``schema:keyspace:dimension-record`` | unminted | the dimension registration record and its five budget parameters | binds; shipped at (`entry:memory:identity-thresholds`) and (`entry:memory:audit-metadata`); the record additionally carries an observation-capacity field the specification does not declare | ``packages/assayer/docs/spec/data-model-identity.md`` |
| ``req`` | ``req:keyspace:encoding-contract`` | ``id:contract`` | the encoding contract applied to key spaces, and the cost of ignoring it | binds; **unbuilt** — the range check exists but its error is worded for the wrong registry | ``packages/assayer/docs/spec/data-model-identity.md`` |
| ``alg`` | ``alg:keyspace:observation-protocol`` | ``id:observe`` | the four-step observation protocol and its deferred graph processing | binds; shipped | ``packages/assayer/docs/spec/data-model-identity.md`` |
| ``def`` | ``def:keyspace:competitive-set`` | ``id:competitive`` | the competitive set as a depth-bounded entry set, and what it does not guarantee | binds | ``packages/assayer/docs/spec/data-model-identity.md`` |
| ``def`` | ``def:keyspace:competitive-indicators`` | ``id:indicators`` | one indicator per competitive cell, and the additive risk decomposition | binds; shipped | ``packages/assayer/docs/spec/data-model-identity.md`` |
| ``tab`` | ``tab:keyspace:dimension-features`` | ``id:dim-feats`` | the per-dimension feature block, structural and measurement | binds; shipped; the competitive index publishes the total-importance input at (`entry:memory:total-importance`) | ``packages/assayer/docs/spec/data-model-identity.md`` |
| ``tab`` | ``tab:keyspace:cross-dimension-features`` | ``id:cross-dim`` | the eight cross-dimension aggregates and their anchor positions | binds; shipped, all six index claims holding | ``packages/assayer/docs/spec/data-model-identity.md`` |
| ``tab`` | ``tab:keyspace:signal-cache`` | ``id:sigcache`` | the signal cache: capacity, entry size, eviction, memory | binds; shipped | ``packages/assayer/docs/spec/data-model-identity.md`` |
| ``tab`` | ``tab:keyspace:measurement-state`` | ``id:measure`` | per-range measurement state, its four components and their update rules | binds; **unbuilt** in its smoothing factor, which has no configuration surface | ``packages/assayer/docs/spec/data-model-identity.md`` |
| ``tab`` | ``tab:keyspace:outcome-state`` | ``id:outcome`` | per-cell outcome state, its six components and its two purposes | binds; shipped, including the pure decayed read at (`entry:memory:decay-view`) | ``packages/assayer/docs/spec/data-model-identity.md`` |
| ``tab`` | ``tab:keyspace:decay-rates`` | ``id:decay`` | the three decay mechanisms and their rates | binds; shipped | ``packages/assayer/docs/spec/data-model-identity.md`` |
| ``alg`` | ``alg:keyspace:cell-entry`` | ``id:cell-entry`` | competitive entry, its batching unit, and its cost | binds; shipped | ``packages/assayer/docs/spec/data-model-identity.md`` |
| ``alg`` | ``alg:keyspace:cell-exit`` | ``id:cell-exit`` | competitive exit, and what information it preserves | binds; shipped, including outcome retention and graph-eviction cleanup at (`entry:memory:warm-start`) | ``packages/assayer/docs/spec/data-model-identity.md`` |
| ``req`` | ``req:keyspace:host-duties`` | ``id:host-duties`` | what the host does at construction, assessment, label time, and ongoing, with the upstream pointer for where the budget parameters are defined | binds; the ongoing duty asks for encoding effectiveness *per dimension*, and the shipped reading is per Sentinel — the legacy per-dimension formula is retired at (`entry:assayer:defer-encoding-effectiveness`) and its mean-slot-weight successor does not descend to a dimension, so the duty now names a reading nothing produces and no deferral holds open; the review restored the dropped upstream pointer, without which a host is told to configure parameters and not told where they are defined | ``packages/assayer/docs/spec/data-model-identity.md`` |

Citation spine. The competitive set is the chapter's hub, cited by the indicators, the per-dimension features, both cell lifecycles, and the dimension map's competitive range. The cross-dimension features are cited by the anchor projection and by the aggregate block. The encoding contract of Chapter 2 is cited here and nowhere restated. The registry chapter's dimension protocols are cited from the lifecycle pointer, which mints nothing.

**Entry (Chapter 8, Host Signals)** · `entry:assayer:spec-outline-ch08`

The chapter head mints ``chap:spec:host-signals``. The shortest chapter and, per section, the most exactly implemented: both sections check out completely (`entry:assayer:spec-sections-ch08`). One new environment, for the thing the chapter leaves unstated and a reader cannot derive: the shapes have different widths, and the cyclic shape occupies two positions, so the block width is not computable from the chapter as it stands.

| Kind | Head | Derived from | Scope | Flags | Document |
| --- | --- | --- | --- | --- | --- |
| ``schema`` | ``schema:signal:shapes`` | ``sig:shapes`` | the seven signal shapes, their two persistence modes, and the declaration record | binds; shipped | ``packages/assayer/docs/spec/data-model-signals.md`` |
| ``rem`` | ``rem:signal:shape-widths`` | NEW | the shapes occupy different widths, and the cyclic shape occupies two | binds as a stated width rule; shipped and unstated | ``packages/assayer/docs/spec/data-model-signals.md`` |
| ``req`` | ``req:signal:schema-fixed`` | ``sig:schema`` | the schema is fixed at construction, and positions derive in declaration order | binds; shipped | ``packages/assayer/docs/spec/data-model-signals.md`` |

Citation spine. The schema is cited by the feature vector's block layout and by the dimension map's signal block; the shapes are cited by the host-duties environment of the identity chapter. Nothing in this chapter cites forward.

**Entry (Chapter 9, The Feature Vector)** · `entry:assayer:spec-outline-ch09`

The chapter head mints ``chap:spec:feature-vector``. The chapter that assembles Part II and the chapter with the most drift in the register: its arithmetic is impeccable and its structural claims are not (`entry:assayer:spec-sections-ch09`). Three decisions the register asks for by name. The interaction template definitions are minted individually, because two of the five have different semantics in the code and one has no implementation at all, and five separately citable definitions are what lets the migration name which one moved. The map's version-consistency claim is minted as an ``inv`` with its own head rather than left a bullet inside a prose list, because a path in the registry chapter breaks it and nothing detects that. And the feature-class assignment rule is minted new, replacing the authority the rewrite deleted and left the priors table floating without.

| Kind | Head | Derived from | Scope | Flags | Document |
| --- | --- | --- | --- | --- | --- |
| ``def`` | ``def:feature:vector-structure`` | ``phi:structure`` | the block layout of the feature vector, block by block | binds; shipped | ``packages/assayer/docs/spec/data-model-features.md`` |
| ``tab`` | ``tab:feature:dimension-formula`` | ``tbl:phi:dims`` | the dimension formula and its per-block terms | binds; shipped, exact at the reference configuration | ``packages/assayer/docs/spec/data-model-features.md`` |
| ``sec`` | ``sec:feature:aggregate`` | unminted | division: aggregate features | structural | ``packages/assayer/docs/spec/data-model-features.md`` |
| ``mot`` | ``mot:feature:aggregate-purpose`` | unminted | why aggregate features exist and what they summarise | explains | ``packages/assayer/docs/spec/data-model-features.md`` |
| ``tab`` | ``tab:feature:aggregate-block`` | ``phi:agg`` | the fifteen aggregate features | binds; shipped | ``packages/assayer/docs/spec/data-model-features.md`` |
| ``alg`` | ``alg:feature:concordance-recalibration`` | ``phi:concordance`` | recalibrating the concordance threshold: percentile, window, cadence | binds; **unbuilt as specified** — the window is half the size and aggregates one entry per assessment rather than one per reporting Sentinel | ``packages/assayer/docs/spec/data-model-features.md`` |
| ``sec`` | ``sec:feature:interactions`` | unminted | division: the interaction template system | structural | ``packages/assayer/docs/spec/data-model-features.md`` |
| ``mot`` | ``mot:feature:interaction-overview`` | unminted | interactions as pairwise products, and why the five types scale differently | explains | ``packages/assayer/docs/spec/data-model-features.md`` |
| ``def`` | ``def:feature:template-per-sentinel`` | ``phi:t1-per-sentinel`` | the per-Sentinel template | binds; shipped | ``packages/assayer/docs/spec/data-model-features.md`` |
| ``def`` | ``def:feature:template-named-pair`` | ``phi:t2-cross-explicit`` | the named-pair template, one feature for one declared pair | binds; **unbuilt** — the code's second type is the wildcard, so the named-pair template has no implementation | ``packages/assayer/docs/spec/data-model-features.md`` |
| ``def`` | ``def:feature:template-wildcard`` | ``phi:t3-cross-wildcard`` | the wildcard template over all pairs, and its quadratic scaling | binds; **shared-shape** — implemented as the code's second type, so the scaling warning attaches to the wrong mechanism today | ``packages/assayer/docs/spec/data-model-features.md`` |
| ``def`` | ``def:feature:template-aggregate`` | ``phi:t4-aggregate`` | the aggregate template | binds; shipped | ``packages/assayer/docs/spec/data-model-features.md`` |
| ``def`` | ``def:feature:template-competitive`` | ``phi:t5-competitive`` | the competitive template, one per identity dimension | binds; shipped | ``packages/assayer/docs/spec/data-model-features.md`` |
| ``tab`` | ``tab:feature:default-interaction-set`` | ``phi:defaults`` | the default set: five per-Sentinel, eight aggregate, one competitive per dimension | binds; **unbuilt** as a library default — it exists only as a test fixture | ``packages/assayer/docs/spec/data-model-features.md`` |
| ``tab`` | ``tab:feature:interaction-convergence`` | ``phi:cascade`` | co-occurrence rates and effective convergence per template type | explains; two rows attach to the wrong mechanisms while the type swap stands | ``packages/assayer/docs/spec/data-model-features.md`` |
| ``alg`` | ``alg:feature:lifecycle-interactions`` | unminted | what Sentinel and dimension lifecycle events do to the interaction set | binds; **shared-shape** — the prose describes the code's second type rather than the specification's | ``packages/assayer/docs/spec/data-model-features.md`` |
| ``eq`` | ``eq:feature:default-interaction-dimension`` | ``phi:pint`` | the default interaction dimension, derived | binds; holds at the reference configuration | ``packages/assayer/docs/spec/data-model-features.md`` |
| ``sec`` | ``sec:dimension:map`` | unminted | division: the dimension map | structural | ``packages/assayer/docs/spec/data-model-features.md`` |
| ``inv`` | ``inv:dimension:covering`` | ``map:covering`` | every position of the vector is covered by exactly one block | binds; shipped | ``packages/assayer/docs/spec/data-model-features.md`` |
| ``inv`` | ``inv:dimension:contiguity`` | ``map:contiguity`` | each block is contiguous | binds; shipped | ``packages/assayer/docs/spec/data-model-features.md`` |
| ``inv`` | ``inv:dimension:version-consistency`` | ``map:version`` | map, model parameters and standardisation statistics share one dimension at every published version | binds; **broken by a path in the registry chapter**, and nothing detects it — the register asks for this head expressly | ``packages/assayer/docs/spec/data-model-features.md`` |
| ``alg`` | ``alg:dimension:compaction`` | ``map:compaction`` | physical compaction, its triggers and its cost | binds; shipped | ``packages/assayer/docs/spec/data-model-features.md`` |
| ``def`` | ``def:dimension:layers`` | ``map:layers`` | block, slot and feature layers, and the derived slot offsets | binds; **shared-shape** — the width field holds the extraction width, and both derived offsets are recomputed at each use site | ``packages/assayer/docs/spec/data-model-features.md`` |
| ``conv`` | ``conv:dimension:axis-index`` | ``map:axis-index`` | the axis-index convention | binds; shipped | ``packages/assayer/docs/spec/data-model-features.md`` |
| ``alg`` | ``alg:dimension:compilation-pipeline`` | ``map:pipeline`` | declaration, resolution and compilation, with their three record types | binds; **unbuilt** in its third stage — the code has two, and marks both gaps itself | ``packages/assayer/docs/spec/data-model-features.md`` |
| ``def`` | ``def:dimension:competitive-range`` | ``map:competitive`` | the competitive range, its layout and its temporal ordering | binds; **unbuilt** in its ordering — cells take the caller's order, tie-break included | ``packages/assayer/docs/spec/data-model-features.md`` |
| ``def`` | ``def:dimension:anchor-projection`` | ``map:anchor-proj`` | the fifteen-feature anchor projection: thirteen gathers and two computed | binds; **shared-shape** — the code gathers all fifteen from different positions; see (`obs:assayer:s3-anchor-projection`) | ``packages/assayer/docs/spec/data-model-features.md`` |
| ``schema`` | ``schema:dimension:map-record`` | unminted | the dimension map record, its ordering requirement, and what it is authoritative for | binds; **shared-shape** — the record differs field by field, and carries an identity offset map the specification defines a convention for but does not declare | ``packages/assayer/docs/spec/data-model-features.md`` |
| ``sec`` | ``sec:standardisation:online`` | unminted | division: online standardisation | structural | ``packages/assayer/docs/spec/data-model-features.md`` |
| ``def`` | ``def:standardisation:purpose`` | ``std:purpose`` | why standardisation is needed for an isotropic prior | binds; shipped | ``packages/assayer/docs/spec/data-model-features.md`` |
| ``req`` | ``req:standardisation:timing`` | ``std:timing`` | where the statistics live and the three mechanisms that feed them | binds; **shared-shape** | ``packages/assayer/docs/spec/data-model-features.md`` |
| ``alg`` | ``alg:standardisation:label-time-procedure`` | ``std:procedure`` | the label-time procedure, its rate, its clip and its bias exclusion | binds; shipped | ``packages/assayer/docs/spec/data-model-features.md`` |
| ``tab`` | ``tab:standardisation:class-priors`` | ``tbl:std:priors`` | feature-class priors for nineteen classes | binds; shipped | ``packages/assayer/docs/spec/data-model-features.md`` |
| ``req`` | ``req:standardisation:class-assignment`` | NEW | which feature belongs to which class, and on whose authority | binds; restores the deleted authority the priors table now floats without | ``packages/assayer/docs/spec/data-model-features.md`` |
| ``alg`` | ``alg:standardisation:batch-initialisation`` | ``std:batch-init`` | batch initialisation from the first hundred assessments | binds; shipped | ``packages/assayer/docs/spec/data-model-features.md`` |
| ``bound`` | ``bound:standardisation:restandardisation`` | ``std:restand`` | the re-standardisation mismatch bound and its two components | explains | ``packages/assayer/docs/spec/data-model-features.md`` |
| ``req`` | ``req:standardisation:lifecycle-entries`` | unminted | what lifecycle events do to standardisation entries | binds; shipped | ``packages/assayer/docs/spec/data-model-features.md`` |
| ``alg`` | ``alg:standardisation:sentinel-bootstrap`` | ``std:bootstrap`` | the per-Sentinel bootstrap and its soft blend | binds; **unbuilt** — the hook exists, nothing outside a test calls it, and the two constants are documented as an unrelated procedure | ``packages/assayer/docs/spec/data-model-features.md`` |

Citation spine. The dimension formula is the chapter's hub and the document's: it is cited by the dimensional summary appendix, by the resource chapter's reference configuration, and by every registry protocol that changes a count. The five template definitions are cited by the default set, by the convergence table, and by the lifecycle algorithm. The version invariant is cited by the publication chapter's snapshot and by the axis registration protocol, which is the path that breaks it — the citation is the record of that. The class-priors table cites the new assignment requirement; the label-time procedure cites the update path of Part VI.

## Part III — The Core Models · `sec:assayer:spec-outline-part-3`

The Part head mints ``part:spec:core-models``. Three chapters, and the register found the core models implemented close to the specification with the four exceptions concentrated in one chapter and one theme — which observations train which model with what weight (`summ:assayer:spec-sections-b`). Every one of those four is recorded per environment below, because they are not a Part IV question: they sit inside chapters the code otherwise implements, and letting them inherit Part IV's disposition would hide four live divergences behind a vocabulary problem.

Structure. The heads below carry the Part itself and its chapters, and are claimed here because the outline tracks the specification's skeleton as well as its contents.

| Head | Document |
| --- | --- |
| ``part:spec:core-models`` | ``packages/assayer/docs/spec/core-models-risk.md`` |
| ``chap:spec:core-risk-models`` | ``packages/assayer/docs/spec/core-models-risk.md`` |
| ``chap:spec:axis-prediction`` | ``packages/assayer/docs/spec/core-models-axes.md`` |
| ``chap:spec:outcome-ledger`` | ``packages/assayer/docs/spec/core-models-ledger.md`` |

**Entry (Chapter 10, The Core Risk Models)** · `entry:assayer:spec-outline-ch10`

The chapter head mints ``chap:spec:core-risk-models``. The densest normative chapter and the most drifted outside Part IV. Two structural decisions. The blend's two persistence mechanisms get their own head, which resolves one of the seven dangling identities (`reg:assayer:spec-dangling-tags`): the concept is cited and its statement sits unminted in the following paragraph. And the Sherman–Morrison update is minted with its step 7 stated against the *prior* covariance-vector product, as the code computes it — the register found the section ambiguous where the code is not (`obs:assayer:s3-sm-step7`), and this is a specification defect the rewrite fixes rather than a drift it records.

| Kind | Head | Derived from | Scope | Flags | Document |
| --- | --- | --- | --- | --- | --- |
| ``def`` | ``def:risk:model-triple`` | ``model:triple`` | operational, sister and anchor, under one shared prior | binds; shipped | ``packages/assayer/docs/spec/core-models-risk.md`` |
| ``def`` | ``def:risk:anchor-model`` | ``model:anchor`` | the anchor as a fixed low-dimensional model over a stable projection | binds; shipped | ``packages/assayer/docs/spec/core-models-risk.md`` |
| ``prop`` | ``prop:risk:anchor-measurement-only`` | ``model:anchor-f13`` | the measurement-only anchor feature as Ledger-independent evidence | binds; **shared-shape** — the property survives at a different projection slot; see (`obs:assayer:s3-anchor-projection`) | ``packages/assayer/docs/spec/core-models-risk.md`` |
| ``def`` | ``def:risk:subspace-blend`` | ``model:blend`` | the subspace-restricted blend and its weight | binds; shipped | ``packages/assayer/docs/spec/core-models-risk.md`` |
| ``rem`` | ``rem:risk:blend-mechanisms`` | ``model:blend-mech`` | the two mechanisms by which the blend persists | explains; **resolves a dangling identity** — cited twice, minted nowhere | ``packages/assayer/docs/spec/core-models-risk.md`` |
| ``prop`` | ``prop:risk:blend-variance`` | ``model:blend-var`` | the blend's three-term variance | binds; shipped | ``packages/assayer/docs/spec/core-models-risk.md`` |
| ``dec`` | ``dec:risk:anchor-floor`` | ``dn:anchor-floor`` | why the anchor floors the blend rather than the blend flooring the anchor | binds as a recorded decision | ``packages/assayer/docs/spec/core-models-risk.md`` |
| ``def`` | ``def:risk:probability`` | ``model:phat`` | the risk probability derived from the blended estimate | binds; shipped | ``packages/assayer/docs/spec/core-models-risk.md`` |
| ``dec`` | ``dec:risk:evidence-only-uncertainty`` | unminted | the verdict's uncertainty made evidence-only along its own direction, and the borrowed share that rides beside it | binds as a recorded decision | ``packages/assayer/docs/spec/core-models-risk.md`` |
| ``def`` | ``def:risk:intervention-effectiveness`` | ``model:delta`` | the blended-minus-operational divergence, reported per assessment | binds; shipped | ``packages/assayer/docs/spec/core-models-risk.md`` |
| ``schema`` | ``schema:risk:basis`` | ``model:basis`` | the fourteen-field risk basis handed to the derivation | binds; shipped, field for field | ``packages/assayer/docs/spec/core-models-risk.md`` |
| ``sec`` | ``sec:platt:calibration`` | unminted | division: Platt calibration | structural | ``packages/assayer/docs/spec/core-models-risk.md`` |
| ``mot`` | ``mot:platt:purpose`` | unminted | what the calibration parameter is for | explains | ``packages/assayer/docs/spec/core-models-risk.md`` |
| ``def`` | ``def:platt:regimes`` | ``cal:regimes`` | the two regimes and the soft transition between them | binds; shipped | ``packages/assayer/docs/spec/core-models-risk.md`` |
| ``def`` | ``def:platt:objective`` | ``cal:objective`` | weighted cross-entropy with recency | binds; shipped | ``packages/assayer/docs/spec/core-models-risk.md`` |
| ``constr`` | ``constr:platt:buffer`` | ``cal:buffer`` | the calibration buffer: capacity, fields, regime assignment | binds; shipped | ``packages/assayer/docs/spec/core-models-risk.md`` |
| ``alg`` | ``alg:platt:fitting`` | ``cal:fit`` | golden-section search, its tolerance and its iteration cap | binds; shipped | ``packages/assayer/docs/spec/core-models-risk.md`` |
| ``req`` | ``req:platt:minimum-samples`` | ``cal:min`` | the minimum sample requirement per regime | binds; shipped | ``packages/assayer/docs/spec/core-models-risk.md`` |
| ``tab`` | ``tab:platt:refit-cadence`` | ``cal:cadence`` | the five refitting triggers | binds; shipped | ``packages/assayer/docs/spec/core-models-risk.md`` |
| ``def`` | ``def:platt:initial-value`` | unminted | the initial calibration parameter | binds; shipped | ``packages/assayer/docs/spec/core-models-risk.md`` |
| ``alg`` | ``alg:platt:drift-integration`` | ``cal:drift`` | the drift threshold and the accumulator reset | binds; shipped | ``packages/assayer/docs/spec/core-models-risk.md`` |
| ``disc`` | ``disc:platt:regime-transition`` | ``cal:transition`` | what a regime transition does during convergence | explains | ``packages/assayer/docs/spec/core-models-risk.md`` |
| ``sec`` | ``sec:eligibility:training`` | unminted | division: training eligibility | structural; added post-review — the rewrite wave found eligibility nesting under the Platt division, and eligibility is not calibration | ``packages/assayer/docs/spec/core-models-risk.md`` |
| ``tab`` | ``tab:eligibility:training`` | ``elig:table`` | which observation trains which model, and why | binds; **unbuilt as specified** — the predicate disagrees on two of six rows in opposite directions, training the sister on confounded evidence and starving it of the default population | ``packages/assayer/docs/spec/core-models-risk.md`` |
| ``conv`` | ``conv:eligibility:ground-truth`` | ``elig:ground-truth`` | what the ground-truth flag asserts and who sets it | binds; shipped | ``packages/assayer/docs/spec/core-models-risk.md`` |
| ``sec`` | ``sec:weighting:importance`` | unminted | division: importance weighting | structural | ``packages/assayer/docs/spec/core-models-risk.md`` |
| ``tab`` | ``tab:weighting:trackers`` | ``imp:trackers`` | two class-rate trackers, each decaying at its own model's rate | binds; **unbuilt as specified** — one shared rate ships, matching neither model, and the section says no new parameter is introduced | ``packages/assayer/docs/spec/core-models-risk.md`` |
| ``alg`` | ``alg:weighting:tracker-update`` | unminted | the EWMA update on the positive-valence indicator | binds; shipped | ``packages/assayer/docs/spec/core-models-risk.md`` |
| ``def`` | ``def:weighting:initial-value`` | ``imp:init`` | the trackers' initial value and its convergence horizon | binds; shipped | ``packages/assayer/docs/spec/core-models-risk.md`` |
| ``dec`` | ``dec:weighting:no-time-decay`` | ``imp:no-time`` | why the trackers carry no time-indexed decay | binds as a recorded decision | ``packages/assayer/docs/spec/core-models-risk.md`` |
| ``def`` | ``def:weighting:balancing-weights`` | ``imp:weights`` | the balancing weights, their ceiling, and their gradient-balance property | binds; **unbuilt as specified** — the code computes the odds ratio, so the effective class ratio is the square of the specified one and every tabulated row is wrong about the shipped system | ``packages/assayer/docs/spec/core-models-risk.md`` |
| ``disc`` | ``disc:weighting:leverage-interaction`` | ``imp:leverage`` | how the weights interact with the leverage bound | explains | ``packages/assayer/docs/spec/core-models-risk.md`` |
| ``def`` | ``def:risk:target`` | ``model:target`` | the risk target and its sign | binds; shipped; cites the valence convention | ``packages/assayer/docs/spec/core-models-risk.md`` |
| ``def`` | ``def:risk:compression-scale`` | ``model:kappa-v`` | the feature compression scale | binds; shipped | ``packages/assayer/docs/spec/core-models-risk.md`` |
| ``alg`` | ``alg:update:sherman-morrison`` | ``sm:update`` | the leverage-bounded update, seven steps, the mean stated against the prior product | binds; shipped; **fixes a specification defect** — (`obs:assayer:s3-sm-step7`) | ``packages/assayer/docs/spec/core-models-risk.md`` |
| ``prop`` | ``prop:update:leverage-bound`` | ``sm:leverage`` | what the leverage bound guarantees about a single observation | binds; shipped | ``packages/assayer/docs/spec/core-models-risk.md`` |
| ``tab`` | ``tab:risk:forgetting-rates`` | ``tbl:model:rates`` | label-indexed and time-indexed forgetting, per model | binds; shipped | ``packages/assayer/docs/spec/core-models-risk.md`` |

Citation spine. The model triple is cited by every environment of the chapter and by the axis chapter's per-axis model; the eligibility table is cited by the host contract's eligibility policy, by the importance trackers, and by the guarantees register; the risk basis is cited by the derivation signature, by the assessment structure, and by the boundary appendix; the update algorithm is cited by the label update path and by the two-level guarantee. The anchor model cites the dimension map's anchor projection and the identity chapter's cross-dimension features.

**Entry (Chapter 11, Outcome Axis Prediction Models)** · `entry:assayer:spec-outline-ch11`

The chapter head mints ``chap:spec:axis-prediction``. The shortest normative chapter and the cleanest against the code: no drift found in five sections (`entry:assayer:spec-sections-ch11`). Its neutrality statement is the boundary Part IV depends on and is minted in the landscape chapter, cited here.

| Kind | Head | Derived from | Scope | Flags | Document |
| --- | --- | --- | --- | --- | --- |
| ``def`` | ``def:axis:per-axis-model`` | ``ax:model`` | one Bayesian linear model per declared axis, uninterpreted by the Core | binds; shipped | ``packages/assayer/docs/spec/core-models-axes.md`` |
| ``def`` | ``def:axis:training-target`` | ``ax:target`` | the eligibility mode and the bounded training target | binds; shipped | ``packages/assayer/docs/spec/core-models-axes.md`` |
| ``def`` | ``def:axis:adaptive-compression`` | ``ax:kappa-shift`` | the adaptive compression scale and its adaptation rate | binds; shipped | ``packages/assayer/docs/spec/core-models-axes.md`` |
| ``tab`` | ``tab:axis:inference-outputs`` | ``ax:outputs`` | raw prediction, raw uncertainty, and the interval | binds; shipped, line for line | ``packages/assayer/docs/spec/core-models-axes.md`` |
| ``def`` | ``def:axis:prior-only-prediction`` | unminted | the degraded fallback: the axis prior through the same transform, added by the conformance repair campaign's ninth wave | binds; shipped | ``packages/assayer/docs/spec/core-models-axes.md`` |
| ``prop`` | ``prop:axis:cross-axis-prediction`` | ``ax:cross`` | cross-axis prediction emerges from shared features and is not modelled | explains | ``packages/assayer/docs/spec/core-models-axes.md`` |
| ``alg`` | ``alg:axis:lifecycle`` | unminted | registration extends and deregistration marginalises | binds; shipped, including event-local diagnostics on the lifecycle result and bounded health event | ``packages/assayer/docs/spec/core-models-axes.md`` |

Citation spine. The per-axis model cites the Gaussian chapter's extension and the registry chapter's axis protocols; the training target cites the valence convention and the eligibility table; the inference outputs are cited by the assessment structure. The neutrality guarantee of Part IV is cited here and minted there.

**Entry (Chapter 12, The Outcome Ledger)** · `entry:assayer:spec-outline-ch12`

The chapter head mints ``chap:spec:outcome-ledger``. Twenty-one sections dividing sharply into a normative half the code implements exactly and an analytic half that exists to say the Ledger is structurally uninformative for most cells (`entry:assayer:spec-sections-ch12`). One dangling identity resolves here: the dual-condition materiality result is cited and stated without a mint, and it is a result, so it takes a ``thm``. The laundering section keeps a discussion of its own and cites the limitation register rather than restating it — the exposure is a limitation, and limitations have one home.

| Kind | Head | Derived from | Scope | Flags | Document |
| --- | --- | --- | --- | --- | --- |
| ``def`` | ``def:ledger:purpose`` | ``led:purpose`` | the layered per-Sentinel outcome map, and what it does not inherit | binds; shipped | ``packages/assayer/docs/spec/core-models-ledger.md`` |
| ``summ`` | ``summ:ledger:value`` | ``led:value`` | the two conditions under which the Ledger is worth its cost | explains | ``packages/assayer/docs/spec/core-models-ledger.md`` |
| ``thm`` | ``thm:ledger:materiality`` | ``led:materiality`` | the dual-condition materiality result | binds as a result; **resolves a dangling identity** — cited, stated, never minted | ``packages/assayer/docs/spec/core-models-ledger.md`` |
| ``tab`` | ``tab:ledger:entry-state`` | ``led:entry`` | the nine fields of a per-entry outcome state | binds; shipped | ``packages/assayer/docs/spec/core-models-ledger.md`` |
| ``data`` | ``data:ledger:attenuation`` | ``led:attenuation`` | EWMA convergence and steady-state attenuation, tabulated | explains; its reporting is (`entry:assayer:defer-ledger-materiality`) | ``packages/assayer/docs/spec/core-models-ledger.md`` |
| ``alg`` | ``alg:ledger:all-layers-update`` | ``led:all-layers`` | all-layers routing, its eight substeps, and its cost | binds; shipped | ``packages/assayer/docs/spec/core-models-ledger.md`` |
| ``def`` | ``def:ledger:time-decay`` | ``led:decay`` | time-indexed decay, in its write-time and read-time modes | binds; shipped | ``packages/assayer/docs/spec/core-models-ledger.md`` |
| ``sec`` | ``sec:ledger:entry-lifecycle`` | unminted | division: entry lifecycle | structural | ``packages/assayer/docs/spec/core-models-ledger.md`` |
| ``alg`` | ``alg:ledger:entry-creation`` | ``led:create`` | creation with neutral state, and no inheritance | binds; shipped | ``packages/assayer/docs/spec/core-models-ledger.md`` |
| ``alg`` | ``alg:ledger:entry-deletion`` | ``led:delete`` | deletion after a threshold of consecutive absences, and no merge | binds; shipped; the code comment naming the threshold calls it hibernation and must migrate to this label with the word corrected — (`obs:assayer:s3-ledger-absent-naming`) | ``packages/assayer/docs/spec/core-models-ledger.md`` |
| ``alg`` | ``alg:ledger:garbage-collection`` | ``led:gc`` | the collection floor, the horizon, and the two admissible strategies | binds; shipped | ``packages/assayer/docs/spec/core-models-ledger.md`` |
| ``just`` | ``just:ledger:no-state-transfer`` | ``led:no-transfer`` | why no state transfers on creation or deletion | explains | ``packages/assayer/docs/spec/core-models-ledger.md`` |
| ``sec`` | ``sec:ledger:contamination`` | unminted | division: contamination and starvation | structural | ``packages/assayer/docs/spec/core-models-ledger.md`` |
| ``tab`` | ``tab:ledger:loop-timescales`` | ``led:loop-scales`` | the loop's four timescales and the binding constraint | explains | ``packages/assayer/docs/spec/core-models-ledger.md`` |
| ``def`` | ``def:ledger:starvation-score`` | ``led:starvation`` | the starvation-relief score | binds; shipped | ``packages/assayer/docs/spec/core-models-ledger.md`` |
| ``disc`` | ``disc:ledger:laundering`` | unminted | why symmetric decay is exploitable in the forgiving direction, and why the trade was taken | explains; cites the limitation register rather than restating the exposure | ``packages/assayer/docs/spec/core-models-ledger.md`` |
| ``def`` | ``def:ledger:root-semantics`` | ``led:root`` | the root entry as fallback, baseline and contamination anchor | binds; shipped | ``packages/assayer/docs/spec/core-models-ledger.md`` |
| ``tab`` | ``tab:ledger:versus-identity`` | ``led:vs-id`` | how Ledger outcome state relates to identity-layer outcome state | explains | ``packages/assayer/docs/spec/core-models-ledger.md`` |
| ``tab`` | ``tab:ledger:low-maturity-detection`` | ``led:fallback`` | the seven detection pathways that work at low Ledger maturity | binds in part; shipped | ``packages/assayer/docs/spec/core-models-ledger.md`` |
| ``dec`` | ``dec:ledger:no-adaptive-rate`` | ``dn:no-adaptive-rate`` | why the Ledger rate is fixed rather than adaptive | binds as a recorded decision | ``packages/assayer/docs/spec/core-models-ledger.md`` |

Citation spine. The per-entry state is cited by the extraction chapter's Ledger features, by the alarm summary's maturity predicate, and by the attenuation analysis; the time decay is cited by the temporal chapter's inventory, by the contamination loop of Part I, and by the publication chapter's read-time purity; the starvation score is cited by the guidance chapter's scoring criteria; the root semantics is cited by the routing environment of Part VI and by the collection algorithm. The materiality result is cited by the alarm summary and by the monitoring chapter's immature-cell metric — which is precisely the pair the register found satisfied by different work.

## Part IV — The Decision Landscape · `sec:assayer:spec-outline-part-4`

The Part head mints ``part:spec:decision-landscape``. This Part is where the audit's fourth requirement bites hardest, and the outline discharges it per environment rather than per Part. The chapter-level finding (`warn:assayer:drift-landscape-unimplemented`) is correct and too coarse: the section register separates seventeen shared rows, eleven that diverge in shape, and twenty that are unbuilt outright (`summ:assayer:spec-sections-b`), and the difference decides what the migration actually costs. Under (`dec:assayer:spec-authoritative`) no row here is a criticism of the specification; the status column records the shipped surface's distance from it, and (`reg:assayer:s4-cheap-burden`) is the register that says which of those distances is short.

One structural note the whole Part turns on. The challenge posterior arrives at the derivation boundary as a moment pair with its conjugate structure discarded (`obs:assayer:s4-posterior-flattened`), which is why dominance and credible intervals are blocked by a type rather than by missing mathematics. The outline mints the posterior as a two-variant signature and flags both consumers against it, so the review can check the single change that unblocks the largest share of the Part.

Structure. The heads below carry the Part itself and its chapters, and are claimed here because the outline tracks the specification's skeleton as well as its contents.

| Head | Document |
| --- | --- |
| ``part:spec:decision-landscape`` | ``packages/assayer/docs/spec/landscape-derivation.md`` |
| ``chap:spec:derivation-interface`` | ``packages/assayer/docs/spec/landscape-derivation.md`` |
| ``chap:spec:channel-policy`` | ``packages/assayer/docs/spec/landscape-policy.md`` |
| ``chap:spec:crossover-landscape`` | ``packages/assayer/docs/spec/landscape-crossovers.md`` |
| ``chap:spec:worked-landscapes`` | ``packages/assayer/docs/spec/landscape-worked.md`` |
| ``chap:spec:exploration-and-guidance`` | ``packages/assayer/docs/spec/landscape-guidance.md`` |

**Entry (Chapter 13, The Derivation Function Interface)** · `entry:assayer:spec-outline-ch13`

The chapter head mints ``chap:spec:derivation-interface``. The chapter that fixes Part IV's shape. Its three inputs diverge differently and are minted separately for exactly that reason: the risk basis is shared and is minted in Part III with a citation here, the channel policy is minted in Chapter 14, and the challenge posterior is minted here because the boundary is where its conjugate structure is lost.

| Kind | Head | Derived from | Scope | Flags | Document |
| --- | --- | --- | --- | --- | --- |
| ``sig`` | ``sig:landscape:derivation-function`` | ``land:iface`` | the derivation signature: determinism, statelessness, closed form | binds; **unbuilt** — the shipped entry point takes four arguments and returns a tag profile | ``packages/assayer/docs/spec/landscape-derivation.md`` |
| ``sig`` | ``sig:companion:posterior`` | ``land:posterior`` | the challenge posterior in two variants, conjugate and moment-matched | binds; **shared-shape** — only the moment pair crosses the boundary, which blocks two consumers by type | ``packages/assayer/docs/spec/landscape-derivation.md`` |
| ``sig`` | ``sig:landscape:output`` | ``land:output`` | the landscape: shared risk term, crossover vector, per-action regimes | binds; **unbuilt** | ``packages/assayer/docs/spec/landscape-derivation.md`` |
| ``inv`` | ``inv:landscape:presentation-free`` | ``inv:presentation-free`` | the primary output is a function of risk basis, policy and posterior alone | binds; **unbuilt** — the display configuration is a required argument of the shipped call; minted here rather than in the guarantees register because it is this chapter's own claim | ``packages/assayer/docs/spec/landscape-derivation.md`` |
| ``pf`` | ``pf:landscape:purity`` | unminted | the purity guarantee's five clauses, demonstrated by construction | binds; shipped, and carried by a dedicated test | ``packages/assayer/docs/spec/landscape-derivation.md`` |
| ``sig`` | ``sig:landscape:utilities`` | ``land:utils`` | the optimal-action and fragility utilities | binds; **unbuilt** — neither name occurs in the crate | ``packages/assayer/docs/spec/landscape-derivation.md`` |
| ``alg`` | ``alg:landscape:optimal-action`` | unminted | the optimal action from the upper envelope of the per-action cost curves | binds; **unbuilt** | ``packages/assayer/docs/spec/landscape-derivation.md`` |
| ``dec`` | ``dec:landscape:rendering-optional`` | ``land:render-opt`` | rendering is a layer over the landscape, not the landscape | binds as a recorded decision; **unbuilt** — (`obs:assayer:drift-rendering-not-optional`) | ``packages/assayer/docs/spec/landscape-derivation.md`` |

Citation spine. The signature cites the risk basis of Part III, the channel policy of Chapter 14, and the posterior minted here; the output is cited by every environment of Chapter 15, by the worked landscapes, and by the rendering contract of the appendix; the utilities are cited by the fragility definition and by the composition example. The purity demonstration cites the signature and is cited by the guarantees register.

**Entry (Chapter 14, Channel Policy)** · `entry:assayer:spec-outline-ch14`

The chapter head mints ``chap:spec:channel-policy``. The most nearly-shared chapter of Part IV, because it is mostly numbers and the numbers survived (`entry:assayer:spec-sections-ch14`). Two outline decisions. The sensitivity exponents are minted beside the rewards rather than inside them, which is the clean-up the register names. And the Companion's challenge-decay rate is *not* imported into the reward environment: it rides inside the shipped reward struct and belongs with the Companion, where this outline mints it (`obs:assayer:s4-gamma-qt-extra`).

| Kind | Head | Derived from | Scope | Flags | Document |
| --- | --- | --- | --- | --- | --- |
| ``def`` | ``def:channel:posture`` | ``pol:posture`` | posture as the host's cursor over the decision axis | binds; **unbuilt** — the shipped derivation renders the whole axis and takes no posture | ``packages/assayer/docs/spec/landscape-policy.md`` |
| ``tab`` | ``tab:channel:posture-constants`` | unminted | the four named posture constants | binds; **unbuilt** | ``packages/assayer/docs/spec/landscape-policy.md`` |
| ``prin`` | ``prin:channel:relativity`` | ``pol:relative`` | posture is meaningful only relative to the policy that reads it | explains | ``packages/assayer/docs/spec/landscape-policy.md`` |
| ``def`` | ``def:channel:actions`` | ``pol:actions`` | the ordered action set and what an action means to the Core | binds; shipped | ``packages/assayer/docs/spec/landscape-policy.md`` |
| ``cav`` | ``cav:channel:action-space-sizing`` | unminted | what narrowing the action space costs | discloses | ``packages/assayer/docs/spec/landscape-policy.md`` |
| ``tab`` | ``tab:channel:reward-parameters`` | unminted | the reward parameters and their defaults | binds; shipped, every default at its specified value | ``packages/assayer/docs/spec/landscape-policy.md`` |
| ``data`` | ``data:channel:reward-sensitivity`` | ``pol:sensitivity`` | crossover position against the reward ratio | explains; the arithmetic is performed by the shipped code | ``packages/assayer/docs/spec/landscape-policy.md`` |
| ``data`` | ``data:channel:challenge-interaction`` | ``pol:qc`` | how the challenge estimate moves the crossover | explains; shipped | ``packages/assayer/docs/spec/landscape-policy.md`` |
| ``rem`` | ``rem:channel:verify-costs`` | unminted | verify declared costs before reading the landscape | explains | ``packages/assayer/docs/spec/landscape-policy.md`` |
| ``prop`` | ``prop:channel:challenge-width`` | ``pol:challenge-width`` | the challenge regime width and its three widening levers | binds; **shared-shape** — width is implicit in the rendered magnitudes and never reported | ``packages/assayer/docs/spec/landscape-policy.md`` |
| ``def`` | ``def:channel:sensitivity-exponents`` | unminted | the two sensitivity exponents, beside the rewards | binds; **shared-shape** — they live inside the reward struct today | ``packages/assayer/docs/spec/landscape-policy.md`` |
| ``def`` | ``def:channel:neutral-zone`` | unminted | the neutral zone, accepted and inert | binds; shipped exactly as specified | ``packages/assayer/docs/spec/landscape-policy.md`` |

Citation spine. The action set and the reward parameters are cited by every cost and crossover environment of Chapter 15 and by all four worked landscapes; the posture is cited by the fragility definition, by the rendering placement, and by the posture-independence guarantee. The reward parameters cite the extended parameters of Chapter 15 rather than restating them.

**Entry (Chapter 15, The Crossover Landscape)** · `entry:assayer:spec-outline-ch15`

The chapter head mints ``chap:spec:crossover-landscape``. The chapter the rewrite created, and the one where the register's most useful result sits: the mathematics is already implemented and only the object is missing (`entry:assayer:spec-sections-ch15`). The outline mints the decomposition, the crossover record and the covariance diagonal as separate environments precisely because each is computed today and merely unexposed — separate heads are what let the burn-list work name them one at a time.

| Kind | Head | Derived from | Scope | Flags | Document |
| --- | --- | --- | --- | --- | --- |
| ``def`` | ``def:landscape:posture-sensitivity`` | ``land:sens`` | the posture-sensitivity functions and their exponential-on-logit form | binds; shipped | ``packages/assayer/docs/spec/landscape-crossovers.md`` |
| ``sec`` | ``sec:landscape:cost-model`` | unminted | division: the per-action cost model | structural | ``packages/assayer/docs/spec/landscape-crossovers.md`` |
| ``def`` | ``def:landscape:extended-rewards`` | unminted | the two extended reward parameters and their semantics | binds; shipped | ``packages/assayer/docs/spec/landscape-crossovers.md`` |
| ``tab`` | ``tab:landscape:action-costs`` | ``land:costs`` | per-action benign-class and adverse-class costs | binds; shipped | ``packages/assayer/docs/spec/landscape-crossovers.md`` |
| ``def`` | ``def:landscape:reward-differentials`` | ``land:diff`` | the differentials, generally and at four and three actions | binds; shipped | ``packages/assayer/docs/spec/landscape-crossovers.md`` |
| ``thm`` | ``thm:landscape:catching-forfeiture`` | ``land:forfeit`` | the catching-forfeiture threshold and its inversion depth | binds; **shared-shape** — the condition is detected and emitted as an infinite crossover, not a threshold | ``packages/assayer/docs/spec/landscape-crossovers.md`` |
| ``thm`` | ``thm:landscape:rigid-translation`` | ``land:rigid`` | the rigid-translation decomposition into a shared term and per-transition offsets | binds; **shared-shape** — computed as one logarithm, never separated | ``packages/assayer/docs/spec/landscape-crossovers.md`` |
| ``eq`` | ``eq:landscape:rigid-decomposition`` | ``eq:land:rigid`` | the decomposition, displayed | binds | ``packages/assayer/docs/spec/landscape-crossovers.md`` |
| ``def`` | ``def:landscape:action-crossover`` | ``land:crossover`` | the crossover record and its reported fields | binds; **unbuilt** — crossovers are private and carry three fields, not the reported set | ``packages/assayer/docs/spec/landscape-crossovers.md`` |
| ``eq`` | ``eq:landscape:crossover`` | ``eq:land:crossover`` | the crossover derivation, displayed | binds | ``packages/assayer/docs/spec/landscape-crossovers.md`` |
| ``thm`` | ``thm:landscape:dominance`` | ``land:dominance`` | dominance as an incomplete-beta probability, never enforced | binds; **unbuilt** — blocked by the flattened posterior, not by missing arithmetic | ``packages/assayer/docs/spec/landscape-crossovers.md`` |
| ``thm`` | ``thm:landscape:crossover-covariance`` | ``land:cov`` | the rank-2 covariance of the crossover vector | binds; **shared-shape** on the diagonal, **unbuilt** off it | ``packages/assayer/docs/spec/landscape-crossovers.md`` |
| ``prop`` | ``prop:landscape:width-variance`` | ``land:width-var`` | regime widths at zero risk variance, as a corollary | binds; **unbuilt**, and closed-form from quantities already computed | ``packages/assayer/docs/spec/landscape-crossovers.md`` |
| ``def`` | ``def:landscape:credible-intervals`` | ``land:interval`` | intervals by quantile push-through | binds; **unbuilt** — requires posterior quantiles the moment-only input cannot supply | ``packages/assayer/docs/spec/landscape-crossovers.md`` |
| ``alg`` | ``alg:landscape:computation-order`` | NEW | the six-step order fixing when crossovers, bandwidths, magnitudes, interior locations, boundary locations and classification tags are computed, and the acyclicity of the chain | binds; **NEW**, restoring a section deleted with the old derivation chapter, added at the review which accepted the second open question; **shared-shape** — the rendering computes in some order and the document fixes none, so the restoration closes a specification gap rather than commissioning code | ``packages/assayer/docs/spec/landscape-crossovers.md`` |
| ``def`` | ``def:landscape:outcome-neutrality`` | ``land:neutrality`` | axis predictions have no pathway into the derivation, stated concretely | binds; shipped; cites the neutrality guarantee | ``packages/assayer/docs/spec/landscape-crossovers.md`` |

Citation spine. The decomposition and the crossover derivation are cited by the covariance, the dominance result, the intervals and all four worked landscapes; the cost tables cite the channel policy's rewards and are cited by the rendering appendix's placement formula; the neutrality definition cites the axis chapter's inference outputs and the guarantees register. The catching-forfeiture threshold is cited by the dominated-tag treatment of the appendix.

**Entry (Chapter 16, Worked Landscapes)** · `entry:assayer:spec-outline-ch16`

The chapter head mints ``chap:spec:worked-landscapes``. Five sections, wholly expository in force and wholly unbuilt in substance, and the most useful Part IV artefact the register found: the four landscapes are a complete numeric acceptance oracle for Chapters 13 through 15, stated to three decimals, and each was chosen to isolate a property a first implementation would get wrong (`entry:assayer:spec-sections-ch16`). The outline records that they should be built as tests before the chapters they exercise are written, which is a scheduling claim the review should check against the rewrite plan.

| Kind | Head | Derived from | Scope | Flags | Document |
| --- | --- | --- | --- | --- | --- |
| ``setup`` | ``setup:landscape:worked-assumptions`` | unminted | the assumptions common to all four landscapes, and the derived constants | explains; **unbuilt** — the constants are computable from shipped code and never assembled | ``packages/assayer/docs/spec/landscape-worked.md`` |
| ``ex`` | ``ex:landscape:low-risk`` | ``ex:low`` | a low-risk known entity, and how its fragility reads | explains; **unbuilt** | ``packages/assayer/docs/spec/landscape-worked.md`` |
| ``ex`` | ``ex:landscape:high-risk`` | ``ex:high`` | a high-risk new entity; the regime table is unchanged from the first | explains; **unbuilt**; isolates the risk-independence of the regime table | ``packages/assayer/docs/spec/landscape-worked.md`` |
| ``ex`` | ``ex:landscape:medium-risk`` | ``ex:mid`` | medium risk under high uncertainty, and how the label request reads | explains; **unbuilt** | ``packages/assayer/docs/spec/landscape-worked.md`` |
| ``ex`` | ``ex:landscape:three-action`` | ``ex:three`` | a three-action channel, the widening of the challenge regime, and the per-transition crossover sensitivities at defaults | explains; **unbuilt**; quantifies the action-space sizing caveat; carries the worked sensitivity numbers the review moved here from the deleted table, as an acceptance oracle rather than a derivation | ``packages/assayer/docs/spec/landscape-worked.md`` |
| ``summ`` | ``summ:landscape:design-summary`` | unminted | eleven design aspects and how the landscape treats each | explains; a restatement register, and a generation target | ``packages/assayer/docs/spec/landscape-worked.md`` |

Citation spine. Every example cites the derivation output, the crossover derivation, the covariance and the channel policy; the design summary cites whatever it restates and is cited by nothing, which is why it is a generation target rather than authored prose (`obs:assayer:spec-restatement-chapters`).

**Entry (Chapter 17, Exploration and Label Guidance)** · `entry:assayer:spec-outline-ch17`

The chapter head mints ``chap:spec:exploration-and-guidance``. Part IV's sharpest deliberate replacement sits beside one of its two fully-shipped surfaces (`entry:assayer:spec-sections-ch17`). The outline keeps them separate and says so in the flags, because the migration is not additive: the shipped exploration surface is the thing fragility retired, so hosts consuming it will get a different quantity under the same position (`obs:assayer:s4-fragility-replacement`).

| Kind | Head | Derived from | Scope | Flags | Document |
| --- | --- | --- | --- | --- | --- |
| ``sec`` | ``sec:fragility:decision`` | unminted | division: decision fragility | structural | ``packages/assayer/docs/spec/landscape-guidance.md`` |
| ``def`` | ``def:fragility:definition`` | ``frag:def`` | the modal action, the flip probability, and the two evidence shares | binds; **unbuilt** — what ships is the entropy gauge this definition replaced | ``packages/assayer/docs/spec/landscape-guidance.md`` |
| ``tab`` | ``tab:fragility:interpretation`` | ``frag:interp`` | the interpretation bands, and fragility as a value-of-information proxy | explains; **unbuilt** | ``packages/assayer/docs/spec/landscape-guidance.md`` |
| ``sec`` | ``sec:guidance:core`` | unminted | division: Core label guidance | structural | ``packages/assayer/docs/spec/landscape-guidance.md`` |
| ``sig`` | ``sig:guidance:interface`` | ``guide:iface`` | budget, parameters, the three request lists, and the candidate record | binds; shipped, down to the field names | ``packages/assayer/docs/spec/landscape-guidance.md`` |
| ``def`` | ``def:guidance:risk-informative`` | ``guide:risk`` | the risk-informative scoring criterion | binds; shipped | ``packages/assayer/docs/spec/landscape-guidance.md`` |
| ``def`` | ``def:guidance:investigation`` | ``guide:invest`` | the investigation scoring criterion | binds; shipped | ``packages/assayer/docs/spec/landscape-guidance.md`` |
| ``def`` | ``def:guidance:starvation`` | ``guide:starv`` | the starvation scoring criterion | binds; shipped, factor for factor | ``packages/assayer/docs/spec/landscape-guidance.md`` |
| ``dec`` | ``dec:guidance:no-deduplication`` | ``guide:dedup`` | independent lists with a cross-membership field, rather than deduplication | binds as a recorded decision; shipped | ``packages/assayer/docs/spec/landscape-guidance.md`` |
| ``ex`` | ``ex:fragility:composition`` | ``frag:compose`` | composing fragility with Core guidance | explains; **unbuilt** — the pattern calls two absent utilities | ``packages/assayer/docs/spec/landscape-guidance.md`` |

Citation spine. The fragility definition cites the landscape output, the covariance and the utilities; the guidance interface cites the Ledger's starvation score and the identity chapter's competitive set; the composition example cites both halves and is the only place they meet.

## Part V — The Companion Tracker · `sec:assayer:spec-outline-part-5`

The Part head mints ``part:spec:companion-tracker``. One chapter, and the closest agreement between specification and code anywhere in Parts V–VIII with three exceptions, one of them sharp (`entry:assayer:spec-sections-ch18`).

Structure. The heads below carry the Part itself and its chapters, and are claimed here because the outline tracks the specification's skeleton as well as its contents.

| Head | Document |
| --- | --- |
| ``part:spec:companion-tracker`` | ``packages/assayer/docs/spec/companion-tracking.md`` |
| ``chap:spec:challenge-effectiveness`` | ``packages/assayer/docs/spec/companion-tracking.md`` |

**Entry (Chapter 18, Challenge Effectiveness Tracking)** · `entry:assayer:spec-outline-ch18`

The chapter head mints ``chap:spec:challenge-effectiveness``. The two design notes get recorded decisions of their own, because both fix a choice the code has to honour — the blend-toward-prior *form* of the decay and its rate — and the register found the read half of that decay unimplemented, so the form is exactly what a reader needs to be able to cite. The Companion's own challenge-decay rate is minted here rather than travelling inside the channel policy, per the register's recommendation (`obs:assayer:s4-gamma-qt-extra`).

| Kind | Head | Derived from | Scope | Flags | Document |
| --- | --- | --- | --- | --- | --- |
| ``def`` | ``def:companion:purpose`` | ``comp:purpose`` | the catch probability, and its two channels into the landscape | explains | ``packages/assayer/docs/spec/companion-tracking.md`` |
| ``def`` | ``def:companion:state`` | ``comp:state`` | pseudo-counts, last-update time, configuration, override | binds; **shared-shape** — the code flattens the nested configuration onto the state | ``packages/assayer/docs/spec/companion-tracking.md`` |
| ``def`` | ``def:companion:prior`` | ``comp:prior`` | the uninformative default prior, and the five-prior guidance table | binds; shipped | ``packages/assayer/docs/spec/companion-tracking.md`` |
| ``alg`` | ``alg:companion:inference`` | ``comp:estimate`` | point estimate, variance, effective count, decayed at read time | binds; **unbuilt** in its read-time decay — reads return stored counts, so a tracker starved of labels never regresses toward its prior in what the host sees | ``packages/assayer/docs/spec/companion-tracking.md`` |
| ``alg`` | ``alg:companion:update`` | ``comp:update`` | the conjugate update on a contributing label, and the three contribution conditions | binds; shipped | ``packages/assayer/docs/spec/companion-tracking.md`` |
| ``data`` | ``data:companion:contributing-rate`` | ``comp:rate`` | the contributing-label rate and the targeting-quality table | explains | ``packages/assayer/docs/spec/companion-tracking.md`` |
| ``alg`` | ``alg:companion:decay`` | ``comp:decay`` | pseudo-count decay as a convex blend toward the prior | binds; shipped on the write path only | ``packages/assayer/docs/spec/companion-tracking.md`` |
| ``dec`` | ``dec:companion:decay-form`` | ``dn:decay-form`` | why the decay blends toward the prior rather than scaling the counts | binds as a recorded decision | ``packages/assayer/docs/spec/companion-tracking.md`` |
| ``dec`` | ``dec:companion:decay-rate`` | ``dn:decay-rate`` | why the rate is what it is | binds as a recorded decision | ``packages/assayer/docs/spec/companion-tracking.md`` |
| ``def`` | ``def:companion:challenge-decay`` | ``cfg:gamma-qt`` | the challenge-effectiveness decay rate, sited with the Companion | binds; **shared-shape** — it ships inside the channel policy's reward struct | ``packages/assayer/docs/spec/companion-tracking.md`` |
| ``alg`` | ``alg:companion:override`` | ``comp:override`` | the host override and its default variance, with accumulation continuing beneath | binds; shipped | ``packages/assayer/docs/spec/companion-tracking.md`` |
| ``alg`` | ``alg:companion:injection`` | ``comp:inject`` | evidence injection, its non-negativity, and its per-count ceiling | binds; shipped, under a differently-named error | ``packages/assayer/docs/spec/companion-tracking.md`` |
| ``tab`` | ``tab:companion:health`` | ``comp:health`` | the Companion health report | binds; **unbuilt** in six of its ten rows — four ship, and the most actionable diagnostic is absent | ``packages/assayer/docs/spec/companion-tracking.md`` |
| ``bound`` | ``bound:companion:convergence`` | ``comp:converge`` | convergence time under four targeting regimes | explains | ``packages/assayer/docs/spec/companion-tracking.md`` |
| ``inv`` | ``inv:companion:boundary`` | ``comp:boundary`` | none to the Core, one to the derivation, one to the host | binds; shipped | ``packages/assayer/docs/spec/companion-tracking.md`` |
| ``req`` | ``req:companion:replacement-trait`` | ``comp:trait`` | the replacement surface, with its defaulting posterior method | binds; **unbuilt** — (`obs:assayer:drift-companion-trait`) | ``packages/assayer/docs/spec/companion-tracking.md`` |

Citation spine. The state and the prior are cited by the inference and the update; the inference is cited by the derivation's posterior input and by the health report; the boundary invariant is cited by the feed-forward guarantee and by the boundary appendix. The configuration table of Part VIII cites the four parameters here rather than restating them.

## Part VI — The Operational Cycle · `sec:assayer:spec-outline-part-6`

The Part head mints ``part:spec:operational-cycle``. Five chapters and the operational contract. This Part carries the register's most serious finding outside Part IV — label-time reconstruction populates three of six sources, so every model trains on a vector that differs systematically from the vector its prediction was computed from (`summ:assayer:spec-sections-c`) — and the outline records it as unbuilt work inside a chapter the code otherwise implements step for step. It is not a Part IV question and must not inherit Part IV's disposition.

Structure. The heads below carry the Part itself and its chapters, and are claimed here because the outline tracks the specification's skeleton as well as its contents.

| Head | Document |
| --- | --- |
| ``part:spec:operational-cycle`` | ``packages/assayer/docs/spec/operations-runtime.md`` |
| ``chap:spec:host-contract`` | ``packages/assayer/docs/spec/operations-runtime.md`` |
| ``chap:spec:assessment-interface`` | ``packages/assayer/docs/spec/operations-assessment.md`` |
| ``chap:spec:label-pipeline`` | ``packages/assayer/docs/spec/operations-labels.md`` |
| ``chap:spec:concurrency`` | ``packages/assayer/docs/spec/operations-concurrency.md`` |
| ``chap:spec:temporal-governance`` | ``packages/assayer/docs/spec/operations-temporal.md`` |

**Entry (Chapter 19, Runtime Declarations and the Host Contract)** · `entry:assayer:spec-outline-ch19`

The chapter head mints ``chap:spec:host-contract``. Eight sections fixing what the host declares and promises (`entry:assayer:spec-sections-ch19`). The valence convention is cited from Chapter 1 and not restated, which removes one of the document's several duplications.

| Kind | Head | Derived from | Scope | Flags | Document |
| --- | --- | --- | --- | --- | --- |
| ``req`` | ``req:host:construction`` | ``host:construct`` | the three construction categories, and the absence of any channel concept | binds; shipped | ``packages/assayer/docs/spec/operations-runtime.md`` |
| ``req`` | ``req:host:eligibility-policy`` | ``host:elig`` | the eligibility policy and its default | binds; **unbuilt** — the field exists, defaults correctly, and is read by nothing, so the shipped system runs permanently on the non-default branch | ``packages/assayer/docs/spec/operations-runtime.md`` |
| ``req`` | ``req:host:axis-registration`` | unminted | axis registration as a lifecycle event the host initiates | binds; shipped | ``packages/assayer/docs/spec/operations-runtime.md`` |
| ``req`` | ``req:host:schema-declaration`` | unminted | the signal schema is declared once at construction | binds; shipped | ``packages/assayer/docs/spec/operations-runtime.md`` |
| ``tab`` | ``tab:host:label-reporting`` | ``host:label`` | the five required label fields, and the two routed elsewhere | binds; shipped | ``packages/assayer/docs/spec/operations-runtime.md`` |
| ``alg`` | ``alg:host:pre-seeding`` | ``host:preseed`` | pre-seeding through the normal label path, and its constraints | binds; **shared-shape** — the entry type carries a channel the Core is specified not to have | ``packages/assayer/docs/spec/operations-runtime.md`` |
| ``rem`` | ``rem:host:severity-discrimination`` | ``host:severity`` | combining risk and severity, as deployment guidance | explains | ``packages/assayer/docs/spec/operations-runtime.md`` |

Citation spine. The eligibility policy cites the training-eligibility table of Part III and is cited by the configuration table; the label-reporting contract is cited by the label interface and by the update path; the schema declaration cites the signal chapter. The valence convention of Chapter 1 is cited here.

**Entry (Chapter 20, The Assessment Interface)** · `entry:assayer:spec-outline-ch20`

The chapter head mints ``chap:spec:assessment-interface``. The chapter carrying the Core's principal output type. Two divergences sat inside otherwise sound sections and both got their own head so neither could hide: the compact health snapshot shipped five of nine fields without its join key, and the ledger-immature flag tested whether any Ledger entry exists rather than the maturity predicate, which — since the root entry always exists — made it almost never true. The second is repaired, the flag now taking both specified conditions against the entry the Ledger's own read routing reaches (`entry:assayer:spec-sections-ch20`).

| Kind | Head | Derived from | Scope | Flags | Document |
| --- | --- | --- | --- | --- | --- |
| ``alg`` | ``alg:runtime:report-reception`` | ``run:report-rx`` | reception as atomic replacement, serialised per Sentinel | binds; shipped; the shipped signature additionally returns an acknowledgement | ``packages/assayer/docs/spec/operations-assessment.md`` |
| ``req`` | ``req:runtime:assessment-interface`` | ``run:assess`` | a batch of contexts to a vector of assessments, with no channel | binds; shipped | ``packages/assayer/docs/spec/operations-assessment.md`` |
| ``alg`` | ``alg:runtime:assessment-pipeline`` | ``run:pipeline`` | the pipeline, step by step, per-request independent | binds; shipped | ``packages/assayer/docs/spec/operations-assessment.md`` |
| ``inv`` | ``inv:runtime:enumerated-writes`` | ``run:writes`` | the assessment path writes exactly this set and nothing else | binds; shipped | ``packages/assayer/docs/spec/operations-assessment.md`` |
| ``tab`` | ``tab:runtime:numeric-checkpoints`` | unminted — absorbed from the retired degradation record's placement, wave three of the repair campaign | the six numbered numeric readings from the boundary inward, and each one's response | binds; shipped | ``packages/assayer/docs/spec/operations-assessment.md`` |
| ``def`` | ``def:runtime:time-correction`` | ``run:time-corr`` | uncertainty corrected for snapshot age at read time | binds; shipped | ``packages/assayer/docs/spec/operations-assessment.md`` |
| ``def`` | ``def:runtime:report-index`` | ``run:report-index`` | the current report index, rebuilt atomically, deepest covering entry winning | binds; shipped | ``packages/assayer/docs/spec/operations-assessment.md`` |
| ``def`` | ``def:runtime:ledger-index`` | ``run:ledger-index`` | deepest-entry read routing and all-layers write routing | binds; shipped | ``packages/assayer/docs/spec/operations-assessment.md`` |
| ``alg`` | ``alg:runtime:extraction-routing`` | unminted | how extraction reaches Ledger features under pure read-time decay | binds; shipped, all three lookups, the Ledger one taking the Ledger's own read routing | ``packages/assayer/docs/spec/operations-assessment.md`` |
| ``alg`` | ``alg:runtime:cell-set-maintenance`` | ``run:cellset`` | appearances, absences and disappearances, with no event classification | binds; shipped | ``packages/assayer/docs/spec/operations-assessment.md`` |
| ``bound`` | ``bound:runtime:routing-cost`` | unminted | the cost of routing, per assessment | explains | ``packages/assayer/docs/spec/operations-assessment.md`` |
| ``bound`` | ``bound:runtime:ledger-memory`` | unminted | Ledger memory under decay and collection | explains | ``packages/assayer/docs/spec/operations-assessment.md`` |
| ``schema`` | ``schema:output:assessment`` | ``out:assessment`` | the assessment structure: assessment, basis, predictions, per-Sentinel | binds; shipped | ``packages/assayer/docs/spec/operations-assessment.md`` |
| ``schema`` | ``schema:output:health-snapshot`` | unminted | the compact health snapshot embedded in every assessment, and its join key | binds; **unbuilt** in four of nine fields, the join key among them, so the forensic design the snapshot exists for is unrealised | ``packages/assayer/docs/spec/operations-assessment.md`` |
| ``def`` | ``def:runtime:alarm-summary`` | ``run:alarm`` | the per-Sentinel alarm summary, its composite, and its maturity predicate | binds; structure and predicate both shipped — the predicate was an entry-existence test and is now the specified disjunction over the entry the extraction routing reaches, repaired at (`entry:assayer:wl-ledger-root-only-read`); it stays adjacent to but distinct from the health-count entry (`entry:memory:immature-count`), which takes one of its two arms over every entry | ``packages/assayer/docs/spec/operations-assessment.md`` |
| ``ex`` | ``ex:runtime:composition`` | ``run:compose`` | composing the three components, with replay and label feedback | explains | ``packages/assayer/docs/spec/operations-assessment.md`` |

Citation spine. The assessment structure is the chapter's hub: it cites the risk basis, the axis inference outputs, the alarm summary and the health snapshot, and is cited by the output restatements and by the boundary appendix. The alarm summary cites the Ledger's materiality result and the coverage states. The report index is cited by the publication chapter's staleness table.

**Entry (Chapter 21, The Label Interface and Learning Pipeline)** · `entry:assayer:spec-outline-ch21`

The chapter head mints ``chap:spec:label-pipeline``. The chapter carried the lane's most serious finding. Reconstruction is minted as its own environment with the six sources enumerated in it, and the pending entry as another, because the register found the gap from both sides: at the review three of six sources were populated, and the four stored components the missing steps would read were themselves absent from the struct (`summ:assayer:spec-sections-c`). Two environments, two fixes, one of which had to land before the other. Both have landed: the pending entry stores the four components and the reconstruction fills all six sources.

| Kind | Head | Derived from | Scope | Flags | Document |
| --- | --- | --- | --- | --- | --- |
| ``req`` | ``req:runtime:label-interface`` | unminted | the label interface, serialised internally, never blocking assessment | binds; **shared-shape** — the arguments arrive as one payload and the return is a typed acknowledgement | ``packages/assayer/docs/spec/operations-labels.md`` |
| ``alg`` | ``alg:runtime:update-path`` | ``run:update-path`` | the label update path, step by step and in order | binds; shipped with two fixed scoped helpers beneath the single steward, completing (`entry:assayer:defer-label-parallelism`) | ``packages/assayer/docs/spec/operations-labels.md`` |
| ``alg`` | ``alg:runtime:identity-outcome-update`` | ``run:id-update`` | the identity outcome update across all competitive cells | binds; shipped | ``packages/assayer/docs/spec/operations-labels.md`` |
| ``bound`` | ``bound:runtime:label-cost`` | unminted | per-label cost, cited rather than duplicated | explains | ``packages/assayer/docs/spec/operations-labels.md`` |
| ``def`` | ``def:runtime:pending-entry`` | ``run:buffer`` | the pending entry's fields, with no channel and no landscape | binds; shipped — the four fields the review found missing (active Sentinels, reporting Sentinels, entity base features, entity axis features) are stored and cross the journal boundary | ``packages/assayer/docs/spec/operations-labels.md`` |
| ``def`` | ``def:runtime:storage-precision`` | unminted | single-precision storage with upcast at label time | binds; shipped | ``packages/assayer/docs/spec/operations-labels.md`` |
| ``req`` | ``req:runtime:buffer-capacity`` | unminted | capacity computed from rate and latency, and eviction of the oldest | binds; **shared-shape** — capacity is fixed rather than computed; eviction reporting is shipped at (`entry:retention:eviction-counter`) and (`entry:retention:eviction-visibility`) | ``packages/assayer/docs/spec/operations-labels.md`` |
| ``alg`` | ``alg:runtime:reconstruction`` | ``run:reconstruct`` | reconstructing the feature vector at label time, from six numbered sources | binds; shipped — all six sources are filled, and the reconstruction reproduces the assessment-time raw vector exactly when the working state is unchanged. At the review it was **unbuilt in more than half**: the identity, axis, aggregate and interaction blocks were left at zero and only two of the six numbered sources were written, not the three the register reported, since the first source's identity base features were not written either | ``packages/assayer/docs/spec/operations-labels.md`` |

Citation spine. Reconstruction cites the pending entry, the dimension map, the standardisation procedure and the slot; the update path cites the Sherman–Morrison update, the eligibility table and the importance weights; the label interface cites the label-reporting contract of Chapter 19. The pending entry is cited by the publication chapter's buffer environment and by the resource chapter's memory summary.

**Entry (Chapter 22, Concurrency)** · `entry:assayer:spec-outline-ch22`

The chapter head mints ``chap:spec:concurrency``. The soundest chapter in the region and the model the register says the rewrite should copy: a structural specification and its implementation corresponding exactly (`entry:assayer:spec-sections-ch22`). Three of the twenty-nine formal properties are stated here and mint here, in the guarantees area, per the rule the last chapter's entry sets out.

| Kind | Head | Derived from | Scope | Flags | Document |
| --- | --- | --- | --- | --- | --- |
| ``inv`` | ``inv:guarantee:non-blocking`` | ``conc:principle`` ``inv:nonblock`` | no call blocks on another; staleness is acceptable and delay is not | binds; shipped | ``packages/assayer/docs/spec/operations-concurrency.md`` |
| ``def`` | ``def:publication:model-snapshot`` | ``conc:snapshot`` | the published model snapshot and its version consistency | binds; shipped, field for field and in order | ``packages/assayer/docs/spec/operations-concurrency.md`` |
| ``req`` | ``req:publication:interval`` | unminted | the publication interval as an amortisation knob | binds; **unbuilt** — no Core configuration carries it and the label path publishes unconditionally, so neither the amortisation nor the staleness widening is reachable; the review notes a field of the same name exists on the blend-statistics configuration and is a different quantity, which the rewrite must not mistake for this one | ``packages/assayer/docs/spec/operations-concurrency.md`` |
| ``alg`` | ``alg:publication:identity-draining`` | ``run:id-drain`` | deferred identity observations drain off the assessment path, bounded and drop-oldest | binds; shipped | ``packages/assayer/docs/spec/operations-concurrency.md`` |
| ``inv`` | ``inv:publication:ledger-concurrency`` | unminted | Ledger reads decay purely; Ledger writes happen only at label time | binds; shipped | ``packages/assayer/docs/spec/operations-concurrency.md`` |
| ``tab`` | ``tab:publication:staleness-bounds`` | ``tbl:conc:stale`` | the staleness bound of each published component | binds; shipped | ``packages/assayer/docs/spec/operations-concurrency.md`` |
| ``inv`` | ``inv:guarantee:staleness`` | ``inv:staleness`` | every read is stale by at most its component's bound | binds; shipped | ``packages/assayer/docs/spec/operations-concurrency.md`` |
| ``inv`` | ``inv:guarantee:lifecycle-publication`` | ``conc:lifecycle`` ``inv:lifecycle-pub`` | a lifecycle event is never visible mid-call, and never drains, pauses or delays | binds; shipped | ``packages/assayer/docs/spec/operations-concurrency.md`` |
| ``req`` | ``req:publication:pending-buffer`` | unminted | written by assessment, read by label processing, without contention | binds; shipped | ``packages/assayer/docs/spec/operations-concurrency.md`` |
| ``req`` | ``req:publication:label-queue`` | ``conc:queue`` | a bounded FIFO queue off the assessment path, with refused losses counted | binds; capacity, live depth and the cumulative full-queue loss counter ship; the concurrency record governs refuse-newest overflow | ``packages/assayer/docs/spec/operations-concurrency.md`` |
| ``req`` | ``req:publication:report-reception`` | unminted | cell-set maintenance runs inside the reception call, before the index swap | binds; shipped | ``packages/assayer/docs/spec/operations-concurrency.md`` |
| ``tab`` | ``tab:publication:tiers`` | ``tbl:conc:tiers`` | the concurrency tiers and the four whole-call guarantees | binds; shipped | ``packages/assayer/docs/spec/operations-concurrency.md`` |

Citation spine. The snapshot is cited by the assessment pipeline, the staleness table and the dimension map's version invariant; the tiers table cites every component it classifies; the non-blocking guarantee is cited by the label queue, the draining algorithm and the guarantees register. The pending buffer cites Chapter 21's entry definition.

**Entry (Chapter 23, Temporal Governance)** · `entry:assayer:spec-outline-ch23`

The chapter head mints ``chap:spec:temporal-governance``. The cleanest verification in the region: the decay inventory is a table of rates and every Core rate in it is the rate the code carries (`entry:assayer:spec-sections-ch23`). One qualification recorded per row: the identity outcome rate the inventory presents as configuration is pipeline-internal, so a host cannot set what the chapter says is settable.

| Kind | Head | Derived from | Scope | Flags | Document |
| --- | --- | --- | --- | --- | --- |
| ``def`` | ``def:temporal:two-mechanisms`` | ``time:two`` | label-indexed and time-indexed decay, independent and multiplicative | binds; shipped | ``packages/assayer/docs/spec/operations-temporal.md`` |
| ``tab`` | ``tab:temporal:decay-inventory`` | ``tbl:time:inventory`` | every decaying quantity, its mechanism and its rate | binds; every Core row shipped; the identity outcome rate is not host-configurable as tabulated | ``packages/assayer/docs/spec/operations-temporal.md`` |
| ``alg`` | ``alg:temporal:lazy-application`` | ``time:lazy`` | decay applied lazily at access, with the read-time correction | binds; shipped | ``packages/assayer/docs/spec/operations-temporal.md`` |
| ``tab`` | ``tab:temporal:component-rates`` | unminted | the per-component rate table | binds; shipped | ``packages/assayer/docs/spec/operations-temporal.md`` |
| ``thm`` | ``thm:temporal:ledger-decay-bound`` | ``time:ledger`` | Ledger time-decay as the contamination-loop breaker, and its starvation bound | binds; shipped | ``packages/assayer/docs/spec/operations-temporal.md`` |

Citation spine. The inventory is cited by every chapter that owns a decaying quantity — the Ledger, the identity layer, the Companion, standardisation and the risk models — and cites none of them, which is the correct direction for an inventory. The lazy mechanism is cited by the Ledger's read-time decay and the Companion's inference.

## Part VII — Properties and Analysis · `sec:assayer:spec-outline-part-7`

The Part head mints ``part:spec:properties-and-analysis``. Four chapters, three of them overwhelmingly analytic and one — the monitoring chapter — the place where the deferral register does most of its work and, in five environments, does not. The outline's rule here comes straight from the register's warning: where an environment is deferred it cites the deferral, and where it is absent it says so and names the deferral it is *not* (`summ:assayer:spec-sections-c`).

Structure. The heads below carry the Part itself and its chapters, and are claimed here because the outline tracks the specification's skeleton as well as its contents.

| Head | Document |
| --- | --- |
| ``part:spec:properties-and-analysis`` | ``packages/assayer/docs/spec/analysis-warmup.md`` |
| ``chap:spec:initialisation-and-warmup`` | ``packages/assayer/docs/spec/analysis-warmup.md`` |
| ``chap:spec:convergence-and-resources`` | ``packages/assayer/docs/spec/analysis-resources.md`` |
| ``chap:spec:detection-analysis`` | ``packages/assayer/docs/spec/analysis-detection.md`` |
| ``chap:spec:health-monitoring`` | ``packages/assayer/docs/spec/analysis-monitoring.md`` |

**Entry (Chapter 24, System Initialisation and Warm-Up)** · `entry:assayer:spec-outline-ch24`

The chapter head mints ``chap:spec:initialisation-and-warmup``. Overwhelmingly analytic; the two bootstraps bind and the rest illustrates (`entry:assayer:spec-sections-ch24`). The six warm-up stages fold into one table rather than taking six heads: they are described as approximate and are not transitions the code implements, so six citation targets would promise a precision the chapter disclaims.

Two demotions, recorded at the review because the earlier revision took them silently. The section register marks the milestone table and the pre-calibration section as partly binding, and the audit's chapter partition (`tab:assayer:spec-partition`) names the milestone table among the things that bind; this outline makes both explain. The milestone table restates thresholds that bind at the calibration cadence, the Companion convergence bound and the interaction convergence table, and a restatement that cites its sources binds nothing of its own — that is the same rule Chapter 29 is disposed by. The pre-calibration section tells an operator what not to trust before the first refit, which is deployment awareness rather than a constraint on the implementation. Both demotions narrow what the rewrite must satisfy, so both are stated here rather than left to be discovered: a force this outline lowers is a promise it withdraws.

| Kind | Head | Derived from | Scope | Flags | Document |
| --- | --- | --- | --- | --- | --- |
| ``tab`` | ``tab:warmup:stages`` | ``warm:s1-coldstart`` … ``warm:s6-steady`` | the six warm-up stages from cold start to steady state | explains; folds six identities | ``packages/assayer/docs/spec/analysis-warmup.md`` |
| ``alg`` | ``alg:warmup:concordance-bootstrap`` | unminted | the concordance threshold's initial value, first calibration and steady-state window | binds; **unbuilt as specified** — the window is a quarter of its size and aggregates per assessment rather than per reporting Sentinel | ``packages/assayer/docs/spec/analysis-warmup.md`` |
| ``alg`` | ``alg:warmup:standardisation-bootstrap`` | unminted | batch initialisation and the per-Sentinel bootstrap, at warm-up | binds; batch half shipped, per-Sentinel half unbuilt | ``packages/assayer/docs/spec/analysis-warmup.md`` |
| ``ex`` | ``ex:warmup:profiles`` | ``warm:profiles`` | worked convergence at three deployment profiles | explains | ``packages/assayer/docs/spec/analysis-warmup.md`` |
| ``tab`` | ``tab:warmup:milestones`` | ``tbl:warm:milestones`` | the Core, Companion and system milestones | explains; restates thresholds fixed elsewhere and cites them | ``packages/assayer/docs/spec/analysis-warmup.md`` |
| ``rem`` | ``rem:warmup:pre-calibration`` | ``warm:precal`` | what is and is not trustworthy before the first refit | explains | ``packages/assayer/docs/spec/analysis-warmup.md`` |
| ``ex`` | ``ex:warmup:recovery`` | ``warm:recovery`` | recovery after atrophy, after a transient event, and after a policy change | explains | ``packages/assayer/docs/spec/analysis-warmup.md`` |

Citation spine. Both bootstraps cite their owning chapters — concordance to the aggregate block, standardisation to the standardisation section — and are cited by the configuration tables. The milestone table cites the calibration cadence, the Companion convergence bound and the interaction convergence table.

**Entry (Chapter 25, Convergence and Resource Bounds)** · `entry:assayer:spec-outline-ch25`

The chapter head mints ``chap:spec:convergence-and-resources``. Expository save the reference configuration, which binds and checks (`entry:assayer:spec-sections-ch25`). The cost tables are cited rather than duplicated from Chapter 21, and the outline keeps that discipline: the tables mint here and the operational chapters cite them.

| Kind | Head | Derived from | Scope | Flags | Document |
| --- | --- | --- | --- | --- | --- |
| ``bound`` | ``bound:resource:convergence-budget`` | ``warm:budget`` | the convergence budget and its rule of thumb, at four deployment sizes | explains | ``packages/assayer/docs/spec/analysis-resources.md`` |
| ``prop`` | ``prop:resource:incremental-addition`` | ``res:incremental`` | adding a Sentinel or axis after convergence is incremental, not a reconvergence | explains | ``packages/assayer/docs/spec/analysis-resources.md`` |
| ``tab`` | ``tab:resource:assessment-cost`` | ``tbl:res:assess`` | per-assessment cost | explains | ``packages/assayer/docs/spec/analysis-resources.md`` |
| ``tab`` | ``tab:resource:label-cost`` | ``tbl:res:label`` | per-label cost | explains | ``packages/assayer/docs/spec/analysis-resources.md`` |
| ``tab`` | ``tab:resource:memory`` | ``tbl:res:memory`` | the memory summary, dominated by the pending buffer | explains; the dominant figure matches the shipped default that Chapter 21 specifies as computed | ``packages/assayer/docs/spec/analysis-resources.md`` |
| ``tab`` | ``tab:resource:reference-configuration`` | ``res:reference`` | the reference configuration and its dimensional construction | binds; shipped, exact | ``packages/assayer/docs/spec/analysis-resources.md`` |

Citation spine. The reference configuration is cited by the dimensional summary appendix, by the cost tables, by the warm-up worked example and by the dimension formula; it cites the feature vector's blocks. The cost tables cite the update path and the assessment pipeline.

**Entry (Chapter 26, Detection Analysis)** · `entry:assayer:spec-outline-ch26`

The chapter head mints ``chap:spec:detection-analysis``. Argumentative throughout with two exceptions — the divergence scope and the per-channel reporting rule, which are the two sections the register marks normative — and the argument is worth carrying: the admission that the avoidance case covers the measurement layer and not the memory layer is what makes the laundering limitation honest (`entry:assayer:spec-sections-ch26`).

| Kind | Head | Derived from | Scope | Flags | Document |
| --- | --- | --- | --- | --- | --- |
| ``tab`` | ``tab:detection:stack`` | ``tbl:det:stack`` | the detection layers, their mechanisms and their timescales | explains | ``packages/assayer/docs/spec/analysis-detection.md`` |
| ``disc`` | ``disc:detection:compound`` | ``det:discard`` | cross-Sentinel compound detection, and what the architecture discards | explains | ``packages/assayer/docs/spec/analysis-detection.md`` |
| ``disc`` | ``disc:detection:avoidance-dilemma`` | ``det:dilemma`` | the multi-objective avoidance dilemma, and the measurement–memory distinction | explains; load-bearing behind the laundering limitation | ``packages/assayer/docs/spec/analysis-detection.md`` |
| ``tab`` | ``tab:detection:convergence-window`` | ``det:window`` | the cross-Sentinel convergence window by template type | explains | ``packages/assayer/docs/spec/analysis-detection.md`` |
| ``def`` | ``def:detection:divergence-scope`` | ``model:delta-scope`` | what intervention effectiveness measures and where it may be read | binds; shipped; its scope caveat directs hosts to a metric that does not exist | ``packages/assayer/docs/spec/analysis-detection.md`` |
| ``req`` | ``req:detection:per-channel-reporting`` | unminted | the Core reports one figure per assessment and no channel dimension | binds; shipped | ``packages/assayer/docs/spec/analysis-detection.md`` |

Citation spine. The divergence scope cites the intervention-effectiveness definition of Part III and the resolution-utilisation metric of Chapter 27 — which is the citation that will dangle until that metric exists, and is therefore worth having. The avoidance dilemma is cited by the laundering limitation.

**Entry (Chapter 27, Health Monitoring)** · `entry:assayer:spec-outline-ch27`

The chapter head mints ``chap:spec:health-monitoring``. The largest chapter of Part VII and the one whose environments carry the most deferral citations. Three outline decisions. The synchronisation error's definition is minted here, once, and the Gaussian chapter's monitoring procedure cites it — the register found the code computing a diagonal maximum where the specification means a Frobenius norm, and a single definition site is how that stops recurring. The empirical coverage diagnostic was minted while wholly unimplemented, because three environments depended on it: the compact health snapshot, the discrimination-visibility guarantee, and the honest-notes emphasis. It now ships, computed at each calibration refit and reported, with its inflation factor on the compact health snapshot. The two formerly unregistered absences now ship from live producer state and retain the heads that made the gaps explicit.

| Kind | Head | Derived from | Scope | Flags | Document |
| --- | --- | --- | --- | --- | --- |
| ``alg`` | ``alg:monitoring:drift-cusums`` | ``hm:drift`` | the two one-sided drift recursions and their noise allowance | binds; shipped as specified. At the review the allowance **drifted fivefold**; the default is now the tabulated value | ``packages/assayer/docs/spec/analysis-monitoring.md`` |
| ``tab`` | ``tab:monitoring:drift-diagnostics`` | unminted | mean absolute residual, steps since reset, residual sign | binds; shipped | ``packages/assayer/docs/spec/analysis-monitoring.md`` |
| ``tab`` | ``tab:monitoring:drift-resets`` | unminted | the four reset mechanisms and the flag they feed | binds; shipped, including the configured compact flag | ``packages/assayer/docs/spec/analysis-monitoring.md`` |
| ``alg`` | ``alg:monitoring:auc`` | ``hm:auc`` | aggregate, per-regime and recent AUC, with the instability flag | binds; computation and both sample gates shipped as specified. At the review both gates **drifted**; the recent window is now the specified fixed count rather than a proportion | ``packages/assayer/docs/spec/analysis-monitoring.md`` |
| ``def`` | ``def:monitoring:top-decile-lift`` | ``hm:lift`` | top-decile lift over the buffer positive rate | binds; shipped | ``packages/assayer/docs/spec/analysis-monitoring.md`` |
| ``def`` | ``def:monitoring:axis-correlation`` | unminted | per-axis correlation between stored predictions and reported values | binds; shipped | ``packages/assayer/docs/spec/analysis-monitoring.md`` |
| ``tab`` | ``tab:monitoring:interpretation`` | ``tbl:hm:interp`` | interpretation bands over the discrimination metrics | explains | ``packages/assayer/docs/spec/analysis-monitoring.md`` |
| ``alg`` | ``alg:monitoring:empirical-coverage`` | ``hm:coverage`` | standardised residuals, two coverage fractions, and the inflation factor | binds; shipped — computed at each calibration refit and reported, with the inflation factor also carried on the compact health snapshot. At the review it was **unbuilt**, with zero occurrences in the package and no covering deferral despite three dependent environments | ``packages/assayer/docs/spec/analysis-monitoring.md`` |
| ``def`` | ``def:monitoring:synchronisation-error`` | unminted | the synchronisation error, defined once, with its threshold | binds; shipped as the Frobenius residual per model and aggregate maximum, published beside its prior-induced component and its residual, and it is the residual the cadence acts on | ``packages/assayer/docs/spec/analysis-monitoring.md`` |
| ``def`` | ``def:monitoring:condition-number`` | unminted | the cheap diagonal ratio, the condition number it bounds, and the growth arm the first feeds | binds; shipped, the cheap ratio read relatively against the value the last rebuild recorded rather than against a fixed threshold the replenishment floor would satisfy permanently | ``packages/assayer/docs/spec/analysis-monitoring.md`` |
| ``req`` | ``req:monitoring:conditioning-bound-named`` | unminted | the diagonal ratio is published as a bound and never as the condition number | binds; shipped as two separately named readings, the measured one absent until a model's first recomputation | ``packages/assayer/docs/spec/analysis-monitoring.md`` |
| ``req`` | ``req:monitoring:replenishment-floor`` | unminted | dimensions at the prior floor, counted and reported | binds; shipped | ``packages/assayer/docs/spec/analysis-monitoring.md`` |
| ``alg`` | ``alg:monitoring:per-entity-concordance`` | unminted | per-entity concordance and its flagging rule | binds; shipped with bounded least-recently-observed eviction | ``packages/assayer/docs/spec/analysis-monitoring.md`` |
| ``def`` | ``def:monitoring:alarm-outcome-cusums`` | unminted | the two-directional alarm-outcome disagreement accumulators | binds; shipped per Sentinel and axis | ``packages/assayer/docs/spec/analysis-monitoring.md`` |
| ``def`` | ``def:monitoring:feature-stable-drift`` | ``hm:integrity`` | feature-stable outcome drift, as a conjunction | binds; shipped | ``packages/assayer/docs/spec/analysis-monitoring.md`` |
| ``def`` | ``def:monitoring:quantile-concentration`` | unminted | quantile error concentration, as a max-to-mean ratio | binds; shipped | ``packages/assayer/docs/spec/analysis-monitoring.md`` |
| ``cav`` | ``cav:monitoring:integrity-scope`` | ``hm:integrity-scope`` | the four blind spots of label-integrity monitoring | discloses | ``packages/assayer/docs/spec/analysis-monitoring.md`` |
| ``def`` | ``def:monitoring:maturity-coverage`` | ``hm:maturity`` | measurement maturity coverage and its reading threshold | binds; shipped from current report-index cell state to full health under the configured threshold | ``packages/assayer/docs/spec/analysis-monitoring.md`` |
| ``def`` | ``def:monitoring:resolution-utilisation`` | ``hm:resutil`` | the resolution-utilisation ratio, its adequacy threshold and its reading threshold | binds; shipped from current Ledger evidence to full health under the configured threshold | ``packages/assayer/docs/spec/analysis-monitoring.md`` |
| ``def`` | ``def:monitoring:feedback-latency`` | ``hm:latency`` | end-to-end feedback latency and its decomposition | binds; shipped, all four stages smoothed at publication and the report stage reported as a lower bound | ``packages/assayer/docs/spec/analysis-monitoring.md`` |
| ``def`` | ``def:monitoring:encoding-effectiveness`` | ``hm:encoding`` | per-Sentinel weight mass as the mean absolute operational weight over its own slot, and what a reading of weight does not say about what the outcome depends on | binds; shipped and reported | ``packages/assayer/docs/spec/analysis-monitoring.md`` |
| ``def`` | ``def:monitoring:dimension-informativeness`` | unminted | per-dimension weight mass as the same mean absolute operational weight restricted to an identity dimension's own block, with the composition law that holds, the readings that inherit it, and the block families that make it hold | binds; shipped and reported per registered dimension, minting the quantity the host duty at (`req:keyspace:host-duties`) had been citing at a granularity nothing produced | ``packages/assayer/docs/spec/analysis-monitoring.md`` |
| ``def`` | ``def:monitoring:slot-association`` | unminted | the screening reading over any entity's own block: mean absolute feature-outcome correlation over the coordinates that varied, calibrated against the analytic null floor, with its window, its licensed evidence, its alert band and its composition law | binds; shipped per Sentinel and per registered dimension, accumulated prequentially at label time, and carrying the early-warning role the weight-mass reading was measured unable to hold | ``packages/assayer/docs/spec/analysis-monitoring.md`` |
| ``def`` | ``def:monitoring:slot-contribution`` | unminted | the confirming reading over the same block: the loss cost of removing it from the score, normalised by the score's own loss, with its alert band and the reason no aggregate of it composes | binds; shipped per Sentinel and per registered dimension from the same accumulation site, and the only carrier the retirement claim has | ``packages/assayer/docs/spec/analysis-monitoring.md`` |
| ``def`` | ``def:monitoring:immature-cells`` | unminted | the count of immature Ledger cells against the materiality threshold | binds; shipped at (`entry:memory:immature-count`), which is one arm of the alarm summary's per-assessment flag taken over a different population, and is not that flag's fleet-wide total | ``packages/assayer/docs/spec/analysis-monitoring.md`` |
| ``def`` | ``def:monitoring:ledger-value`` | ``hm:value`` | Ledger value realisation and the attenuation-limited fraction | binds; **unbuilt** — (`entry:assayer:defer-ledger-materiality`) | ``packages/assayer/docs/spec/analysis-monitoring.md`` |
| ``tab`` | ``tab:monitoring:dimension-health`` | unminted | the per-dimension identity health metrics | binds; all six rows ship — the register carried that claim while the coverage fraction and the per-cell weight mass had no producer, and both are now produced, surfaced and exported | ``packages/assayer/docs/spec/analysis-monitoring.md`` |
| ``tab`` | ``tab:monitoring:observability-summary`` | unminted | scope, meaningfulness and alert threshold per observability metric | explains; its shipped census is derived from cited producer-path tests and its two unavailable quantities cite their owner STOPs | ``packages/assayer/docs/spec/analysis-monitoring.md`` |
| ``inv`` | ``inv:monitoring:report-only`` | unminted | health monitoring reports and never acts | binds; shipped | ``packages/assayer/docs/spec/analysis-monitoring.md`` |
| ``tab`` | ``tab:monitoring:standardisation-transition`` | unminted | the cold ramp's phase and accepted count, on the compact snapshot and the full report | binds; **unbuilt** — the accepted count exists nowhere in the package and the phase is inferred from the variance floor rather than stored | ``packages/assayer/docs/spec/analysis-monitoring.md`` |
| ``scenario`` | ``scenario:monitoring:atrophy`` | ``hm:scenario-atrophy`` | restriction-induced spatial atrophy, end to end | explains | ``packages/assayer/docs/spec/analysis-monitoring.md`` |
| ``scenario`` | ``scenario:monitoring:displacement`` | ``hm:scenario-displacement`` | source displacement, end to end | explains | ``packages/assayer/docs/spec/analysis-monitoring.md`` |
| ``scenario`` | ``scenario:monitoring:starvation`` | ``hm:scenario-starvation`` | the restriction-induced starvation loop | explains | ``packages/assayer/docs/spec/analysis-monitoring.md`` |
| ``rem`` | ``rem:monitoring:operator-message`` | unminted | what an operator should take from the scenarios | explains | ``packages/assayer/docs/spec/analysis-monitoring.md`` |

Citation spine. The empirical coverage diagnostic is the chapter's most-cited environment and the most consequential: it is cited by the health snapshot of Chapter 20, by the discrimination-visibility guarantee, and by the honest-notes appendix. The synchronisation error is cited by the Gaussian chapter's monitor. The resolution-utilisation ratio is cited by the detection chapter's scope definition. Every deferred environment cites its owner STOP, and every delivered row above states that it ships.

## Part VIII — Reference · `sec:assayer:spec-outline-part-8`

The Part head mints ``part:spec:reference``. Three chapters, of which one is the configuration surface, one is pure restatement, and one is the document's two registers of record.

Structure. The heads below carry the Part itself and its chapters, and are claimed here because the outline tracks the specification's skeleton as well as its contents.

| Head | Document |
| --- | --- |
| ``part:spec:reference`` | ``packages/assayer/docs/spec/reference-configuration.md`` |
| ``chap:spec:configuration`` | ``packages/assayer/docs/spec/reference-configuration.md`` |
| ``chap:spec:output-structures`` | ``packages/assayer/docs/spec/reference-outputs.md`` |
| ``chap:spec:guarantees-and-limitations`` | ``packages/assayer/docs/spec/reference-guarantees.md`` |

**Entry (Chapter 28, Configuration)** · `entry:assayer:spec-outline-ch28`

The chapter head mints ``chap:spec:configuration``. Nineteen tables and roughly ninety parameters, of which fifty-nine were read against their code sites and fifty-three matched (`entry:assayer:spec-sections-ch28`). The outline mints one environment per table and folds the twenty individually-identified parameters into the table that carries them, because a parameter is a row and a row is not an environment. Two flags apply to whole tables rather than rows: the monitoring table, whose rows now all have a public, validated configuration surface while two consumers remain under their named definition STOPs. The other flag applies to the derivation table, whose subject is the unbuilt Part IV surface. And the rendering configuration table is minted new, which resolves the last of the seven dangling identities: the specification cites it twice and Chapter 28 does not contain it, while the type is real in the implementation (`obs:assayer:drift-rendering-config`).

The chapter also carries the register's clearest tooling argument: fifty-nine defaults read by hand and six wrong is not a ratio that survives the next revision, and the checking is entirely mechanical (`goal:assayer:tools-over-discipline`). The outline records the need; building the check is not this entry's work.

| Kind | Head | Derived from | Scope | Flags | Document |
| --- | --- | --- | --- | --- | --- |
| ``sec`` | ``sec:config:core`` | unminted | division: Core configuration | structural | ``packages/assayer/docs/spec/reference-configuration.md`` |
| ``conv`` | ``conv:config:parameter-citation`` | unminted | how a parameter is cited from the chapter that uses it | binds | ``packages/assayer/docs/spec/reference-configuration.md`` |
| ``tab`` | ``tab:config:risk-model`` | ``tbl:cfg:core`` | the core risk model parameters | binds; shipped, all rows | ``packages/assayer/docs/spec/reference-configuration.md`` |
| ``tab`` | ``tab:config:eligibility`` | ``tbl:cfg:elig`` | the eligibility policy parameter | binds; default shipped and never consumed | ``packages/assayer/docs/spec/reference-configuration.md`` |
| ``tab`` | ``tab:config:axis`` | ``tbl:cfg:axis`` | the per-axis defaults | binds; shipped | ``packages/assayer/docs/spec/reference-configuration.md`` |
| ``tab`` | ``tab:config:temporal`` | ``tbl:cfg:time`` | the time-indexed rates | binds; shipped | ``packages/assayer/docs/spec/reference-configuration.md`` |
| ``tab`` | ``tab:config:identity`` | ``tbl:cfg:id`` | the identity layer parameters | binds; mixed — several are host-supplied as the table says | ``packages/assayer/docs/spec/reference-configuration.md`` |
| ``tab`` | ``tab:config:ledger`` | ``tbl:cfg:led`` | the Ledger parameters | binds; shipped | ``packages/assayer/docs/spec/reference-configuration.md`` |
| ``tab`` | ``tab:config:calibration`` | ``tbl:cfg:cal`` | the calibration parameters | binds; shipped, all rows | ``packages/assayer/docs/spec/reference-configuration.md`` |
| ``tab`` | ``tab:config:standardisation`` | ``tbl:cfg:std`` | the standardisation parameters | binds; values shipped, and the code's field documentation misdescribes two of them | ``packages/assayer/docs/spec/reference-configuration.md`` |
| ``tab`` | ``tab:config:concurrency`` | ``tbl:cfg:conc`` | the concurrency parameters | binds; **publication interval unbuilt** | ``packages/assayer/docs/spec/reference-configuration.md`` |
| ``def`` | ``def:config:reference-peak-request-rate-100-per-second`` | unminted | the reference peak request rate, 100 per second, the R of the pending-buffer capacity computation | binds; shipped | ``packages/assayer/docs/spec/reference-configuration.md`` |
| ``def`` | ``def:config:reference-label-latency-3600-seconds`` | unminted | the reference median label latency, 3,600 seconds, the L of the pending-buffer capacity computation | binds; shipped | ``packages/assayer/docs/spec/reference-configuration.md`` |
| ``tab`` | ``tab:config:pending-buffer`` | ``tbl:cfg:buffer`` | the pending buffer parameters | binds; shipped | ``packages/assayer/docs/spec/reference-configuration.md`` |
| ``tab`` | ``tab:config:extraction`` | ``tbl:cfg:ext`` | the extraction parameters | binds; **chain-length normalisation drifts eight-fold**, rescaling two features every slot carries — one chain-structure and one coordination — and the review notes the chain one is logarithmic, so the eight-fold parameter drift is not an eight-fold feature drift | ``packages/assayer/docs/spec/reference-configuration.md`` |
| ``tab`` | ``tab:config:guidance`` | ``tbl:cfg:guide`` | the label guidance parameters | binds; shipped | ``packages/assayer/docs/spec/reference-configuration.md`` |
| ``tab`` | ``tab:config:monitoring`` | ``tbl:cfg:hm`` | the health monitoring parameters | binds; all rows are public, validated configuration and the live declaration projection covers them; feedback latency and attenuation consumption remain definition-STOPped | ``packages/assayer/docs/spec/reference-configuration.md`` |
| ``def`` | ``def:config:synchronisation-threshold`` | unminted | the synchronisation-error reporting threshold, $10^{-6} \cdot p$ | binds; configured coefficient and exact comparison shipped | ``packages/assayer/docs/spec/reference-configuration.md`` |
| ``def`` | ``def:config:concordance-minimum-labels`` | unminted | the per-entity concordance sample gate, five labels | binds; configured exact gate shipped | ``packages/assayer/docs/spec/reference-configuration.md`` |
| ``def`` | ``def:config:concordance-deficit-threshold`` | unminted | the per-entity concordance deficit threshold, 0.25 | binds; configured strict deficit shipped | ``packages/assayer/docs/spec/reference-configuration.md`` |
| ``def`` | ``def:config:strong-alarm-threshold`` | unminted | the alarm-outcome strong-alarm threshold, 3.0 | binds; configured inclusive boundary shipped | ``packages/assayer/docs/spec/reference-configuration.md`` |
| ``def`` | ``def:config:no-alarm-threshold`` | unminted | the alarm-outcome no-alarm threshold, 0.5 | binds; configured inclusive boundary shipped | ``packages/assayer/docs/spec/reference-configuration.md`` |
| ``def`` | ``def:config:alarm-outcome-noise-allowance`` | unminted | the alarm-outcome accumulator noise allowance, 0.02 | binds; configured subtraction shipped | ``packages/assayer/docs/spec/reference-configuration.md`` |
| ``def`` | ``def:config:feature-stability-threshold`` | unminted | the feature-stability cut, 0.001 | binds; configured independent threshold shipped | ``packages/assayer/docs/spec/reference-configuration.md`` |
| ``def`` | ``def:config:maturity-threshold`` | unminted | the per-cell cut for measurement-maturity coverage, 0.1 | binds; configured diagnostic shipped | ``packages/assayer/docs/spec/reference-configuration.md`` |
| ``def`` | ``def:config:resolution-adequacy-threshold`` | unminted | the resolution-utilisation evidence gate, twenty eligible labels per cell | binds; configured diagnostic shipped | ``packages/assayer/docs/spec/reference-configuration.md`` |
| ``def`` | ``def:config:feedback-latency-ewma-rate`` | unminted | the end-to-end feedback-latency EWMA rate, 0.99 | binds; field and validation shipped, consumer definition-STOPped | ``packages/assayer/docs/spec/reference-configuration.md`` |
| ``def`` | ``def:config:ledger-materiality-threshold`` | unminted | the immature-Ledger-cell materiality gate, one hundred eligible labels | binds; configured, and consumed by both the full-health count and the per-assessment immaturity flag | ``packages/assayer/docs/spec/reference-configuration.md`` |
| ``def`` | ``def:config:attenuation-materiality-floor`` | unminted | the per-assessment Ledger attenuation floor, 0.25 | binds; field, validation and consumer all shipped — the immaturity flag's rate arm compares against it | ``packages/assayer/docs/spec/reference-configuration.md`` |
| ``tab`` | ``tab:config:derivation`` | ``tbl:cfg:policy`` | the derivation function configuration | binds; **flagged whole** — the subject is the unbuilt Part IV surface | ``packages/assayer/docs/spec/reference-configuration.md`` |
| ``tab`` | ``tab:config:rendering`` | ``tbl:cfg:rend`` | the rendering configuration | binds; **NEW** — cited twice and never written; **resolves a dangling identity**; the type is real in the implementation | ``packages/assayer/docs/spec/reference-configuration.md`` |
| ``tab`` | ``tab:config:companion`` | ``tbl:cfg:comp`` | the Companion tracker configuration | binds; shipped, all rows | ``packages/assayer/docs/spec/reference-configuration.md`` |

Citation spine. Every table cites the environments whose parameters it tabulates — the direction that keeps a configuration reference a reference — and the general rule is that nothing cites back. The review found the rule stated here as though it had no exceptions and one exception stated two sentences later, so it is drawn properly now: the rendering table is cited, by the rendering contract and by the derivation table, which are the two sites that cite it today and find nothing, and the rendering appendix's bandwidths cite it as well. That is the whole of the back-citation, and it is confined to the one table this outline mints new; the fifteen tables carried over from the present chapter are cited by nothing, as a reference should be.

**Entry (Chapter 29, Output Structures)** · `entry:assayer:spec-outline-ch29`

The chapter head mints ``chap:spec:output-structures``, and the chapter mints **no environments at all**. This is a deliberate disposition, taken against the audit's sixth requirement (`obs:assayer:spec-restatement-chapters`) and the section register's sharpening of it. Under the calculus a restatement names nothing new — it refers — and three of the chapter's four sections are short prose restatements of structures defined in the derivation, Companion, assessment and monitoring chapters, already checked at their defining sites. They become citations, and the chapter becomes a generated register of them (`entry:assayer:tool-generated-registers`).

The fourth section is the one to treat carefully, and it is the reason the chapter survives as a head at all. Its health field table is the only place the health report is enumerated as a whole, and roughly half its rows name quantities that are absent or placeholder-backed. Transcribing it would carry those claims forward; discarding it would lose the only whole-report view. Generating it against the code is what stops it claiming fields that do not exist, and the outline records that as the requirement (`req:assayer:spec-outline-schedule`) rather than as an environment.

**Entry (Chapter 30, Guarantees and Known Limitations)** · `entry:assayer:spec-outline-ch30`

The chapter head mints ``chap:spec:guarantees-and-limitations``. The specification's two registers of record, and the outline's largest single block of environments, because both are already environments in table clothing and each row wants a head (`summ:assayer:spec-sections-c`).

Three rules govern this chapter. **Formal properties mint once**, and Chapter 30 is the home of most of them. Seven mint elsewhere, at the chapter that states the property as the whole content of an environment, and this chapter cites them: the feed-forward invariant at Chapter 1, presentation-freedom at Chapter 13, map covering and map contiguity at Chapter 9, and non-blocking, staleness and lifecycle publication at Chapter 22. Four of those seven keep area ``guarantee`` because the guarantee is what they are; three take their chapter's area because the environment there is the statement and the guarantee is its force. So the register is twenty-nine properties, twenty-two minted here and seven cited. **Each property carries its own disposition**, not the chapter's, and the disposition is stated in the row that mints it. Over the twenty-two minted here: nine are settleable by reading code, three by pointing at a test, six by neither — those carry the obliges force — and four are contradicted or partial against the shipped code, which is the finding, not a rounding error. The seven minted elsewhere carry their dispositions at their own rows, where six are shipped and one — presentation-freedom at Chapter 13 — is unbuilt; a reader wanting the whole picture reads twenty-two rows here and seven there. An earlier revision gave a single twelve-seven-six-four partition across all twenty-nine, which the review could not recompute from the rows and which undercounted the unbuilt properties by leaving presentation-freedom out of every bucket; the partition is drawn per minting site now so that it can be checked. **Limitations mint once here** and every chapter that discusses one cites it; the count is forty-two, not the forty the audit estimated.

| Kind | Head | Derived from | Scope | Flags | Document |
| --- | --- | --- | --- | --- | --- |
| ``inv`` | ``inv:guarantee:assess-only`` | ``inv:assess-only`` | assessment writes only the enumerated set | binds; shipped; code-verifiable | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``inv`` | ``inv:guarantee:derivation-purity`` | ``inv:deriv-pure`` | the derivation is a pure function of its inputs | binds; shipped on the legacy surface; code-verifiable | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``inv`` | ``inv:guarantee:evidence-authority`` | unminted | assessment moves observational geometry and no outcome-learned state, and discloses what it moved | binds; code-verifiable; the authority half shipped, the **disclosure half unbuilt** — the package publishes its cold coordinate change in one step and carries neither phase nor count | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``inv`` | ``inv:guarantee:companion-independence`` | ``inv:comp-indep`` | the Companion reads no Core state and the Core holds none of its | binds; shipped; code-verifiable | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``inv`` | ``inv:guarantee:blend-subspace`` | ``inv:blend-subspace`` | the blend operates within the anchor's subspace | binds; shipped; code-verifiable | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``inv`` | ``inv:guarantee:ledger-floor`` | ``inv:ledger-floor`` | Ledger influence is floored, never eliminated | binds; shipped; code-verifiable | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``inv`` | ``inv:guarantee:outcome-neutrality`` | ``inv:neutrality`` | axis predictions never enter the derivation | binds; shipped; code-verifiable | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``inv`` | ``inv:guarantee:drift-visibility`` | ``inv:drift-vis`` | model drift is detected and reported | binds; shipped; code-verifiable | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``inv`` | ``inv:guarantee:encoding-transparency`` | ``inv:enc-transparency`` | the encoding's consequences are visible to the host | binds; **unbuilt** — the declarations it rests on do not exist | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``inv`` | ``inv:guarantee:conjugacy`` | ``inv:conjugate`` | the Companion's update is conjugate | binds; shipped; code-verifiable | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``inv`` | ``inv:guarantee:replay`` | ``inv:replay`` | a recorded assessment replays to the same output | binds; shipped on the legacy surface; code-verifiable | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``inv`` | ``inv:guarantee:structural-exactness`` | ``inv:struct-exact`` | lifecycle operations are exact on the structure | binds; shipped; test-backed | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``inv`` | ``inv:guarantee:axis-lifecycle`` | ``inv:axis-lifecycle`` | axis registration and deregistration preserve the posterior | binds; shipped; test-backed | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``inv`` | ``inv:guarantee:precision`` | ``inv:precision`` | precision is maintained within its stated bounds: a spectral floor on the least eigenvalue, a coordinatewise floor that bounds no eigenvalue, and the monitor that catches the drift neither addresses | binds; shipped; test-backed | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``inv`` | ``inv:guarantee:per-observation-exactness`` | ``inv:per-obs`` | the per-observation update is exact under the fixed model | obliges; analytic | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``inv`` | ``inv:guarantee:blend-variance`` | ``inv:blend-var`` | the blend's variance is the stated three-term form | obliges; analytic | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``inv`` | ``inv:guarantee:rigid-translation`` | ``inv:rigid`` | the crossover set translates rigidly in the shared term | obliges; analytic; describes the unbuilt landscape | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``inv`` | ``inv:guarantee:two-source-uncertainty`` | ``inv:two-source`` | crossover uncertainty has exactly two sources | obliges; analytic; describes the unbuilt landscape | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``inv`` | ``inv:guarantee:dominance`` | ``inv:dominance`` | dominance is reported and never enforced | obliges; analytic; describes the unbuilt landscape | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``inv`` | ``inv:guarantee:honest-uncertainty`` | ``inv:honest-unc`` | reported uncertainty is never narrower than the evidence supports | obliges; analytic | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``inv`` | ``inv:guarantee:posture-independence`` | ``inv:posture-indep`` | the regime structure is independent of the posture read | binds; **contradicted** — stated unconditionally, and the shipped surface takes no posture | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``inv`` | ``inv:guarantee:discrimination-visibility`` | ``inv:disc-vis`` | empirical coverage is tracked and reported | binds; shipped — coverage is computed at each calibration refit and reported, with the inflation factor also carried on the compact health snapshot. At the review it was **contradicted** because nothing computed coverage | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``inv`` | ``inv:guarantee:exploration`` | ``inv:explore`` | the Core and the derivation together expose an exploration signal | binds; **contradicted** — only the Core half exists, and the derivation half is the retired quantity | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``fig`` | ``fig:architecture:information-flow`` | ``fig:arch:flow`` | every arrow unidirectional, the only cycle through the host | explains; derivative — its boundary inventory is the interface appendix | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``cav`` | ``cav:limitation:valence`` | ``lim:valence`` | outcomes are observed on one side of the action only | discloses | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``cav`` | ``cav:limitation:ledger-contamination`` | ``lim:ledger-contam`` | the Ledger absorbs the consequences of the actions it informs | discloses | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``cav`` | ``cav:limitation:axis-contamination`` | ``lim:axis-contam`` | the same loop reaches spatially-enabled axes | discloses | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``cav`` | ``cav:limitation:host-loop`` | ``lim:host-loop`` | the host's policy shapes the population the Core learns from | discloses | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``cav`` | ``cav:limitation:interaction-convergence`` | ``lim:interaction-conv`` | interaction features converge slowly and unevenly | discloses | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``cav`` | ``cav:limitation:challenge-convergence`` | ``lim:qc-conv`` | the challenge estimate converges only as fast as contributing labels arrive | discloses | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``cav`` | ``cav:limitation:challenge-thin`` | ``lim:qc-thin`` | contributing labels are thin in realistic deployments | discloses | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``cav`` | ``cav:limitation:challenge-policy-loop`` | ``lim:qc-policy-loop`` | the challenge population is chosen by the policy it informs | discloses | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``cav`` | ``cav:limitation:challenge-narrow`` | ``lim:challenge-narrow`` | narrowing the action space narrows the evidence | discloses | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``cav`` | ``cav:limitation:laundering`` | ``lim:laundering`` | symmetric decay is exploitable in the forgiving direction | discloses; the exposure is accepted deliberately | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``cav`` | ``cav:limitation:ledger-low-traffic`` | ``lim:ledger-lowtraffic`` | the Ledger is structurally uninformative for low-traffic cells | discloses | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``cav`` | ``cav:limitation:fresh-entry`` | ``lim:fresh-entry`` | a fresh entry carries no evidence and inherits none | discloses | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``cav`` | ``cav:limitation:anchor-floor`` | ``lim:anchor-floor`` | the anchor floor bounds how far the blend can be pulled | discloses | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``cav`` | ``cav:limitation:linear`` | ``lim:linear`` | the models are linear in the feature vector | discloses | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``cav`` | ``cav:limitation:interaction-type-three`` | ``lim:type3`` | the wildcard template's feature count grows quadratically | discloses | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``cav`` | ``cav:limitation:cross-sentinel-gap`` | ``lim:cross-gap`` | cross-Sentinel structure is learned only through interactions | discloses | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``cav`` | ``cav:limitation:conditioning`` | ``lim:conditioning`` | the posterior can become ill-conditioned | discloses | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``cav`` | ``cav:limitation:ceiling`` | ``lim:ceiling`` | the importance-weight ceiling binds at extreme class rates | discloses; binding frequency and gradient balance now ship for both model streams | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``cav`` | ``cav:limitation:kappa-nonstationary`` | ``lim:kappa-nonstat`` | the compression scale assumes a stationary target distribution | discloses | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``cav`` | ``cav:limitation:axis-confounding`` | ``lim:axis-confound`` | axis training inherits the eligibility confound | discloses | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``cav`` | ``cav:limitation:cross-axis`` | ``lim:cross-axis`` | cross-axis structure is emergent and unmodelled | discloses | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``cav`` | ``cav:limitation:axis-cost`` | ``lim:axis-cost`` | each axis costs features and labels | discloses | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``cav`` | ``cav:limitation:chain-depth`` | ``lim:chain-depth`` | chain depth is bounded and the bound is visible in the features | discloses | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``cav`` | ``cav:limitation:chain-maximum`` | ``lim:chain-max`` | the maximum view carries an order-statistics bias | discloses | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``cav`` | ``cav:limitation:inter-report`` | ``lim:inter-report`` | nothing between reports is observable | discloses | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``cav`` | ``cav:limitation:signal-cache`` | ``lim:sigcache`` | the signal cache evicts, and eviction loses evidence | discloses | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``cav`` | ``cav:limitation:competitive-churn`` | ``lim:comp-churn`` | competitive churn resets measurement state | discloses | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``cav`` | ``cav:limitation:silent-encoding`` | ``lim:silent-encoding`` | a bad encoding degrades everything and announces nothing | discloses; its early warning ships as the mean-slot-weight reading — (`entry:assayer:defer-sentinel-informativeness`) | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``cav`` | ``cav:limitation:abstraction`` | ``lim:abstraction`` | the Core sees effects and never Sentinel-internal operations | discloses | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``cav`` | ``cav:limitation:late-standardisation`` | ``lim:late-std`` | a Sentinel added late standardises against a moving target | discloses | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``cav`` | ``cav:limitation:pre-calibration`` | ``lim:precal`` | outputs before the first refit are uncalibrated | discloses | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``cav`` | ``cav:limitation:buffer-eviction`` | ``lim:buffer-evict`` | a label arriving after eviction is lost | discloses | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``cav`` | ``cav:limitation:auc-lag`` | ``lim:auc-lag`` | discrimination metrics lag the change they measure | discloses | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``cav`` | ``cav:limitation:mimic-regime`` | ``lim:mimic-regime`` | a regime change can mimic feature-stable drift | discloses | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``cav`` | ``cav:limitation:patient-corruption`` | ``lim:patient-corrupt`` | patient label corruption is not detectable by these means | discloses | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``cav`` | ``cav:limitation:investigation-pipeline`` | ``lim:invest-pipeline`` | investigation requests presuppose a pipeline the Core does not have | discloses | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``cav`` | ``cav:limitation:severity-cold`` | ``lim:severity-cold`` | severity discrimination is cold until severity labels arrive | discloses | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``cav`` | ``cav:limitation:severity-path`` | ``lim:severity-path`` | severity enters only through the target, not the features | discloses | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``cav`` | ``cav:limitation:reward-sensitivity`` | ``lim:reward-sens`` | the landscape is only as good as the declared costs | discloses | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``cav`` | ``cav:limitation:fragility-gaussian`` | ``lim:frag-gauss`` | fragility assumes a Gaussian crossover distribution | discloses; describes the unbuilt surface | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``cav`` | ``cav:limitation:divergence-attribution`` | ``lim:delta-attrib`` | the divergence attributes to no single cause | discloses | ``packages/assayer/docs/spec/reference-guarantees.md`` |
| ``cav`` | ``cav:limitation:hibernation`` | ``lim:hibernate`` | hibernation preserves self-structure only | discloses; the restored cross-feature blocks are zero and a focused test holds them there | ``packages/assayer/docs/spec/reference-guarantees.md`` |

Citation spine. The guarantees register is the most-cited block in the document and cites almost nothing: each property cites the environment whose content it guarantees, and every chapter that states one cites back. The limitation register is cited from the chapters that create each exposure — the Ledger's laundering discussion, the encoding fallback, the action-space caveat, the buffer capacity — and cites the honest-notes appendix nowhere, because that appendix is generated from these rows.

## Appendices and matter · `sec:assayer:spec-outline-appendices`

Nine appendices in the present document, of which three carry decomposed environments, two become generated registers, one carries two orientation environments over imported material, one describes an index it does not contain, and two are reference tables that want one environment each rather than decomposition. The appendix heads mint ``app:spec:*``.

Structure. The heads below carry the appendices themselves, and are claimed here because the outline tracks the specification's skeleton as well as its contents.

| Head | Document |
| --- | --- |
| ``app:spec:resonance-rendering`` | ``packages/assayer/docs/spec/appendix-rendering.md`` |
| ``app:spec:extraction-reference`` | ``packages/assayer/docs/spec/appendix-reference-tables.md`` |
| ``app:spec:dimensional-summary`` | ``packages/assayer/docs/spec/appendix-reference-tables.md`` |
| ``app:spec:glossary`` | ``packages/assayer/docs/spec/appendix-dispositions.md`` |
| ``app:spec:honest-notes`` | ``packages/assayer/docs/spec/appendix-dispositions.md`` |
| ``app:spec:interface-map`` | ``packages/assayer/docs/spec/appendix-interfaces.md`` |
| ``app:spec:terminology-mapping`` | ``packages/assayer/docs/spec/appendix-interfaces.md`` |
| ``app:spec:background`` | ``packages/assayer/docs/spec/appendix-background.md`` |

**Entry (Front matter)** · `entry:assayer:spec-outline-front`

The front matter carries three things and keeps one. Its statement of the two-level tag convention dies with the scheme (`dec:assayer:tags-superseded`). Its component overview is the architecture chapter's material and is not repeated. What survives is its fixing of the qualified form for upstream references — the eighty-six citations of other corpora that the migration is expressly not about (`tab:assayer:spec-reference-forms`) — which becomes a convention with a head, because the legacy lint has to be told which forms remain valid (`req:assayer:spec-legacy-lint-shapes`).

| Kind | Head | Derived from | Scope | Flags | Document |
| --- | --- | --- | --- | --- | --- |
| ``conv`` | ``conv:spec:upstream-references`` | NEW | how a document of another corpus is cited, and why those forms are not legacy | binds; the lint is parameterized by it | ``packages/assayer/docs/spec/foundations-purpose.md`` |

**Entry (Appendix A, Resonance Rendering)** · `entry:assayer:spec-outline-app-a`

The head mints ``app:spec:resonance-rendering``. The audit's central inversion: the rewrite demoted the rendering to an optional appendix and the implementation ships the appendix and not the object it renders (`entry:assayer:spec-sections-app-a`). Seven of nine sections are shared, several exactly, so the migration's work here is to *subordinate* an existing surface rather than build a missing one — which is why every row below is flagged shipped or shared-shape, and only the examples are unbuilt.

| Kind | Head | Derived from | Scope | Flags | Document |
| --- | --- | --- | --- | --- | --- |
| ``sig`` | ``sig:rendering:contract`` | ``rend:contract`` | rendering as a function over a landscape and a risk basis | binds within the layer; **shared-shape** — the shipped entry point is the derivation itself, and there is no landscape argument | ``packages/assayer/docs/spec/appendix-rendering.md`` |
| ``tab`` | ``tab:rendering:spectrum`` | ``rend:spectrum`` | the two tag families and their three parameters each | binds; shipped | ``packages/assayer/docs/spec/appendix-rendering.md`` |
| ``eq`` | ``eq:rendering:kernel`` | ``rend:kernel`` | the kernel and its normalisation | binds; shipped | ``packages/assayer/docs/spec/appendix-rendering.md`` |
| ``prop`` | ``prop:rendering:completeness`` | ``rend:complete`` | the rendered distribution is complete over the spectrum | binds; shipped | ``packages/assayer/docs/spec/appendix-rendering.md`` |
| ``prop`` | ``prop:rendering:properness`` | ``rend:proper`` | the rendering is proper in the stated sense | binds; shipped | ``packages/assayer/docs/spec/appendix-rendering.md`` |
| ``prop`` | ``prop:rendering:no-clamping`` | ``rend:noclamp`` | magnitudes are not clamped | binds; shipped | ``packages/assayer/docs/spec/appendix-rendering.md`` |
| ``alg`` | ``alg:rendering:placement`` | ``rend:placement`` | action tag placement by crossover matching, with its existence test | binds; shipped, with the specification's own symbol | ``packages/assayer/docs/spec/appendix-rendering.md`` |
| ``def`` | ``def:rendering:bandwidths`` | ``rend:qband`` | classification, action and identity bandwidths, and the display floor | binds; shipped; whether the floor's quantitative threshold and cold-start figure are restored is **open** — see (`reg:assayer:spec-outline-open`) | ``packages/assayer/docs/spec/appendix-rendering.md`` |
| ``def`` | ``def:rendering:magnitudes`` | ``rend:magnitude`` | magnitudes, relevance and attenuation | binds; shipped | ``packages/assayer/docs/spec/appendix-rendering.md`` |
| ``alg`` | ``alg:rendering:dominated-treatment`` | ``rend:dominated`` | the three triggers and three effects of dominated-tag display | binds; shipped | ``packages/assayer/docs/spec/appendix-rendering.md`` |
| ``ex`` | ``ex:rendering:examples`` | ``rend:examples`` | the four worked landscapes, rendered | explains; **unbuilt** — depends on landscapes that do not exist | ``packages/assayer/docs/spec/appendix-rendering.md`` |
| ``sig`` | ``sig:rendering:ambiguity-gauge`` | ``rend:iv`` | tag probabilities and the ambiguity gauge, with its disclaimer | binds; **shared-shape** — the shipped type is the old metrics record and the disclaimer is absent | ``packages/assayer/docs/spec/appendix-rendering.md`` |

Citation spine. The contract cites the landscape output and the risk basis; the placement algorithm cites the crossover derivation and the posture-sensitivity functions; the bandwidths cite the rendering configuration table minted new in Chapter 28; the ambiguity gauge is cited by the fragility interpretation, which is where the two surfaces are distinguished.

**Entry (Appendix B, Extraction Reference Table)** · `entry:assayer:spec-outline-app-b`

The head mints ``app:spec:extraction-reference``. One table, no subsections, and a clean verification: the index groups partition the range without gap or overlap and the width formula holds (`entry:assayer:spec-sections-app-b`). A reference table in the strict sense, wanting one environment rather than decomposition — and a prime candidate for the mechanical check the configuration chapter argues for, since every row mirrors a constant.

| Kind | Head | Derived from | Scope | Flags | Document |
| --- | --- | --- | --- | --- | --- |
| ``tab`` | ``tab:extraction:reference-index`` | ``tbl:ext:layout`` | the complete extraction index, position by position, with the slot conversion | binds; shipped | ``packages/assayer/docs/spec/appendix-reference-tables.md`` |

**Entry (Appendix C, Dimensional Summary)** · `entry:assayer:spec-outline-app-c`

The head mints ``app:spec:dimensional-summary``. The specification's central accounting identity, which verifies at the reference configuration (`entry:assayer:spec-sections-app-c`). One table and one displayed equation.

| Kind | Head | Derived from | Scope | Flags | Document |
| --- | --- | --- | --- | --- | --- |
| ``eq`` | ``eq:dimension:total`` | unminted | the dimension formula, displayed | binds; shipped | ``packages/assayer/docs/spec/appendix-reference-tables.md`` |
| ``tab`` | ``tab:dimension:summary`` | unminted | the eight blocks at reference, and the scaling table | binds; shipped | ``packages/assayer/docs/spec/appendix-reference-tables.md`` |

**Entry (Appendix D, Glossary)** · `entry:assayer:spec-outline-app-d`

The head mints ``app:spec:glossary``, and the appendix mints **no environments**. A glossary defines nothing that is not defined elsewhere: every row is term, gloss and pointer, which is exactly what a citation index carries, and the register calls it the clearest generation candidate in the corpus (`entry:assayer:spec-sections-app-d`). It becomes a generated register (`entry:assayer:tool-generated-registers`), which also disposes of its seven rows naming Part IV types that do not exist: a generated glossary names what the document actually mints.

**Entry (Appendix E, Honest Notes Register)** · `entry:assayer:spec-outline-app-e`

The head mints ``app:spec:honest-notes``, and the appendix mints **no environments**. It is restatement (`obs:assayer:spec-restatement-chapters`) and it is exact restatement — every one of the forty-two limitations appears in exactly one grade list and the four tallies sum to forty-two (`entry:assayer:spec-sections-app-e`), which is the register's proof that generation would work here. It becomes a generated register over the limitation labels of Chapter 30 and their grades. Its closing emphasis on three items is authored prose and survives as a remark inside the limitation register's own preamble, not as an appendix environment.

**Entry (Appendix F, Interface Map)** · `entry:assayer:spec-outline-app-f`

The head mints ``app:spec:interface-map``. The boundary verification, and normative on that account (`entry:assayer:spec-sections-app-f`). One new environment restores the deleted graph-to-Sentinel section: the appendix's stated job is a complete interface map, one of its three interfaces is missing, and the loss reads accidental rather than edited (`reg:assayer:spec-divergence-losses`). The symbol cross-reference stays a translation aid and does not gain force by getting a head.

| Kind | Head | Derived from | Scope | Flags | Document |
| --- | --- | --- | --- | --- | --- |
| ``tab`` | ``tab:boundary:consumed`` | ``fmap:consumed`` | the record types the Core consumes from a batch report | binds; shipped | ``packages/assayer/docs/spec/appendix-interfaces.md`` |
| ``tab`` | ``tab:boundary:not-consumed`` | ``fmap:not-consumed`` | the record types it does not consume, and why | binds; shipped | ``packages/assayer/docs/spec/appendix-interfaces.md`` |
| ``tab`` | ``tab:boundary:graph-interface`` | NEW | the graph-to-Sentinel interface, with its capability and property pointers | binds; **restores a deleted section** — see (`reg:assayer:spec-outline-open`) | ``packages/assayer/docs/spec/appendix-interfaces.md`` |
| ``tab`` | ``tab:boundary:symbols`` | ``fmap:symbols`` | the Sentinel symbol cross-reference | explains | ``packages/assayer/docs/spec/appendix-interfaces.md`` |
| ``tab`` | ``tab:boundary:verification`` | ``fmap:verify`` | the boundaries, and the four that are never crossed | binds; shipped | ``packages/assayer/docs/spec/appendix-interfaces.md`` |

Citation spine. The verification table cites the feed-forward guarantee, the Companion boundary and the derivation purity, and is cited by the information- flow figure. The consumed table cites the report-reception environment; the new graph interface cites the upstream documents through the qualified form the front matter fixes.

**Entry (Appendix G, Cross-Layer Terminology Mapping)** · `entry:assayer:spec-outline-app-g`

The head mints ``app:spec:terminology-mapping``. Expository throughout, and expository material that prevents a specific error the Core's design depends on not making (`entry:assayer:spec-sections-app-g`). The abstraction boundary takes a ``conv``, because it is a rule for reading rather than a discussion.

| Kind | Head | Derived from | Scope | Flags | Document |
| --- | --- | --- | --- | --- | --- |
| ``tab`` | ``tab:terminology:structural`` | ``xterm:structural`` | structural operations across the three vocabularies | explains | ``packages/assayer/docs/spec/appendix-interfaces.md`` |
| ``tab`` | ``tab:terminology:importance`` | ``xterm:importance`` | what "importance" means in each layer | explains | ``packages/assayer/docs/spec/appendix-interfaces.md`` |
| ``conv`` | ``conv:terminology:abstraction-boundary`` | ``xterm:boundary`` | causation is traced; operations are not corresponded | binds as a rule for reading; shipped | ``packages/assayer/docs/spec/appendix-interfaces.md`` |

**Entry (Appendix H, Background)** · `entry:assayer:spec-outline-app-h`

The head mints ``app:spec:background``. The appendix carries two concise overviews, one for each upstream component, because the specification must state enough about both systems to be independently shareable. Their scope stops at the Core's boundary; the forty detailed subsections of the superseded appendix remain imported material rather than becoming forty Assayer-owned heads.

| Kind | Head | Derived from | Scope | Flags | Document |
| --- | --- | --- | --- | --- | --- |
| ``preview`` | ``preview:architecture:mudlark`` | NEW | what Mudlark's G-V Graph is, what the Core supplies to and consumes from its own instances, and why that is an ownership boundary | explains; restores the lost upstream orientation under the independent-shareability ruling | ``packages/assayer/docs/spec/appendix-background.md`` |
| ``preview`` | ``preview:architecture:spectral-sentinel`` | NEW | what a Spectral Sentinel measures, what the Core consumes from and provides to it, and why the report boundary is one-way | explains; restores the lost upstream orientation under the independent-shareability ruling | ``packages/assayer/docs/spec/appendix-background.md`` |

The overreach caveat (`cav:assayer:spec-implicit-overreach`) still governs the cut. The overviews cite current owner heads and state no upstream theorem, algorithm or configuration as Assayer material. In particular, the index corpus's Kraft-style equality stays imported. The thirty minted-but-never-cited identities from the superseded appendix still earn no Assayer heads; only the two interface explanations the present specification needs are restored.

**Entry (Appendix I, Tag Index)** · `entry:assayer:spec-outline-app-i`

The appendix does not survive, and mints nothing. It declares itself the binding surface for citation, specifies a row format, enumerates five build checks, and contains no rows; the build step it names does not exist and the one checker that does excludes the document by name (`obs:assayer:spec-tag-index-empty`). It is also wrong about the size of the register it indexes, by roughly a hundred identities. What it describes is the citation index the calculus produces by tooling, and the outline carries it forward as a requirement rather than as prose (`req:assayer:spec-outline-schedule`). Its one substantive content — the reserved-slug example — dies with the scheme that needed reserved slugs.

## Dispositions · `sec:assayer:spec-outline-dispositions`

**Register (New mints)** · `reg:assayer:spec-outline-new`

Thirteen environments the rewritten specification carries that its source edition did not — ten as first drawn, the computation order the review added, and the two background overviews restored later. Each is minted deliberately, and each has a reason a reader can check.

- ``conv:spec:upstream-references`` — the qualified form for citing other corpora, which the front matter fixes informally and the legacy lint must be parameterized by.
- ``pre:architecture:report-validation`` — the two Sentinel properties the Core enforces at ingestion with named errors, split out of the assumed-properties table because their force is different.
- ``rem:architecture:inherited-guarantees`` — the Sentinel-internal guarantees the Core inherits through the report, restoring a paragraph the rewrite deleted without replacement.
- ``rem:encoding:unchecked-width`` — the declared width is an unchecked assertion, restoring a deleted sentence that is now the more necessary because the field itself turns out not to exist.
- ``req:gaussian:positive-definiteness`` — the positive-definiteness check stated as its own requirement, against a minimum eigenvalue, because the code tests the diagonal and cites the section as though the two agreed.
- ``rem:extraction:single-precision`` — the extraction is single precision and every model consuming it is double, a split the specification never states.
- ``rem:signal:shape-widths`` — the signal shapes occupy different widths, which a reader cannot derive from the chapter and the dimension map depends on.
- ``req:standardisation:class-assignment`` — which feature belongs to which class, restoring the authority the rewrite deleted and left the priors table floating without.
- ``tab:config:rendering`` — the rendering configuration table, cited twice by the present document and contained nowhere in it.
- ``tab:boundary:graph-interface`` — the graph-to-Sentinel interface, restoring the deleted appendix section that leaves the interface map incomplete.
- ``alg:landscape:computation-order`` — the six-step computation order and its acyclicity, restoring the one piece of the old derivation chapter that was implementable guidance. Added at the review, which accepted the second open question.
- ``preview:architecture:mudlark`` — enough of Mudlark's purpose and direct Core ownership boundary to make the specification legible without its corpus.
- ``preview:architecture:spectral-sentinel`` — enough of the Sentinel's measurement and detached-report boundary to make the specification legible without its corpus. The count of new mints is therefore thirteen, and of environments 478.

Six of the thirteen restore deleted material rather than inventing it, and four of those are loss candidates the audit or a section register flagged as reading accidental. The review tested them, which is what they were minted provisionally for. Three stand on the audit's own reading — the interface map, the Sentinel-guarantees paragraph and the computation order are each recorded there as reading accidental rather than edited — and the unchecked-width sentence stands on the stronger ground that the field it describes turns out not to exist. None was found to be a deliberate deletion wearing the look of an accident, which was the failure mode the provisional minting was guarding against. The two background overviews stand on the later shareability ruling and restore the orientation the historical appendix carried.

**Register (The seven dangling identities, disposed)** · `reg:assayer:spec-outline-dangling`

The audit's third requirement is that the seven identities cited with no mint site (`reg:assayer:spec-dangling-tags`) be resolved deliberately. All seven are disposed here and none is left implicit.

- ``mot:division``, ``mot:consulted``, ``mot:bonus`` — the three mottos of the old scheme's own section. They are cited in running prose and were never minted anywhere, and what they name is the tag scheme's rhetoric rather than the specification's material. **Retired with the scheme**; the citing prose is rewritten without them. Three identities retired.
- ``led:materiality`` — the dual-condition materiality result, cited and stated without a mint. **Minted** as ``thm:ledger:materiality``, which is what a stated result wants, and cited by the alarm summary and the immature-cell metric — the pair the section register found would be satisfied by different work.
- ``model:blend-mech`` — the blend's two persistence mechanisms, cited and then stated unminted in the following paragraph. **Minted** as ``rem:risk:blend-mechanisms``.
- ``tbl:cfg:rend`` — the rendering configuration table, cited twice and never written. **Minted new** as ``tab:config:rendering``, which the implementation supports: the type is real and exported (`obs:assayer:drift-rendering-config`).
- ``land:iv`` — the scheme's own worked example of a deliberately reserved, never-minted slug. Correct by construction under the old scheme and meaningless under the calculus, which has no reserved-slug device. **Retired** with the appendix that needed it.

Three want mint sites and get them; four identities in two families retire. The earlier revision of this register said five and got them, which the review found wrong on both halves of the sentence, and the correction matters because the arithmetic was the only evidence offered that the audit's third finding was discharged.

It is discharged, but not by agreement. The audit read the seven as five wanting mint sites, one wanting a configuration table, and one correct as it stands (`req:assayer:spec-outline-inputs`). This outline departs twice, and the review lets both departures stand while recording them, since neither was recorded before. The three mottos the audit counted among the five wanting mint sites are retired instead: what they name is the tag scheme's rhetoric, not the specification's material, and minting rhetoric to satisfy a count is the overreach (`cav:assayer:spec-implicit-overreach`) forbids. And ``land:iv``, which the audit called correct as it stands, is retired rather than left standing: it was correct only as the old scheme's worked example of a reserved slug, and the calculus has no reserved-slug device for it to be an example of. Both departures shrink the register rather than growing it, which is the direction the overreach caveat asks for; a departure that grew it would want a ruling rather than a review.

**Register (The deferral register, checked against these rows)** · `reg:assayer:spec-outline-deferrals`

Added at the review, which found the check nowhere recorded. The rule this outline follows in Part VII — where an environment is deferred it cites the deferral, and where it is absent it says so and names the deferral it is *not* — was applied inside the monitoring chapter and nowhere else, so nothing said what had become of the rest of the register.

The result is clean on the question that matters. **No row of this outline closes a registered deferral, contradicts one, or re-describes one in its own words.** Every deferral this outline cites is live in the register at the revision reviewed; there are nineteen of them, and each is cited rather than restated, which is the discipline the deferral register asks for.

At the review cut, nine of the twenty-eight registered deferrals were cited by no row here, a fact about coverage rather than a defect. Five named health surfaces that no environment of this outline mints — the Ledger depth-tier distribution, Ledger cell-set deltas, the full Sentinel structural relay, Sentinel coverage metric and zero-Sentinel counter. The other four sat beside rows that described the same surface from the specification's side: the drift-reset counter beside ``tab:monitoring:drift-resets``, signal-cache metrics beside ``tab:keyspace:signal-cache``, positive-class prior metrics beside ``tab:weighting:trackers``, and the Schur correction skip counter beside ``alg:gaussian:regularised-schur``. The rows said what each quantity *is* while the deferrals said whether it was exported, so neither closed the other.

All nine have since completed and their register entries are retired. The historical distinction remains useful, but the rewrite no longer needs open-gap citations for those surfaces: a reader now reaches implementation evidence from each owning record's completed entry.

**Register (What earns no environment)** · `reg:assayer:spec-outline-noenv`

Five dispositions, each deliberate rather than an omission, and each the audit's sixth requirement or the overreach caveat applied.

- **Chapter 29, Output Structures.** Pure restatement of structures defined in four other chapters. Three sections become citations; the fourth — the only whole-report enumeration of the health surface, roughly half of whose rows name absent or placeholder-backed quantities — becomes a generated register checked against the code.
- **Appendix D, Glossary.** Term, gloss and pointer per row: a citation index by another name, generated.
- **Appendix E, Honest Notes.** Grade tallies over the limitation register, exact today and cheaper to generate than to maintain.
- **Appendix H, detailed background.** Two overview environments orient the reader; the forty detailed subsections remain imported citations into their owners, and thirty never-cited identities do not become thirty Assayer environments.
- **Appendix I, Tag Index.** A description of an index, containing no index. Its function is the calculus's citation index, produced by tooling.

Beside these, the bold-lead paragraphs that carry no identity, the display-math blocks that are steps inside derivations, and the configuration parameters that are rows of tables all fold rather than promote, per (`conv:assayer:spec-outline-derivation`).

Three restatements mint anyway, and the review records the exception rather than leaving the rule looking absolute. ``summ:landscape:design-summary`` restates eleven design aspects and is marked a generation target in its own row; ``tab:warmup:milestones`` restates thresholds fixed at three other chapters; and ``tab:monitoring:observability-summary`` restates the scope and threshold of every observability metric. The rule above disposes of restatement that constitutes a *whole chapter or appendix*, where the unit of generation is the document division and nothing is left over. These three are restatements sitting inside chapters that carry authored material around them, so the division cannot be generated as a unit and the restatement needs a head to be generated *into*. The distinction is worth stating because it is the one a rewrite would get wrong: a chapter of pure restatement becomes a citation index, and a table of restatement inside an authored chapter becomes a generated environment with a label. Each of the three is explains-force and cited by nothing, which is what a restatement should be.

**Requirement (What the outline schedules for other entries)** · `req:assayer:spec-outline-schedule`

Six obligations this outline records and does not discharge, each belonging to a named entry. The first four were scheduled when the outline was drawn; the last two arrived at the review, which settled two open questions into obligations rather than leaving them as questions.

1. **Remove the specification from the pre-calculus list in the commit that migrates it.** The linter carries `docs/spec.md` as a pre-calculus document, so it forms no minting or resolution judgment today (`obs:assayer:spec-nonparticipation`); a rewritten document still listed there would lint vacuously, and every label planned above would be inert. The removal belongs to (`spec:spec:measurement-judgement-interpreter`), in the same commit as the assembly.
2. **Configure the legacy-reference lint to the census's shapes.** The forms it must find over migrated Assayer prose, and the upstream forms it must not, are fixed by (`req:assayer:spec-legacy-lint-shapes`); the per-document configuration matters, since the pre-rewrite specification and the tag document are saturated with the forms it hunts and must stay unconfigured until they leave the tree. The work is governed by (`dec:assayer:burn-lists`).
3. **Generate the four registers rather than authoring them.** The health field enumeration, the glossary, the grade tallies and the citation index are all generation targets under (`entry:assayer:tool-generated-registers`), and the health enumeration is the one that must be generated against the code, which is what stops it claiming fields that do not exist.
4. **Convert these tables to tracking tables at concatenation.** The header above deliberately fails to declare tracking (`conv:assayer:spec-outline-columns`); the conversion and the both-ways check are carried by (`plan:assayer:specification-rewrite-contract`) and land with the assembly.
5. **Raise a backlog entry for the label-time reconstruction gap.** The review settled the eleventh open question in the affirmative and the obligation lands here rather than staying a question. Reconstruction leaves the identity, axis, aggregate and interaction blocks at zero, so every model trains on a vector that differs systematically from the one its prediction was computed from; the pending entry lacks the four components the missing steps would read, so the two fixes are ordered and the entry must say which lands first. This is a live correctness question in a shipped learning path and it belongs in the backlog, not in a flag on a row of an outline.
6. **Confirm the fifteen remaining upstream reference targets.** The sixth open question stays open on its residue: one dropped pointer was identified and is restored, and the other fifteen were not individually opened. Confirming that none was load-bearing means reading fifteen sections of two other corpora against the sites that used to cite them. The work belongs with the front matter's upstream-reference convention, whose whole subject is which qualified forms remain valid, and it should be done before that convention is written rather than after.

## Open questions · `sec:assayer:spec-outline-open`

**Register (The fourteen questions, adjudicated; three still open)** · `reg:assayer:spec-outline-open`

Every item below was stated with this outline's recommended disposition and carried to (`plan:assayer:specification-rewrite-contract`). The review has now adjudicated all fourteen: eleven are settled on the evidence and are recorded here with the reasoning that settled them, and three remain open because they are substantive content decisions the evidence cannot make. The three still open are items 3, 6 and 8, and each says so in its own first line so that nobody has to read the list to find them. A settled item is no longer awaiting anything; where the settlement changed the tables above, the change is already made and the budget recomputed.

The loss candidates of (`reg:assayer:spec-divergence-losses`) come first, followed by the divergence notes the section registers added, followed by the questions this outline's own choices raise.

1. **The deleted interface-map section.** SETTLED — restore, as recommended. The graph-to-Sentinel interface was deleted outright, taking the only pointers to the upstream capability list and to the graph's assumed properties, and the surviving section kept its number so nothing marks the hole. *The review accepts. The audit reads the deletion as accidental, the appendix's stated job is a complete interface map, and one of its three interfaces is missing; the content was two sentences of pointer, so the restoration is cheap and the omission is not.* The row ``tab:boundary:graph-interface`` stands as drawn.
2. **The missing computation order.** SETTLED — restore, as recommended, and the row is added. Six steps fixing the order in which crossovers, bandwidths, magnitudes, interior locations, boundary locations and classification tags are computed, with the note that the chain is acyclic, vanished with the old derivation chapter, and nothing in the crossover chapter or the rendering appendix states an order. *The review accepts on the audit's own reading: this is the one piece of the old chapter that was implementable guidance rather than derivation, and it is the piece that vanished. An order that the code performs and the document does not state is precisely the drift this migration exists to end.* Added as ``alg:landscape:computation-order`` in Chapter 15; the environment count moves to 476 and the budget is recomputed.
3. **The quantitative bandwidth-floor threshold.** SETTLED — restore the numbers, by the review's own cheap settler. The shipped derivation configuration carries the floor as a real constant — `q_floor` defaulting to `0.3` in `src/resonance/derivation.rs`, applied as a maximum against the raw value, with a code comment citing the very section of the old text whose figures were dropped. The number to restore is therefore already chosen by the implementation, and the question collapses to a transcription: the rendering appendix's floor environment states the binding condition against the shipped constant, and the rendering configuration table (``tab:config:rendering``) carries it. The falsifiability the old text had is recovered without deciding anything the code has not already decided.
4. **The three-mechanism domination table.** SETTLED — accept the loss, as recommended. Crossover inversion, existence failure and catching forfeiture as three named conditions in one table; all three survive as prose clauses in the rendering appendix. *The review accepts. The clauses carry the content, the table carried only its arrangement, and ``alg:rendering:dominated-treatment`` already mints the three triggers and three effects as one environment, so nothing is lost that a citation cannot reach.*
5. **The per-crossover sensitivity table.** SETTLED — accept the loss, as recommended. The derivative of each crossover against the challenge estimate, per transition, with numeric contributions at defaults; replaced by the rank-2 covariance structure, which is stronger. *The review accepts, and accepts the second half too: the worked per-transition numbers move into ``ex:landscape:three-action``, where they are an acceptance oracle rather than a derivation. A number that can be checked against a running system is worth more than the same number inside a table that derives it.*
6. **Sixteen dropped upstream reference targets.** **STILL OPEN, in part.** Ten of one upstream document and six of another are cited by the old text and appear nowhere in the new one. *The review settles the one the outline had already identified and cannot settle the other fifteen. The identity chapter's host duties sourced its budget-parameter requirement to an upstream section and that pointer survives nowhere, so a host is told to configure parameters and not told where they are defined; that pointer is restored inside ``req:keyspace:host-duties`` regardless of the rest. The remaining fifteen were not individually opened at this review — confirming that none was load-bearing means reading fifteen sections of two other corpora, which is a bounded task but not this one.* It is recorded as an obligation rather than left as a question: the check belongs with the front matter's upstream-reference convention, where the qualified form is fixed.
7. **The deleted Sentinel-guarantees paragraph.** SETTLED — restore, as recommended. The old assumed-properties section closed with a statement that the Sentinel's internal guarantees are inherited through the report without being referenced directly; the rewrite deletes it with no replacement. *The review accepts. It reads accidental, and it is the sentence that explains why the assumed-properties table is short — a table whose shortness is unexplained invites a later editor to lengthen it.* The row ``rem:architecture:inherited-guarantees`` stands as drawn.
8. **Two numeric losses in the valence chapter.** **STILL OPEN.** A directional-coverage figure was changed with no stated basis in either edition and no supporting computation anywhere, and a stated half-life became an unfalsifiable statement deferring to a later analysis. *The review confirms the finding and cannot discharge it. It verified that no supporting computation exists anywhere in the corpus for the changed figure, which establishes the problem and supplies none of the answer: a strengthened empirical claim that arrived without evidence wants either the evidence or the older claim back, and only whoever changed it knows which. The half-life is the same shape — the number restored, or an explicit cross-reference to the analysis it defers to, and the review has no basis to choose.* Both rows carry the flag; neither should be written as prose until this is answered, because both readings produce a confident sentence and only one of them is true.
9. **The lost cost breakdown.** SETTLED — restore, as recommended. The Gaussian chapter's worked cost example was restated to the current reference dimension and lost the per-matrix breakdown that let a reader check the total. *The review accepts. The breakdown is arithmetic already implied by the operation costs above it, so restoring it invents nothing and costs a table row; the total is right and unverifiable as it stands, and unverifiable is the condition this campaign treats as a defect.* Restored inside ``tab:gaussian:operation-costs``.
10. **The Kraft equality.** SETTLED — leave it imported, as recommended. Appendix H is imported material by ruling, and this one subsection is a named mathematical result the audit lists among the theorem candidates. *The review accepts, and notes that the two standing rulings point the same way: the upstream scope ruling makes the appendix imported material, and the overreach caveat names Appendix H by name as the thing not to promote. Minting one theorem out of forty subsections because it is the most theorem-shaped is admission by inclusion under another description, which is the failure the disposition exists to prevent. If the Kraft equality is wanted as an Assayer result, it should be stated where it is used and cited from there, not lifted out of a summary of somebody else's work.*
11. **The reconstruction gap wants its own entry.** SETTLED — it does, and the review raises it. Label-time reconstruction leaves the identity, axis, aggregate and interaction blocks at zero, so every update is applied to a vector differing systematically from the one its prediction came from, and the pending entry lacks the four components the missing steps would read. It sits inside a chapter the code otherwise implements step for step. *The review accepts and sharpens: reading the reconstruction directly confirmed the four zeroed blocks and found the populated share smaller than the register reported, since the first numbered source's identity base features are not written either. It is a live correctness question in a shipped learning path and the outline is the wrong instrument for it.* It moves out of this register and into (`req:assayer:spec-outline-schedule`) as a fifth scheduled obligation.
12. **Area collisions with the projected record set.** SETTLED — the outline's areas stand and the record set renames. Areas are unique across the owner under the ratified naming schema, and this outline claims forty-one of them, not the forty-two an earlier revision counted. *The review made the check the question asks for, against the thirteen clusters proposed at (`proposal:assayer:adr-projected-set`), and the result is worse than the question anticipated: the four defensive renamings missed. The clusters most likely to want an area word are the runtime paths, the host contract, features and standardisation, the feed-forward boundary, concurrency and publication, observability, and the Companion — which collide with* ``runtime``, ``host``, ``feature``, ``standardisation``, ``boundary``, ``publication``, ``monitoring`` *and* ``companion`` *respectively. Two of those, ``publication`` and ``monitoring``, are areas this outline chose specifically to dodge a record collision, and they collide anyway. Renaming defensively does not work when the thing being dodged has not been named yet.*

    The policy, applying the principle the question itself proposes — the concept goes to whichever document states it, not whichever claims first. Every one of those eight is stated by the specification and merely decided about by a record: the specification says what the runtime paths are and the record fixes their lock ordering; the specification defines the host contract and the record records what was chosen about it. So **the outline's areas stand in every collision above, and the record set takes different words.** Three further facts make this cheap rather than contentious. The proposal binds nothing and says so (`cav:assayer:adr-projection-nonbinding`); its names are declared descriptive and explicitly not anticipating the naming schema; and the record set does not exist yet, so renaming costs a choice where renaming an outline area would cost 476 rows. The corollary is that the four defensive renamings this outline already made are unnecessary under the policy. They are kept — ``platt``, ``publication``, ``monitoring`` and ``keyspace`` are all defensible words on their own merits, and rewriting them now would churn the contract to no end — but they should not be read as precedent, and no future area should be chosen defensively.
13. **Whether the guarantee register should mint at all.** SETTLED — keep the split as drawn. This outline mints the formal properties in one place and excepts the seven stated as a chapter's whole content; the alternative is minting every property at its chapter and generating the register entirely. *The review accepts, and the recommendation's own worry does not survive inspection. It warns that a rule with seven exceptions is close to the rule it is not — but four of the seven keep area* ``guarantee``*, so they are the rule wearing a different location, and only three genuinely take their chapter's area. A rule with three exceptions out of twenty-nine is a rule.* The test the recommendation proposes is kept: if the rewrite finds itself adding a fifth or sixth genuine exception, the split is wrong and the whole register should be generated.
14. **Whether ``sec`` division heads should exist at all.** SETTLED — keep them, and the rule that admits them is restated, because as written it did not describe what this outline does. *The review accepts the recommendation and rejects its stated ground. The outline says it keeps division heads only where a section register found a division head carrying no material; it keeps sixteen, and the registers between them mark fifteen. The sets are not nested. The registers' division at Chapter 14 gets no head here, and two heads —* ``sec:dimension:map`` *and* ``sec:config:core`` *— stand over material the registers tabulate differently, the first over a section that carries three invariants this outline promotes out of it. Both are good heads; the rule was simply not the reason for them.* The rule as it should read: a ``sec`` head exists where a chapter's environments group into subjects a reader would otherwise have to hold in mind unaided, and the grouping is recorded here rather than derived by the rewrite. The recommendation's real safeguard is unchanged and is the part that matters — the rewrite adds no division head this outline does not list, because an unlisted division head is exactly the numbering the migration is abolishing, wearing a label.

## Verdict · `sec:assayer:spec-outline-verdict`

**Summary (What this outline plans, and what it leaves open)** · `summ:assayer:spec-outline-verdict`

The rewritten specification carries the Part and chapter skeleton it has today and none of its numbering. Under it sit the environments tabulated above, minted from the identity register the old scheme accumulated, given kinds from the adopted registry, and flagged for binding force and for distance from the shipped code. Four hundred and seventy-eight environments, counted and distributed in (`data:assayer:spec-outline-budget`). That count was made by hand, which is a claim nothing checks, so the review re-derived every figure in the budget mechanically. The distribution by Part, by chapter, by kind, by force and by provenance came back exact, and so did the label set: 478 labels, every one distinct, every one conformant, every kind drawn from the adopted registry and every area from the declared mapping. Four counts did not survive — the number of distinct kinds, the areas claimed at the twelfth open question, the arithmetic of the dangling-identity register, and the binding-row count in the first chapter's entry — and each is corrected in place. The total moved by one, for a restoration the review accepted and not for a miscount.

Four results are worth stating in prose. First, the total exceeds the audit's estimate of roughly two hundred environments, and the excess is not overreach: it is three registers the estimate counted once each — twenty-nine formal properties, forty-two limitations, and the configuration tables — plus the document skeleton, which the estimate did not count at all. The restraint the overreach caveat asks for was applied where it bites, and the folding it produced is now totalled as well as recorded: the rows consume 400 of the census's 456 minted identities, and the 56 that earn no head are accounted for rather than lost.

Second, the Part IV decision the audit demanded is discharged per environment rather than per Part, and the picture is sharper than the chapter-level finding. The genuinely unbuilt surface is narrower than "Part IV is unimplemented" suggests: the risk basis is shared field for field, the purity guarantee holds and is tested, the crossover arithmetic and the two-source variance are already computed and merely unexposed, the rendering appendix ships almost entire, and the Core label guidance is shared down to the field names. What is unbuilt is the object — the landscape itself, its utilities, the fragility half, and the two consumers blocked by a flattened posterior at the boundary.

Third, three environments outside Part IV remain unbuilt: the encoding contract's registration surface and the two unregistered monitoring metrics; label-time reconstruction now ships. The empirical coverage diagnostic carried the same unbuilt status at the review and now ships, as its Chapter 27 row records. The three unbuilt environments are flagged in their rows, and the Chapter 21 reconstruction row records its shipped state.

Fourth, and added at the review: the fourteen questions this outline carried are adjudicated. Eleven were settled on the evidence at the review; the bandwidth-floor numbers settled after it by the review's own cheap settler (the shipped configuration carries the floor constant, so restoration is transcription); the fifteen unopened upstream reference targets travel as a scheduled obligation. One question remains genuinely the owner's: the two numeric losses in the valence chapter, where the evidence establishes that something was lost and cannot establish whether losing it was meant — only whoever changed the figures can supply the basis. The rewrite can proceed around it: it touches a single row.

**Caveat (What this outline did not do)** · `cav:assayer:spec-outline-limits`

Four limits, inherited and new, restated at the review because one of them has since been partly discharged. This outline is built from the chapter audit and the three section registers and did not re-read the specification section by section; where a register's kind call or drift verdict is wrong, the row above is wrong with it, and the registers' own limits — chapter-granularity divergence in one, spot-checked drift in another, no code check of the imported appendix in the third — are inherited rather than discharged.

The second limit was that the outline performed no code check of its own: every shipped, unbuilt and shared-shape flag was a register's reading carried forward with its citation. The review's adversarial pass has now opened thirty-nine of them against the code, spanning all three statuses and drawn across Parts I through VIII — roughly one status assignment in eight. It found thirty-seven sound, one wrong, and one imprecise enough to mislead. The wrong one is the coordinate lift, which the registers read as shipped and which no routine in the crate performs, the specification itself having assigned it to the integration layer; that row is corrected above. The imprecise one is the publication interval, where a field of the same name exists on an unrelated configuration. Three further flags were sharpened rather than overturned: the feed-forward guarantee is held by construction and a build-time lint rather than by a check at reception, the reconstruction gap is wider than the register reported, and the extraction chain-length drift moves two features rather than one. **The remaining seven in eight status assignments are still a register's reading and have not been opened**, and the sampling was deliberately weighted toward the flags a wrong call would cost most — the unbuilt ones and the shared-shape ones, which decide what the migration builds. A flag outside the sample is as good as the register that made it and no better.

The outline fixes scope in one line per environment and no more, which is enough to hold the rewrite to a contract and not enough to write from without reading the source section. And it decides nothing about ordering within a chapter beyond the order of the rows: the rewrite may group and sequence its prose freely, so long as the environments are the ones listed, in the order listed, under the labels given.
