# The Dropped Upstream References · `rep:assayer:lost-upstream-authorities`

This report is evidence for a ruling, and it settles nothing on its own. The outline review left its sixth open question open on a residue: sixteen upstream reference targets cited by the pre-rewrite specification appear nowhere in the rewritten one, the review restored the one it had already identified, and it recorded the remaining fifteen as a scheduled obligation rather than a question — the check belonging with the front matter's upstream-reference convention, where the qualified form is fixed (`plan:assayer:specification-rewrite-contract`). This document discharges the reading half of that obligation. It recomputes the census the audit reported (`data:assayer:spec-divergence-shape`), opens each remaining target against the corpus it points into, reads what the rewritten text says in the place the pointer left, and offers one considered opinion per target. Every opinion is a recommendation; the parts are not edited by this report, and no restore recommended below has been made.

The document practices the discipline it serves. The label at each heading or environment head is that environment's mint; a parenthesized label in running text is a same-owner citation; material in double-backtick spans is displayed without participating. Every label minted here has area `assayer`, every kind is drawn from a fixed registry of environment kinds, under the grammar of the label calculus. Upstream locators of the superseded sign-and-code form are written throughout in display font, never in prose, so that a report about a burn family does not feed the family it reports on.

## Method and census · `sec:assayer:dropped-upstream-method`

**Convention (What this report counts, and how a range counts)** · `conv:assayer:dropped-upstream-counting`

The family is the upstream document reference: the locator sign followed by a corpus code and a division locator, in the two forms the specification's own front matter fixes for the spectral Sentinel corpus and for the dual-tree value-stratified index corpus. Both editions were taken from git history and scanned mechanically, line by line, by one recognizer over the whole file; no occurrence of the family falls inside a fenced block in either edition, so the participating-prose restriction changes no number here.

One counting rule decides everything, and the audit's figures are reproducible only under it: **a range locator counts as one target, the first it names.** A citation written as a span from one division to another is a single reference to a region, and the divisions between its endpoints are not separately cited by it. The alternative rule — expanding a range into every division it covers — is also reported below, because the two rules disagree by three targets and a reader should be able to see which of them a number came from. A locator followed by a dash and a word, as one occurrence in the pre-rewrite text is, is not a range and is normalised to its locator.

**Data (The recomputed census)** · `data:assayer:dropped-upstream-census`

Recomputed 2026-08-19 from git history, against the pre-rewrite edition at 6,589 lines and the July edition at 6,084 lines, both read at the commits the audit names. Occurrences: **120 in the pre-rewrite text, 86 in the July text.** Distinct targets under the range rule above: **72 and 56.** Distinct targets dropped: **sixteen — ten of the Sentinel corpus and six of the index corpus — and none added.**

Every one of these figures reproduces the audit exactly, including the split between the two corpora, and the audit is confirmed rather than corrected. That is worth stating plainly, because two earlier recounts in this campaign moved a number by one and this one does not: the count is 16, the additions are 0, and the arithmetic the outline review inherited — sixteen dropped, one restored, fifteen outstanding — stands as written.

**Table (Distinct targets, by counting rule)** · `tab:assayer:dropped-upstream-rules`

| Rule | Pre-rewrite | July | Dropped | Added |
| --- | --- | --- | --- | --- |
| A range counts as its first target (the audit's rule) | 72 | 56 | 16 | 0 |
| Every written locator counts, ranges included as themselves | 75 | 57 | 18 | 0 |
| A range expands into every division it covers | 78 | 60 | 18 | 0 |

The three rules agree on what matters and differ at the margin. Under the expansion rule three further Sentinel subsections — the dual-shielding, benchmark-compounding and node-state sections of the spatial layer — lose their only mention, but each was only ever reached through the abstraction-boundary range and none was ever named on its own; and the eighth chapter of the Sentinel corpus stops counting as dropped, because the July text still carries a wider range that covers it. No rule produces a single added target. The rewrite subtracted from this family and added nothing to it.

**Register (The sixteen dropped targets, with their disposition)** · `reg:assayer:dropped-upstream-register`

Ordered by the site that lost them. The disposition column is this report's recommendation; the entries below give the grounds.

| Target | What the pre-rewrite text cited it for | Disposition |
| --- | --- | --- |
| ``§ALGO S-3.9`` | The capabilities a Sentinel uses of the graph | Discharged |
| ``§IDEA M-A`` | The graph's mathematical properties, assumed by the Sentinel | Discharged |
| ``§ALGO S-14.13`` | Contour queries, as a Sentinel surface the Core does not consume | Restore |
| ``§ALGO S-3.2`` | The contour, as the observation-receiving surface | Restore |
| ``§IDEA M-5.6`` | The same, in the index corpus's vocabulary | Restore |
| ``§ALGO S-8.3`` | The producing set, as the active analysis surface | Restore |
| ``§ALGO S-3.3`` | What importance means in the Sentinel's configuration | Restore |
| ``§IDEA M-3.2`` | What importance means in the index corpus | Restore |
| ``§ALGO S-3.5`` | Catalytic split and restoration, as named operations | Accept |
| ``§IDEA M-1.3`` | Refinement and eviction, as named operations | Accept |
| ``§IDEA M-3.5`` | Restoration, as a named operation | Accept |
| ``§IDEA M-1.2`` | Importance, as the second of a pair of pointers | Accept |
| ``§ALGO S-3.4`` | The competitive mechanism, in the background appendix | Accept |
| ``§ALGO S-3.6`` | Temporal attenuation, in the background appendix | Accept |
| ``§ALGO S-8`` | The head of the abstraction boundary's operation range | Defer |
| ``§ALGO S-3.11`` | The budget parameters a host configures | Restored already |

**Observation (The identity chapter's pointer, reconciled)** · `obs:assayer:dropped-upstream-restored-one`

The one target the outline review restored is the Sentinel corpus's configuration-parameters section, cited by the pre-rewrite host-duties list for the budget parameters a host must configure. It is the sixteenth row above and it is not open. The restoration landed in word form and is the model every restore recommended below should follow: the rewritten requirement says that the budget parameters are the ones the underlying graph's spatial layer defines and which the Sentinel algorithm document declares at its section 3.11, and it says why the pointer is there — a host told to configure a parameter and not told where it is defined has been told nothing (`req:keyspace:host-duties`). Fifteen targets remain, and the entries below rule on those fifteen.

**Observation (Where the upstream corpora actually are)** · `obs:assayer:dropped-upstream-corpora`

Every target below was opened in this repository before an opinion was formed, and both corpora are present. The Sentinel corpus is one file, and its numbering matches every Sentinel locator in both editions exactly: the spatial layer's twelve sections, the producing sets at the eighth chapter, and the contour queries and wavelet portrait at the end of the output chapter are all where the pre-rewrite text says they are.

The index corpus is **two files, sharing one locator space**, and this is a finding in its own right. Some locators resolve only in the formal specification — the bottom-contour section on plateaus, and the Fibonacci depth bound the background appendix still cites. Others resolve only in the coding-theoretic foundation, whose lettered appendix is the proof catalogue the pre-rewrite interface map pointed at, and whose budget invariant is the ceiling the background appendix still cites. Others resolve *plausibly in one and implausibly in the other*: the mutation section that names refinement, eviction and restoration, the codeword-lifecycle section, and the algebraic-interface section all sit in the foundation, while the same numbers in the formal specification land on the two trees, on integer and floating-point domains, and on nothing at all. A bare number therefore does not determine a target in that corpus. Every restore recommended below is accordingly written in word form and names its document, which the convention already requires and which this split makes load-bearing rather than stylistic (`conv:spec:upstream-references`).

One pointer of the pair resolves badly in both files. The importance row of the pre-rewrite terminology mapping cited two divisions of the index corpus for one claim, and only the second of them is about the value space where importance lives; the first is about what the code describes in one file and about the two trees in the other. That is a defect of the pre-rewrite citation, not of the rewrite, and it decides one of the accepts below.

## Restored already by the landed parts · `sec:assayer:dropped-upstream-discharged`

Two of the fifteen need no ruling, because the rewrite of the parts has already put them back. They are reported rather than passed over, because what is asked for is a count, and a count that quietly omits two is not a count.

**Record (The capability section of the Sentinel corpus)** · `rec:assayer:dropped-upstream-capabilities`

The pre-rewrite interface map's second interface was two sentences of pure pointer: the Sentinel uses the graph through the capabilities listed at one Sentinel section, and assumes the graph's mathematical properties as stated at another. The July rewrite deleted the whole subsection, keeping the number after it, so nothing in that edition marked the hole — the loss the audit reads as accidental (`reg:assayer:spec-divergence-losses`).

The target exists and is exactly what the sentence claimed: a thirteen-row table of the graph operations a Sentinel invokes, from routed observation to plateau iteration to node counts, each with its cost. The rewritten parts restore the interface, name the omission in their own preamble, and carry the pointer in word form as a table row: the capability section of the spectral Sentinel specification, which enumerates the operations a Sentinel invokes (`tab:boundary:graph-interface`). **Discharged.** No edit is wanted.

**Record (The properties appendix of the index corpus)** · `rec:assayer:dropped-upstream-properties`

The second half of the same deleted sentence. The target is the proof catalogue of the index corpus's coding-theoretic foundation, and the stated-assumptions end of the claim is the Sentinel corpus's own key-properties section, which opens by saying that the properties following are assumed throughout that specification. Both survive, and both are reached by the restored row in word form: the properties appendix of the dual-tree value-stratified index specification, assumed by the Sentinel at its stated-assumptions section (`tab:boundary:graph-interface`). **Discharged.** The word form is better than the locator it replaces, because the locator was ambiguous between two files of that corpus and the words are not.

## Restore · `sec:assayer:dropped-upstream-restore`

Six targets, at three sites. One principle separates these from the accepts, and it is worth stating before the entries because it is the whole of the reasoning: **a name correspondence needs no pointer, and a definition does.** A table that says only that one corpus calls a thing refinement and another calls it a catalytic split asserts nothing about either corpus beyond the name, and a reader who wants the mechanism knows which document to open. A table that says what an upstream term *means* has restated upstream material, and restating is the one thing the front matter's convention forbids: an upstream corpus is cited, never restated (`conv:spec:upstream-references`). Every restore below is a place where the rewritten text kept the restatement and dropped the citation.

**Record (Contour queries, and the completeness of an inventory)** · `rec:assayer:dropped-upstream-contour-queries`

The pre-rewrite not-consumed table had four rows, and the third named contour queries with the reason the Core does not read them: it maintains its own spatial index. The July edition deleted the row. The rewritten parts carry three rows and open with the claim that **three** published records are not read (`tab:boundary:not-consumed`), inside an appendix that declares itself normative precisely on completeness — the feed-forward invariant either has a complete inventory behind it or it does not.

The target exists and is a live surface: a four-query interface over the contour — point query, range query, iteration, plateau count — available at any time and explicitly independent of the observation cycle. It is exactly the kind of thing a Core could have consumed and does not, which is what the not-consumed table is for. **Restore**, and this is the most load-bearing of the six: an inventory that claims completeness while omitting an upstream surface is wrong in the one dimension it asserts. The restore belongs in the not-consumed table's own environment, but as a sentence after the table rather than as a fifth row, because that table's column is the record a Sentinel publishes and a query interface is not a record — the Sentinel corpus also publishes a contour query interface at its output chapter's contour-queries section, which the Core does not use because it maintains its own spatial index over the reported cells.

**Record (The contour, in the Sentinel corpus)** · `rec:assayer:dropped-upstream-contour-sentinel`

The pre-rewrite terminology mapping had six rows, and the fourth was the observation-receiving surface: the contour, named in both corpora's vocabularies and pointed into both, with the note that the Core does not observe it directly and reads only the aggregate snapshot. The July edition deleted the row, and the rewritten table carries four rows without it (`tab:terminology:structural`).

The target is the Sentinel corpus's domain-and-spatial-partitioning section, and it is where the term is defined: the bottom contour as the observation-receiving surface of the domain, a step function whose depth at each coordinate records how finely the tree has resolved that region, organised into plateaus. The rewritten specification uses the word *contour* in at least six places — the diagnostic snapshot in three of them, and the identity layer's decay table, which says that spatial decay governs how the contour evolves and who stays competitive — and defines it nowhere. The glossary cannot help: it is generated from the document's own mints and mints nothing (`app:spec:glossary`), so a term the document never defines can never appear there. **Restore.** The natural place is a fifth row of the structural mapping, restoring the deleted row with its pointer in word form; the alternative, a gloss at the decay table, scatters the definition to a use site instead of the translation surface built for it.

**Record (The contour, in the index corpus)** · `rec:assayer:dropped-upstream-contour-index`

The other half of the same deleted row. The target resolves in the index corpus's formal specification, at its bottom-contour section on plateaus, and not in the coding-theoretic foundation, whose fifth chapter stops short of that number — one of the cases that makes the two-file split matter. **Restore**, with the Sentinel half above and in the same row, naming the document: the bottom-contour section of the dual-tree value-stratified index specification. Restoring one half without the other would reproduce the pre-rewrite table's worst habit, which was to point one column into a corpus and leave the adjacent column's identical claim unsourced.

**Record (The producing set, as the active analysis surface)** · `rec:assayer:dropped-upstream-producing-set`

The fifth row of the pre-rewrite mapping, and the one with no counterpart in the index corpus: the producing set of the Sentinel — competitive targets plus online ancestors — against the Core's observable, which is whatever cells are in the current report. A note under the table drew the consequence: the producing set is a strict superset of the contour cells selected for analysis, it includes ancestors above the contour fed by multi-scale delivery rather than by spatial routing, and the Core observes the reported set and never the contour. The July edition deleted the row and the note together.

The target exists and is precisely stated — the producing sets as the online subsets of the investment set, with the slot-vacancy invariant that keeps a warming cell from occupying a production slot. **Restore**, at the same structural mapping, and rank it below the contour rows: the rewritten text does keep the conclusion that the Core reads reported cells and reasons about no Sentinel internal (`cav:limitation:abstraction`), so what the row adds is the reason the reported set is shaped as it is, not the rule itself. Load-bearing enough to want the row, not load-bearing enough to reopen a part on its own.

**Record (What importance means, in the Sentinel corpus)** · `rec:assayer:dropped-upstream-importance-sentinel`

The pre-rewrite importance table gave the Sentinel's row as importance under the standard configuration, equal to accumulated unit-delta observations, and sourced it. The rewritten table keeps the definition — accumulated unit-volume observation at a coordinate — merges the two upstream layers into one row, and carries no pointer at all (`tab:terminology:importance`).

The target is the Sentinel corpus's observation-and-importance section, which states the standard configuration in those words and adds the feed-forward invariant that makes it true: the Sentinel always observes with unit delta, and anomaly scores are never fed back into the importance signal. **Restore.** This row is the clearest instance of the principle above: it is not a name correspondence but a definition of an upstream term, stated in the Assayer's own voice, in a table whose first column is another corpus. Nothing checks it against the corpus it paraphrases, and the paraphrase is one configuration choice away from being wrong.

**Record (What importance means, in the index corpus)** · `rec:assayer:dropped-upstream-importance-index`

The same row's other pointer, and the one that resolves well: the algebraic-interface section of the index corpus's coding-theoretic foundation, which fixes the value space importance inhabits — carrier, aggregation, ground and ordering — with the base axioms and the additive-collapse theorem that makes the standard configuration the unique algebraic optimum rather than a taste. **Restore**, in the same row and naming the document. The merged row now speaks for two corpora at once; if it is to keep doing so it should say where each corpus states its half.

## Accept · `sec:assayer:dropped-upstream-accept`

Six targets. In each case the rewritten passage no longer leans on the pointer, and in two cases the passage is gone by a decision the parts state in their own text.

**Record (Catalytic split and restoration, in the Sentinel corpus)** · `rec:assayer:dropped-upstream-lifecycle-sentinel`

The pre-rewrite mapping pointed the split and restoration rows into the Sentinel corpus's spatial-lifecycle section. That section exists and covers all three forces — refinement, eviction, restoration — in the terms the table names. **Accept.** The rewritten row survives with both names intact and its fourth column, which is the only column the document owns, is a statement about the Core's own behaviour: a cell appears and a fresh Ledger entry is created (`tab:terminology:structural`). The table asserts a name and nothing more, and a name needs no source.

**Record (Refinement and eviction, in the index corpus)** · `rec:assayer:dropped-upstream-mutations-index`

Pointed at the coding-theoretic foundation's mutation section, which lists refinement, restoration and eviction as the three mutations of the partition and shows each preserving the Kraft equality — a good target, correctly cited, and not in the file the number would first suggest. **Accept**, on the same ground as the row above. The one residue worth noting is that this target is the only upstream section the rewritten background appendix still gestures at, where it says the index corpus states a Kraft-style equality over its contour and keeps that result imported rather than minting it (`app:spec:background`) — so the material is reachable, by a deliberate disposition, without this pointer.

**Record (Restoration, in the index corpus)** · `rec:assayer:dropped-upstream-restoration-index`

Pointed at the codeword-lifecycle section of the same foundation, which draws the whole cycle and names competitive promotion as the mechanism that recreates a missing child. **Accept.** It is the third pointer at one row of a name correspondence, and the row survives with the name. Two of the three targets below it in the pre-rewrite table said the same thing about the same operation in two vocabularies, which is what the table is for; the citations were redundant with the columns.

**Record (Importance, the second pointer of a pair)** · `rec:assayer:dropped-upstream-importance-second`

The pre-rewrite importance row cited two divisions of the index corpus. The second is the algebraic interface, recommended for restore above. The first resolves to the description-coding section in one file of that corpus and to the two-trees section in the other, and neither is about accumulated observation weight in a value space. **Accept the drop, and accept it as a gain.** The rewrite retired a pointer that did not support the claim beside it, and the restore recommended for that row supplies one that does. Restoring both would put the defect back.

**Record (The competitive mechanism, in the background appendix)** · `rec:assayer:dropped-upstream-competitive`

Cited twice in the pre-rewrite text: once in the background appendix's description of how the portrait's layers follow depth, and once inside the abstraction boundary's operation range, which is the deferred entry below. The July edition replaced the first with an internal tag of its own appendix, and the rewritten parts deleted that appendix's summary content altogether under a stated disposition: a summary of another corpus's material, given a head of its own here, would be an Assayer-owned statement about a system the Assayer does not own, and nothing checks a summary against its source (`app:spec:background`). **Accept.** There is no passage left to carry the pointer, and the disposition that removed the passage already stands.

**Record (Temporal attenuation, in the background appendix)** · `rec:assayer:dropped-upstream-attenuation`

The pre-rewrite background appendix described how cascade benchmarks attenuate at depth-dependent rates under attenuation with depth-selective decay, and sourced the temporal-decay section of the Sentinel corpus, which states the four regimes — attenuation, amplification, annihilation, detail flush — and the subband-adaptive rule. The July edition swapped the pointer for an internal tag; the rewritten parts deleted the passage with the rest of the appendix. **Accept**, on the same ground as the entry above. Where the rewritten specification does speak of decay it speaks of its own — the identity layer's three mechanisms and three rates — and cites its own environments for them.

## Defer · `sec:assayer:dropped-upstream-defer`

**Record (The abstraction boundary's operation range)** · `rec:assayer:dropped-upstream-boundary-range`

One target, and the only one whose fate this report declines to settle. The pre-rewrite interface map closed with an abstraction-boundary paragraph and a seven-item list of what lies on the Sentinel's side of the wall — node states, depth and promotion events, investment-set and warm-up state, the depth gates, contour and plateau organisation, benchmark compounding — whose last item was a pointer at two ranges of the Sentinel corpus, the spatial layer and the selection-and-assembly chapters. The head of the second range is the dropped target. The July edition deleted the paragraph and the list together, which is a divergence the audit's loss register does not carry — it registered the deleted interface subsection and not this — and the rewritten parts state the boundary as a rule for reading, with its reason and its consequence, and enumerate nothing (`conv:terminology:abstraction-boundary`).

**Defer**, and to a named campaign: the record rewrites (`reg:assayer:decision-record-register`), whose set includes the record that owns the feed-forward boundary. The reasoning is that the pointer cannot be ruled on apart from the enumeration it terminated, and the enumeration is a boundary inventory rather than a reading rule — the same kind of thing as the interface map's verification table, and the kind of thing a decision record holds better than an expository appendix. Two things follow. The loss of the enumeration should be registered as a loss candidate on its own account, because it is not in the register that was supposed to hold every one. And if the ruling instead places the enumeration in the specification's terminology convention, then this target converts to a restore of one clause naming the Sentinel corpus's spatial-layer chapter and its selection-and-assembly part, and it is cosmetic rather than load-bearing.

## Verdict · `sec:assayer:dropped-upstream-verdict`

**Summary (The arithmetic, and what wants an edit soon)** · `summ:assayer:dropped-upstream-summary`

Sixteen targets dropped and none added, confirming the audit exactly. One restored by the outline review, leaving fifteen. Of the fifteen: **two discharged** by the parts as landed, **six restore**, **six accept**, **one defer**. Two plus six plus six plus one is fifteen.

The six restores fall at three sites and not six, which is the practical number, and they are not equally urgent.

| Site | Targets | Weight |
| --- | --- | --- |
| The not-consumed table of the interface map | one | Load-bearing: the appendix claims a complete inventory and omits an upstream surface |
| The importance row of the terminology mapping | two | Load-bearing: an upstream definition is restated in the Assayer's voice with nothing sourcing it |
| The structural mapping of the terminology appendix | three | Two load-bearing, one cosmetic: a term the document uses six times and defines nowhere, plus one expository row |

Five of the six are load-bearing and one — the producing-set row — is cosmetic. The load-bearing five reduce to two sentences and one table row, and both edits sit inside environments the parts phase has already landed (`spec:spec:measurement-judgement-interpreter`); neither reopens a chapter, and neither changes an obligation on the host. Against that, the accepts are not close calls: four of the six are rows that survived with their names intact, and two are pointers whose passages were deleted by a disposition the parts state in their own text.

One caution belongs with the verdict. The rewritten parts carry upstream locators in six places in total, where the July edition carried eighty-six occurrences of the sign-and-code form. Almost all of that reduction is the deliberate work of the convention — word form in place of a locator, a citation of a corpus in place of a restatement of it — and the restores recommended here are not a proposal to reverse it. They are five places where the restatement stayed and the citation did not, which is the one combination the convention does not admit (`conv:spec:upstream-references`).
