# The Assayer Specification Audit · `rep:assayer:specification-rewrite-basis`

This is the audit of `docs/spec.md` as found, the first entry of the specification pipeline in [the campaign backlog](backlog.md) (`rep:assayer:specification-rewrite-basis`). It counts what the document actually contains — its sections, its reference forms, its overlap with the pre-rewrite residue, the environments it carries without naming them, the line between what binds the implementation and what explains it — and it checks the document's claims against the code rather than against its own preamble. Nothing in the specification is edited by this wave. The audit's findings are the parameters of the labelled outline (`plan:assayer:specification-rewrite-contract`), and the verdict section states exactly which of them that entry must carry forward.

The document practices the discipline it serves. The label at each heading or environment head is that environment's mint; a parenthesized label in running text is a same-owner citation; material in fenced blocks is displayed without participating. Every label minted here has area `assayer`, and every kind is drawn from a fixed registry of environment kinds, under the grammar of the label calculus. Legacy tag forms of the superseded scheme (`dec:assayer:tags-superseded`) are written here as plain text, never in backticks, precisely so that a float tag of the old system cannot mint under the new one.

## Method and measurement basis · `sec:assayer:spec-audit-method`

**Convention (What the audit counts)** · `conv:assayer:spec-audit-counting`

Counts are taken over participating prose only: fenced code blocks are excluded from every reference and environment count, because the specification carries fifty of them and they hold Rust signatures rather than references. A *tag occurrence* is a backtick span whose interior matches the two-level grammar of `docs/tags.md`; a *mint site* is such an occurrence at a heading, a bold paragraph lead, a bold list-item lead, a design-note blockquote, a standalone caption line, or a table cell holding nothing but the tag and its optional status grade; every other occurrence is a citation. Section-number forms are counted separately by shape. Drift claims cite ``path:line`` in `packages/assayer/src` and were read, not inferred.

**Data (Measurement basis)** · `data:assayer:spec-audit-basis`

Measured 2026-08-19 against the tree at backlog commit `888d50b8`. Subjects: `docs/spec.md` at 6,084 lines, rewritten 2026-07-06; `docs/spec.old.md` at 6,589 lines, pre-rewrite; and `docs/tags.md` at 511 lines, the superseded scheme retained as a concept outline under the retirement ruling (`dec:assayer:scaffolding-retirement`). Code was checked at `packages/assayer/src`, 690 files in the test profile's census.

**Observation (The specification does not participate today)** · `obs:assayer:spec-nonparticipation`

The linter's adoption data names `packages/assayer/docs/spec.md` and `packages/assayer/docs/tags.md` as pre-calculus documents: both are carried as bytes and form no minting or resolution judgment, which is why the repository check is clean at 271 mints while the specification alone carries 1,727 tag occurrences. Two consequences bind later waves. The commit that migrates the specification must remove its entry there in the same commit, or the rewritten document will lint vacuously. And `docs/spec.old.md` is *not* listed — it participates, and is clean only because it contains no backtick span that parses as a label at all. It is one float-tag edit away from minting into the Assayer's owner, which is a further reason to relegate it on the schedule the retirement ruling (`dec:assayer:scaffolding-retirement`) sets rather than to leave it indefinitely.

## Section inventory · `sec:assayer:spec-audit-inventory`

**Data (Headline shape)** · `data:assayer:spec-inventory-shape`

441 heading lines: one title, one subtitle, eight Part heads, one "Appendices" head, 30 chapters, nine appendices, and 391 subsections nested three to five levels deep. Beside the headings the document carries 169 tables, 110 display-math blocks, and 50 fenced code blocks. The heading ladder is the *only* structural device: there is no boxed environment, no numbered theorem, no environment head of any kind.

**Table (Parts and chapters)** · `tab:assayer:spec-inventory-chapters`

Line counts are the span from each head to the next unit head. "Subs" counts subsections below the chapter; "mints" counts tag mint sites.

| Unit | Line | Lines | Subs | Tables | Math | Subject |
| --- | --- | --- | --- | --- | --- | --- |
| Front matter (title, cross-document reference note) | 1 | 19 | 0 | 1 | 0 | external reference forms; the two-level tag convention; the three components |
| Part I — Foundations | 20 | 4 | — | — | — | part epigraph |
| Ch. 1 Purpose, Scope, and Principles | 24 | 153 | 6 | 3 | 1 | core insight; three principles; feed-forward invariant; architecture; structural constraints; assumed Sentinel properties |
| Ch. 2 The Encoding Contract | 177 | 86 | 7 | 3 | 1 | the host's encoding responsibility; three suitability questions; suitable and unsuitable sources; curves; fallback; domain width |
| Ch. 3 The Valence Asymmetry | 263 | 67 | 4 | 0 | 0 | censored-bandit asymmetry; five mitigation layers; contamination loop; outcome-axis pathway |
| Ch. 4 Mathematical Foundation | 330 | 93 | 5 | 1 | 5 | Gaussian extension, marginalisation, conditioning; Schur regularisation; costs; two-level posterior guarantee; precision monitoring |
| Part II — The Data Model | 423 | 8 | — | — | — | part epigraph |
| Ch. 5 Registries and Lifecycle | 431 | 293 | 27 | 6 | 0 | Sentinel, outcome-axis, and identity-dimension registries; registration and deregistration protocols; partial coverage; hibernation; serialisation |
| Ch. 6 Per-Sentinel Structured Extraction | 724 | 107 | 8 | 6 | 2 | chain z-scores, CUSUMs, structure, coordination, Ledger features, batch context; slot width |
| Ch. 7 The Key Space Identity Layer | 831 | 273 | 15 | 7 | 3 | dimension registry; encoding extension; observation protocol; competitive set; per-dimension and cross-dimension features; caches; measurement and outcome state; decay; lifecycles; host duties |
| Ch. 8 Host Signals | 1104 | 45 | 2 | 0 | 0 | signal shapes and persistence; the fixed-at-construction schema |
| Ch. 9 The Feature Vector | 1149 | 443 | 26 | 9 | 9 | vector structure and dimension formula; aggregate block; interaction template system; dimension map and compilation pipeline; online standardisation |
| Part III — The Core Models | 1592 | 8 | — | — | — | part epigraph |
| Ch. 10 The Core Risk Models | 1600 | 387 | 26 | 8 | 21 | the model triple; anchor; subspace-restricted blend; derived estimates; Platt calibration; eligibility; importance weighting; risk target; leverage-bounded update; forgetting rates |
| Ch. 11 Outcome Axis Prediction Models | 1987 | 66 | 5 | 1 | 3 | per-axis model; training target; inference outputs; cross-axis prediction; lifecycle |
| Ch. 12 The Outcome Ledger | 2053 | 300 | 21 | 8 | 4 | purpose and value; per-entry state; attenuation; update rules; decay; entry lifecycle; contamination and starvation; root semantics; low-maturity fallback |
| Part IV — The Decision Landscape | 2353 | 8 | — | — | — | part epigraph |
| Ch. 13 The Derivation Function Interface | 2361 | 158 | 6 | 0 | 0 | the function signature; inputs; output structure; purity guarantee; utilities; optional rendering layer |
| Ch. 14 Channel Policy | 2519 | 81 | 8 | 3 | 1 | posture as cursor; action-space sizing; reward parameters and sensitivity; other policy fields |
| Ch. 15 The Crossover Landscape | 2600 | 201 | 12 | 7 | 11 | posture-sensitivity; per-action costs and differentials; rigid-translation decomposition; crossover derivation; dominance; covariance; intervals; neutrality |
| Ch. 16 Worked Landscapes | 2801 | 151 | 5 | 12 | 0 | four worked examples and a design summary |
| Ch. 17 Exploration and Label Guidance | 2952 | 109 | 8 | 1 | 2 | decision fragility; core label guidance interface and scoring; composition |
| Part V — The Companion Tracker | 3061 | 8 | — | — | — | part epigraph |
| Ch. 18 Challenge Effectiveness Tracking | 3069 | 321 | 14 | 8 | 6 | purpose, state, prior, inference, update, contributing rate, decay, override, injection, health, convergence, boundary, configuration, replacement |
| Part VI — The Operational Cycle | 3390 | 8 | — | — | — | part epigraph |
| Ch. 19 Runtime Declarations and the Host Contract | 3398 | 119 | 8 | 2 | 0 | construction; eligibility policy; axis registration; schema declaration; label reporting; valence convention; pre-seeding; severity note |
| Ch. 20 The Assessment Interface | 3517 | 275 | 13 | 1 | 1 | report reception; the assessment call; the pipeline; coordinate routing; the assessment structure; alarm summary; composition |
| Ch. 21 The Label Interface and Learning Pipeline | 3792 | 149 | 9 | 1 | 0 | label interface; full update path; identity outcome update; per-label cost; pending buffer and reconstruction |
| Ch. 22 Concurrency | 3941 | 121 | 10 | 3 | 0 | governing principle; published state; draining; Ledger concurrency; staleness bounds; lifecycle; buffer; label processing; report reception; guarantees |
| Ch. 23 Temporal Governance | 4062 | 85 | 5 | 2 | 4 | two decay mechanisms; the decay inventory; the lazy mechanism; per-component rates; Ledger decay |
| Part VII — Properties and Analysis | 4147 | 8 | — | — | — | part epigraph |
| Ch. 24 System Initialisation and Warm-Up | 4155 | 237 | 15 | 13 | 2 | six warm-up stages; bootstraps; end-to-end convergence budget; milestones; pre-calibration awareness; recovery timelines |
| Ch. 25 Convergence and Resource Bounds | 4392 | 96 | 6 | 5 | 0 | convergence budget; incremental additions; per-assessment and per-label cost; memory; reference configuration |
| Ch. 26 Detection Analysis | 4488 | 65 | 7 | 2 | 1 | the detection stack; compound detection; avoidance dilemma; convergence window; intervention-effectiveness measurement |
| Ch. 27 Health Monitoring | 4553 | 223 | 34 | 6 | 10 | drift detection; discrimination; precision health; label-integrity monitoring; cross-layer observability; host-mediated feedback scenarios |
| Part VIII — Reference | 4776 | 4 | — | — | — | part epigraph |
| Ch. 28 Configuration | 4780 | 207 | 16 | 19 | 0 | thirteen core configuration tables plus derivation and Companion configuration |
| Ch. 29 Output Structures | 4987 | 77 | 4 | 3 | 0 | the four output surfaces, restated from their defining chapters |
| Ch. 30 Guarantees and Known Limitations | 5064 | 107 | 3 | 2 | 0 | 29 formal properties; the information-flow figure; 40 known limitations |

**Table (Appendices)** · `tab:assayer:spec-inventory-appendices`

| Appendix | Line | Lines | Subs | Subject |
| --- | --- | --- | --- | --- |
| A. Resonance Rendering (Analysis Layer) | 5175 | 166 | 9 | the optional display layer: contract, spectrum, kernel, placement, bandwidths, magnitudes, domination, examples, utilities |
| B. Extraction Reference Table | 5341 | 27 | 0 | the per-Sentinel extraction index table |
| C. Dimensional Summary | 5368 | 32 | 0 | the dimension accounting and its worked construction |
| D. Glossary | 5400 | 45 | 0 | symbol and term definitions |
| E. Honest Notes Register | 5445 | 16 | 0 | the limitation-grade tallies, by grade |
| F. Interface Map | 5461 | 51 | 3 | Sentinel-to-Core consumption; symbol cross-reference; feed-forward boundary verification |
| G. Cross-Layer Terminology Mapping | 5512 | 27 | 3 | structural operations; the "importance" term; the abstraction boundary |
| H. Background: The Spatial Index and the Spectral Sentinel | 5539 | 535 | 40 | summaries of two upstream components, including the wavelet portrait in ten sub-subsections |
| I. Tag Index | 6074 | 12 | 0 | a description of an index; see (`obs:assayer:spec-tag-index-empty`) |

**Observation (The numbering already lies)** · `obs:assayer:spec-numbering-holes`

Three heading numbers in the rewritten document name nothing. Chapter 9 runs 9.1 then 9.4: the old 9.2 "Fixed Blocks" and 9.3 "Dynamic Blocks" were demoted to bold-lead tables inside 9.1 (spec.md:1161–1178) and their numbers were not reclaimed. Appendix F runs F.1, F.1.1, F.3: F.2 was deleted and F.3 kept its number. Appendix F.1.1 is a level-three head under a level-three head. This is the locator scheme failing inside a single revision of a single document, in exactly the way the supersession ruling (`dec:assayer:tags-superseded`) predicts, and it is the cheapest available argument for the outline wave's premise: numbering carries no identity and the outline must not reproduce it.

## Reference census · `sec:assayer:spec-audit-references`

**Table (Reference forms in the rewritten specification)** · `tab:assayer:spec-reference-forms`

Occurrences over participating prose. "Distinct" counts distinct matched strings.

| Form | Shape | Occurrences | Distinct | Example |
| --- | --- | --- | --- | --- |
| Semantic tag | backticked namespace:slug | 1,640 | 425 | land:rigid, inv:feedforward |
| Float tag | backticked kind:namespace:slug | 87 | 38 | tbl:enc:suitable, eq:land:rigid, fig:arch:flow |
| Status grade | mid-dot plus grade word | 113 | 5 grades | · uncond, · emp, · struct, · host, · cond |
| Section-sign locator | section sign plus number | 5 | 5 | ``§10.3``, ``§20.4.3``, ``§A.1`` |
| Backticked appendix locator | backticked letter.number | 5 | 3 | A.3, A.9, G.1 |
| Chapter pointer | the word Chapter plus number | 61 | 34 | Chapter 10, Chapters 15 |
| Part pointer | the word Part plus roman numeral | 24 | 8 | Part IV |
| Appendix pointer | the word Appendix plus letter | 17 | 9 | Appendix B |
| Upstream document reference | section sign plus ALGO or IDEA code | 86 | 56 | ``§ALGO S-14.2``, ``§IDEA M-6.4`` |
| Decision-record reference | ADR-x-NNN | 0 | 0 | — |
| Markdown link | bracketed link | 0 | 0 | — |

Three facts govern the conversion burden. First, the rewrite already did the hard sweep: `spec.old.md` carries 791 section-sign locators over 236 distinct targets and zero tags, while `spec.md` carries five locators and 1,727 tag occurrences. The residual section-number problem in the specification is therefore small — 112 pointers of the Chapter, Part and Appendix word forms, five section signs, five backticked appendix locators. Second, the real conversion is tag-to-label, at a scale of 1,727 occurrences over 463 distinct identities. Third, the 86 upstream references are *not* the migration's business: they name documents of other corpora that the calculus reaches only through an import, and the specification's own front matter already fixes their qualified form.

**Data (Tag resolution inside the specification)** · `data:assayer:spec-tag-resolution`

463 distinct tags in 1,727 occurrences. 456 have at least one mint site; the mint sites are distributed as 232 at headings, 102 at bold paragraph leads, 78 in table cells, 30 at bold list-item leads, 11 in design-note blockquotes, and 3 as standalone caption lines. Seven distinct tags are cited and never minted anywhere. 153 tags are minted and never cited — a third of the register, concentrated in Appendix H (30 of them), the configuration tables, and the formal-property rows of Chapter 30, whose identities exist so that other documents may cite them.

**Register (Tags cited with no mint site)** · `reg:assayer:spec-dangling-tags`

Seven, verified individually by reading every occurrence:

- mot:division, mot:consulted, mot:bonus — the three mottos of the old scheme's section 2.18 are cited in running prose (spec.md:54, :5508, :2347, :2069) and were never given a mint site anywhere.
- led:materiality — cited at spec.md:2069; the dual-condition materiality result it names is stated at (`data:ledger:attenuation`) without a mint.
- model:blend-mech — cited at spec.md:1670 for the two persistence mechanisms, which are then stated in the following paragraph unminted.
- tbl:cfg:rend — cited twice (spec.md:4957, :5214) for a rendering configuration table that Chapter 28 does not contain; see (`obs:assayer:drift-rendering-config`).
- land:iv — cited at spec.md:6076 as the scheme's own worked example of a deliberately reserved, never-minted slug. This one is correct by construction and is listed only so the count is complete.

**Observation (Appendix I is an empty promise)** · `obs:assayer:spec-tag-index-empty`

Appendix I declares itself "the binding surface for citation", specifies a row format, enumerates five build checks — every tag resolves to exactly one definition site, every cited tag appears in the index, every invariant and limitation row carries a grade, every float names an extant owner, no slug is an ordinal — and states an index invariant that a restructure of any magnitude touches only the locator column. It then contains no rows. It says the index "is maintained as a generated artefact … emitted from the definition sites during the build"; no such build step exists in the workspace, and the one checker that does exist excludes this document by name (`obs:assayer:spec-nonparticipation`). The six defects of (`reg:assayer:spec-dangling-tags`) are precisely what the first of those five checks would have caught. This is the campaign's standing rule (`goal:assayer:tools-over-discipline`) illustrated from the inside: a discipline described but not enforced decayed within one revision of the document that described it.

**Requirement (Shapes the legacy-reference lint must recognise)** · `req:assayer:spec-legacy-lint-shapes`

The legacy-reference lint is parameterized by this census. Over Assayer prose it must find, once a document is declared migrated: backtick spans matching namespace:slug and kind:namespace:slug; the mid-dot status-grade suffix, which is the tag scheme's own vocabulary and has no counterpart in the calculus; the section sign followed by a number; the word forms Chapter, Part and Appendix followed by a number or letter, which are 112 of the 117 in-document locators and would be missed by a section-sign-only rule; backticked appendix locators of the form letter.number, which look like citations and are not; and ADR-x-NNN by number. It must *not* find the upstream forms with ALGO and IDEA codes, which remain valid. The lint's per-document configuration matters here: `docs/spec.old.md` and `docs/tags.md` are saturated with the forms it hunts and must stay unconfigured until they leave the tree.

## Overlap and divergence against the pre-rewrite specification · `sec:assayer:spec-audit-divergence`

This section is the recorded cross-audit that the retirement ruling (`dec:assayer:scaffolding-retirement`) requires before `spec.old.md` is relegated to git history. It is a first pass at chapter granularity, not a line-level reconciliation; the outline review (`plan:assayer:specification-rewrite-contract`) inherits the loss candidates.

**Data (Structural difference)** · `data:assayer:spec-divergence-shape`

`spec.old.md`: 6,589 lines, 457 heading lines. `spec.md`: 6,084 lines, 441 heading lines. The Part structure, the chapter count, and the chapter subjects are identical outside Part IV; the parts, Chapters 1–14 and 17–30, and Appendices B–H stand in one-to-one correspondence. The whole of the divergence is concentrated in Part IV and its appendix, plus four smaller edits. Upstream references fell from 120 occurrences over 72 distinct targets to 86 over 56; sixteen distinct upstream targets present in the old document appear nowhere in the new one, and none was added.

**Table (Deliberate restructure, Part IV and Appendix A)** · `tab:assayer:spec-divergence-editorial`

The rewrite inverted the primacy of the decision landscape and the resonance rendering. Each row was verified by reading both texts.

| Pre-rewrite | Rewritten | Reading |
| --- | --- | --- |
| Ch. 15 The Resonance Model | Ch. 15 The Crossover Landscape | the crossover structure becomes the primary object |
| Ch. 16 Resonance Derivation (13 sections) | Ch. 16 Worked Landscapes; derivation absorbed into Ch. 15 and App. A | the display derivation is demoted to an appendix |
| ``§16.2.5`` p-independence of interior regime widths | the rigid-translation decomposition (`thm:landscape:rigid-translation`) | restated and strengthened into a named result |
| ``§16.11`` catching-forfeiture sensitivity | the catching-forfeiture threshold (`thm:landscape:catching-forfeiture`) | restated in the cost model |
| ``§16.9`` Challenge Effectiveness Input | the challenge posterior among the inputs (`sig:companion:posterior`) | restated; the uninformative prior becomes Beta(1,1) |
| ``§17.1`` Information Value (3 subsections) | Decision Fragility (`sec:fragility:decision`), two subsections | a deliberate replacement; the old scheme records the retirement of the information-value identity |
| ``§17.3`` Action-Informative Label Guidance | Composing Fragility with Core Guidance (`ex:fragility:composition`) | replaced |
| App. A Resonance Mathematics (4 sections) | App. A Resonance Rendering (9 sections) | the appendix absorbs the display derivation; A.1–A.4 are restated within it |
| ``§27.2.1`` AUC, ``§27.2.2`` Recent AUC | AUC, merged (`alg:monitoring:auc`) | merged, and the empirical coverage diagnostic (`alg:monitoring:empirical-coverage`) added |
| ``§20.4.7`` Report Index and Concurrency | Report Reception (`req:publication:report-reception`) | moved to the concurrency chapter, restated |
| ``§22.3`` Identity Dimension Concurrency | Deferred Identity Observation Draining (`alg:publication:identity-draining`) | narrowed and renamed |
| ``§29.2`` Derivation Output: Reckoning | Derivation Output: DecisionLandscape (`chap:spec:output-structures`) | the output type is renamed; see (`warn:assayer:drift-landscape-unimplemented`) |
| — | Reputation Laundering and Symmetric Decay (`disc:ledger:laundering`) | added |
| — | The Rendering Layer, optional (`dec:landscape:rendering-optional`) | added |
| — | App. I Tag Index | added; see (`obs:assayer:spec-tag-index-empty`) |

**Register (Loss candidates: present in the old text, absent from the new)** · `reg:assayer:spec-divergence-losses`

Each was searched for in the rewritten document by distinctive string and by subject, not by section number. Ordered by how accidental the loss looks.

- **Appendix F.2, the G-V Graph to Sentinel interface.** Deleted outright (spec.old.md:5949–5951), taking with it the only pointers to the upstream capability list and to the graph's assumed mathematical properties. F.3 kept its number, so nothing in the document marks the hole. The content was two sentences of pure pointer, but the appendix's stated job is a complete interface map, and one of its three interfaces is now missing. **Reads accidental.**
- **The dependency-ordering statement (old ``§16.4.4``, Computation Order).** Six steps fixing the order in which crossovers, bandwidths, magnitudes, interior locations, boundary locations and classification tags must be computed, with the note that the chain is acyclic. Nothing in Chapter 15 or Appendix A states an order. This is the one piece of old Chapter 16 that was implementable guidance rather than derivation, and it is the piece that vanished. **Reads accidental.**
- **The quantitative bandwidth-floor threshold (old ``§16.7.4``).** The old text gives the exact condition under which the display floor binds and the cold-start figure at which it does. The new A.5 keeps the floor and replaces both numbers with "it may bind". A display-layer number is a small loss, but it is a loss of falsifiability, not of prose. **Reads editorial, worth a deliberate ruling.**
- **The three-mechanism domination table (old ``§16.5``).** Crossover inversion, existence failure and catching forfeiture as three named conditions in one table. All three survive in the new A.7 as prose clauses; only the table is gone. **Reads editorial.**
- **The per-crossover sensitivity table (old ``§16.7.2``).** The derivative of each crossover with respect to the challenge estimate, per transition, with the numeric contributions at defaults. The new crossover covariance (`thm:landscape:crossover-covariance`) gives the rank-2 structure instead, which is stronger; the worked per-transition numbers are gone. **Reads editorial.**
- **Sixteen upstream reference targets.** Ten ALGO and six IDEA targets cited by the old text appear nowhere in the new one; several belong to the deleted F.2 and to the abstraction-boundary list. The outline review should confirm none of them was load-bearing.

Nothing else in the two documents diverges materially: the old ``§9.2`` and ``§9.3`` content survives as bold-lead tables (`obs:assayer:spec-numbering-holes`), and every other apparent deletion checked in this pass was a move.

## Implicit environments · `sec:assayer:spec-audit-environments`

**Data (Environment-shaped material and its present carriers)** · `data:assayer:spec-implicit-shapes`

The specification carries no environment heads. Its environment-shaped material is distributed across five carriers: 343 bold-lead paragraphs (102 of which already carry a tag, and are therefore already environments in all but form); 169 tables; 110 display-math blocks; 50 fenced code blocks; and 11 design-note and deployment-note blockquotes. The 232 heading-borne tags are the fifth carrier and the largest: a heading whose tag is ``inv:feedforward`` or ``land:rigid`` is an environment head wearing a section number.

**Table (Kind assignment for the implicit environments)** · `tab:assayer:spec-implicit-kinds`

Kinds are from the registry of environment kinds. The counts are the audit's estimate of how many environments of each kind the rewritten specification will carry; they are inputs to the outline, not a commitment.

| Present form | Kind | Approx. | Evidence |
| --- | --- | --- | --- |
| The 29 guarantee rows, each a stated system property with a grade | `inv` | 29 | (`chap:spec:guarantees-and-limitations`); the rows already carry identity and grade and want only a head |
| Named algebraic results: extension, marginalisation, the two-level posterior guarantee, the rigid-translation decomposition, dominance, blend variance, the Kraft equality | `thm` or `prop` | 10–15 | (`thm:gaussian:extension`), (`thm:gaussian:marginalisation`), (`thm:gaussian:two-level-guarantee`), (`thm:landscape:rigid-translation`), (`thm:landscape:dominance`), (`prop:risk:blend-variance`), (`app:spec:background`) |
| Statements the host must satisfy: the encoding contract, the valence sign convention, label reporting, ground-truth honesty | `req` | 6–10 | (`req:encoding:host-contract`), (`conv:valence:sign`), (`tab:host:label-reporting`), (`conv:eligibility:ground-truth`); all carry the host grade today |
| Fixed meanings: fragility, the risk target, the competitive set, the slot, starvation score, posture, the effective triple | `def` | 30–45 | (`def:fragility:definition`), (`def:risk:target`), (`def:keyspace:competitive-set`), (`def:extraction:slot`), (`def:ledger:starvation-score`), (`def:channel:posture`), (`def:risk:model-triple`) |
| Numbered protocols: registration, deregistration, the assessment pipeline, the label update path, reconstruction, the standardisation update | `alg` | 10–14 | (`alg:registry:sentinel-registration`), (`alg:registry:sentinel-deregistration`), (`alg:runtime:assessment-pipeline`), (`alg:runtime:update-path`), (`alg:runtime:reconstruction`), (`alg:standardisation:label-time-procedure`) |
| Registry and reference tables: configuration, extraction layout, dimensional summary, decay inventory, staleness bounds, concurrency tiers, detection stack | `tab` | 25–35 | (`chap:spec:configuration`), (`app:spec:extraction-reference`), (`app:spec:dimensional-summary`), (`tab:temporal:decay-inventory`), (`tab:publication:staleness-bounds`), (`tab:publication:tiers`), (`tab:detection:stack`) |
| The 40 limitation rows (`chap:spec:guarantees-and-limitations`) and Appendix E | `cav` | 40 | a limitation is a bound on the claim, not a property of the system |
| Worked examples and rendered examples | `ex` | 8–12 | (`chap:spec:worked-landscapes`), (`ex:warmup:profiles`), (`ex:warmup:recovery`), (`ex:rendering:examples`) |
| Design and deployment notes, which the old scheme itself identified as carrying real decisions | `rem` or `dec` | 11 | the blockquote notes at (`rem:encoding:downstream-fallback`), (`rem:extraction:order-statistic-bias`), (`rem:extraction:lean-extraction`), (`rem:risk:blend-mechanisms`), (`dec:ledger:no-adaptive-rate`), (`dec:companion:decay-form`) |
| Conventions: offset convention, axis-index convention, valence signs, symbol usage | `conv` | 6–10 | (`conv:extraction:offsets`), (`conv:dimension:axis-index`), (`conv:valence:sign`), (`app:spec:glossary`) |
| Scenario analyses of host-mediated feedback | `scenario` | 3 | (`scenario:monitoring:atrophy`), (`scenario:monitoring:displacement`), (`scenario:monitoring:starvation`) |
| The information-flow figure and the architecture diagram | `fig` or `diag` | 2 | (`fig:architecture:information-flow`), (`fig:architecture:overview`) |
| Structural constraints stated as standing assumptions | `assum` | 5 | (`chap:spec:purpose-and-principles`) |
| Costs and resource bounds stated as estimates | `bound` | 6–8 | (`tab:gaussian:operation-costs`), (`tab:resource:assessment-cost`), (`tab:resource:label-cost`), (`tab:resource:memory`) |

**Caveat (What must not be promoted)** · `cav:assayer:spec-implicit-overreach`

Three temptations, each of which would inflate the outline. The 343 bold-lead paragraphs are not 343 environments: most are typographic emphasis inside a running argument, and only the 102 that already carry a tag have any evidence of independent citability. The 110 display-math blocks are overwhelmingly steps inside a derivation and want the `eq` treatment of a display, not an environment each. And Appendix H, at 535 lines and 40 subsections, is a summary of *other* corpora; under the calculus its material wants imported citations into Mudlark and Sentinel far more than it wants 40 Assayer-owned environments.

## Normative and expository partition · `sec:assayer:spec-audit-partition`

**Observation (The specification has no modal vocabulary)** · `obs:assayer:spec-no-modal-vocabulary`

The document uses no capitalised requirement keywords at all: zero occurrences of MUST, SHALL, or REQUIRED. It carries 27 lower-case "must", 79 uses of "never" or "always", and 81 mentions of a default. What binds the implementation is therefore not marked in the text: it is inferred from position — a table of defaults binds, a paragraph of motivation does not. The rewritten specification cannot inherit this, because the kind at each head is exactly where the binding force will be declared: an `inv`, a `req`, a `def` and a `tab` bind; a `rem`, a `mot`, an `ex` and a `disc` do not. The partition below is the audit's reading of which is which today.

**Table (Partition by unit)** · `tab:assayer:spec-partition`

| Unit | Reading |
| --- | --- |
| Ch. 1 | mixed: the principles (`prin:principle:measure-not-decide`) and the standing assumptions (`assum:constraint:dynamic-registries`) bind; the three questions (`mot:architecture:three-questions`), the component ownership table (`tab:architecture:ownership`) and the architecture figure (`fig:architecture:overview`) explain |
| Ch. 2 | normative: the contract, the width rule and the fallback bind; the suitability tables are guidance |
| Ch. 3 | expository throughout: the asymmetry is an argument for the design, not a constraint on it |
| Ch. 4 | normative: the algebra, the regularisation and the guarantee bind |
| Ch. 5–9 | normative: registries, protocols, feature widths, the dimension map and the standardisation procedure are the data model |
| Ch. 10–12 | normative: the models, calibration, eligibility, weighting and the Ledger rules bind; the design notes inside them explain |
| Ch. 13–15 | normative: the signature, the inputs, the output, purity, the cost model and the crossover derivation bind |
| Ch. 16 | expository: four worked examples and a summary |
| Ch. 17 | normative: fragility and the guidance interface bind; the interpretation bands explain |
| Ch. 18 | normative, with an expository convergence discussion (`bound:companion:convergence`) |
| Ch. 19–23 | normative: the host contract, the pipelines, the buffer, the concurrency tiers and the decay inventory are the operational contract |
| Ch. 24 | mixed: the milestone table and the bootstraps bind; the profiles and recovery timelines are worked illustration |
| Ch. 25–26 | expository: cost and detection analysis, save the reference configuration, which binds |
| Ch. 27 | normative in what it requires to be reported, expository in its interpretation guidance |
| Ch. 28 | normative: the defaults are the configuration surface |
| Ch. 29 | derivative: it restates output structures defined elsewhere and binds nothing of its own |
| Ch. 30 | normative in its guarantees and expository in its limitations (`chap:spec:guarantees-and-limitations`), derivative in the information-flow figure (`fig:architecture:information-flow`) |
| App. A | normative within the rendering layer, which is itself optional |
| App. B, C | normative reference tables |
| App. D, E, G, I | expository |
| App. F | normative: it is the boundary verification |
| App. H | expository, and about other corpora |

**Observation (Chapter 29 is pure restatement)** · `obs:assayer:spec-restatement-chapters`

Chapter 29 restates four output structures already defined in Chapters 13, 18 and 20, and Appendix E restates the grade tallies of the guarantees chapter (`chap:spec:guarantees-and-limitations`). Under the calculus a restatement names nothing new — it refers — so neither wants environments of its own. Both are candidates for generated registers rather than authored prose, which is the shape (`entry:assayer:tool-generated-registers`) exists to support. The outline should decide their fate explicitly rather than transcribing them.

## Implementation drift · `sec:assayer:spec-audit-drift`

**Convention (How drift was checked and reported)** · `conv:assayer:spec-drift-method`

Claims were spot-checked against `packages/assayer/src` by reading the named symbol, not by trusting a name match. A drift already held in the audited deferral register is cited by its label and not re-described, per the register's own rule; a drift not in the register is stated in full below and marked new. The register's scope is the health and metrics surfaces, so the new findings below concentrate where the register does not look: the derivation layer and the document's own machinery.

**Warning (The Part IV rewrite was never implemented)** · `warn:assayer:drift-landscape-unimplemented`

**New finding.** The July rewrite renamed and restructured the entire derivation layer; the code is still on the pre-rewrite vocabulary and the pre-rewrite object. Not one of the rewritten specification's Part IV type or function names occurs anywhere in `packages/assayer/src`: `DecisionLandscape`, `derive_landscape`, `ChallengePosterior`, `MomentMatched`, `DecisionFragility`, `ProfileAmbiguity`, `render_resonances`, `information_value` — zero occurrences each, and `fragility` and `domination_probability` likewise zero. What exists instead is the old text's surface: the derivation output is `Reckoning { tags, exploration }` (`src/assessment.rs:543`), produced by `derive_reckoning` (`src/resonance/derivation.rs:169`, re-exported at `src/lib.rs:142`), taking a `ChallengeEstimate` (`src/types.rs:177`), with `ResonanceProfile` kept as an alias of `Reckoning` (`src/assessment.rs:553`) and exploration computed as information value over the rendered tags (`src/resonance/exploration.rs:27`, `src/resonance/exploration.rs:52`). This is spec.old.md ``§29.2`` and ``§17.1`` exactly. The consequence for the campaign is structural, not cosmetic: for Part IV, `spec.old.md` — not `spec.md` — is the document that describes the shipped system, and the outline cannot treat Part IV of the rewritten text as a description of the code. Either the outline records Part IV as specifying unbuilt work, or the campaign obtains a ruling that the rename is pending implementation. It must not be left implicit.

**Observation (The rendering layer is not optional in the code)** · `obs:assayer:drift-rendering-not-optional`

**New finding, and a corollary of the above.** The rewritten rendering clause (`dec:landscape:rendering-optional`) makes rendering an optional layer over a presentation-free landscape, and the presentation-free invariant (`inv:landscape:presentation-free`) states that the primary output is a function of the risk basis, the channel policy and the challenge posterior alone. In the code there is no such separation: the single derivation entry point returns the tag profile with its display constants (`src/assessment.rs:543` through `src/assessment.rs:549`), the `ResonanceConfig` display constants are a required argument of the derivation call (`src/lib.rs:142`), and the module is named `resonance` (`src/resonance/`). The presentation-free invariant the specification states as unconditional has no counterpart in the implementation.

**Observation (Code cites the specification by superseded section number)** · `obs:assayer:drift-code-citations`

**New finding.** The Rust sources carry 494 citations of the specification by section number, over 144 distinct targets, spread across 100 files — heaviest in `src/resonance/derivation.rs` (48), `src/risk/challenge.rs` (29), `src/assessment.rs` (25) and `src/feature/dimension_map.rs` (25); an example is `src/resonance/channel.rs:127`. Resolved against both editions:

| Resolution | Citations | Distinct targets |
| --- | --- | --- |
| resolves in both editions, same section title | 373 | 104 |
| resolves in both editions, **different** section title | 39 | 10 |
| resolves only in the pre-rewrite edition | 54 | 18 |
| resolves in neither edition | 28 | 12 |

The middle row is the dangerous one, because it fails silently. A citation of ``§16.4`` (twelve of them) meant "Action Tag Locations" and now lands on "Example 4: 3-Action System"; ``§16.5`` (eight) meant "Monotonicity Enforcement and Existence Failure" and now lands on "Design Summary"; ``§17.1`` (three) meant "Information Value" and now lands on "Decision Fragility"; ``§15``, ``§16``, ``§16.2``, ``§16.3``, ``§29.2`` and ``§29.3`` move likewise. The 28 citations that resolve in neither edition — ``§4.6``, ``§6.9``, ``§17.5``, ``§25.4.5`` and eight targets in a ``§26.6``–``§26.15`` range that no edition has — point at a third, older numbering. The code also cites decision records by number 984 times (for instance `ADR-L-510 §7` at `src/assessment.rs:248`), against 748 such references in the docs tree and 2,701 in the record set. This is the volume the naming ruling (`dec:assayer:naming-schema`) has to move, and it is an argument for extending the legacy lint's carrier to Rust comments once the scanned- region recognition lands.

**Observation (The rendering configuration table does not exist)** · `obs:assayer:drift-rendering-config`

**New finding.** The specification cites a rendering configuration table twice (spec.md:4957, spec.md:5214) and Chapter 28 does not contain one; its thirteen core tables plus derivation and Companion configuration have no rendering row. The constants themselves appear only inside the Appendix A code block. The type is real in the implementation (`ResonanceConfig`, exported at `src/lib.rs:142`), so this is a hole in the specification's own configuration reference, not a deferral.

**Observation (The Companion replacement trait does not exist)** · `obs:assayer:drift-companion-trait`

**New finding.** The replacement surface (`req:companion:replacement-trait`) specifies a replacement trait for the Companion Tracker carrying a default `posterior()` method. The crate exposes three public traits — `AssessmentSharedState` (`src/assessment.rs:634`), `Clock` (`src/testing/clock.rs:38`) and a test-helper compatibility trait (`src/tests/helpers.rs:110`) — and none is a Companion replacement surface; no `posterior()` method exists anywhere. The Companion itself is implemented as specified in every other respect (`src/risk/challenge.rs`), so this is a single unbuilt extension point rather than a missing component.

**Register (Specification claims already held as deferrals)** · `reg:assayer:spec-drift-known`

These sections of the specification describe surfaces the deferral register already holds open, every retirement claim in it naming the live implementation locus that is its evidence (`sec:assayer:defer-retirements`). They are listed so the outline knows the deferral is the reason and does not re-derive it, and are cited rather than re-described, per that register.

Nothing remains listed here. Hibernation as an optional lifecycle state (`alg:registry:hibernation`) was the last of them and it now ships, its archive contract fixed at (`entry:construction:hibernation`).

The synchronisation error, immature-cell count, per-entity concordance, alarm-outcome accumulators, Ledger value realisation and end-to-end feedback latency formerly listed here are now shipped and their deferrals are retired at (`entry:assayer:defer-sync-error-visibility`), (`entry:assayer:defer-ledger-immature-count`), (`entry:assayer:defer-per-entity-concordance`), (`entry:assayer:defer-alarm-outcome-cusums`) and (`entry:assayer:defer-ledger-materiality`). The last of those left the register by a route the others did not take: its deferral was a definition rather than an engineering gap, and it closed on the definition it was waiting for, that the Ledger may store the raw rate and arrival evidence the materiality theorem's predicate is about.

The encoding-effectiveness indicator formerly listed here has left the register by a second route, and the distinction matters to an outline reading this list. What the specification now defines under that name is the shipped mean-slot-weight reading (`def:monitoring:encoding-effectiveness`), delivered and retired at (`entry:assayer:defer-sentinel-informativeness`); the three-component formula the deferral originally stood against is retired as superseded design history at (`entry:assayer:defer-encoding-effectiveness`), so no deferral holds that section open any longer.

**Data (What was checked and found sound)** · `data:assayer:spec-drift-clean`

Spot checks that agreed: the aggregate block width of 15 (`src/feature/aggregate.rs:48`, `src/feature/dimension_map.rs:56`); the reference configuration at 638 dimensions (`src/tests/dimension_map.rs:152`, `src/snapshot/published.rs:96`); the published-snapshot concurrency model (`def:publication:model-snapshot`), implemented as lock-free published state (`src/health/blend_stats.rs:110`, `src/api/builder.rs:91`); the risk basis (`schema:risk:basis`), whose fourteen fields match the specification's struct field for field (`src/assessment.rs:243` through `src/assessment.rs:292`); the per-Sentinel alarm summary (`def:runtime:alarm-summary`) (`src/extraction/alarm.rs:49`); the Ledger smoothing default (`src/config/types.rs:320`); and the label-guidance three-category output (`sec:guidance:core`) (`src/guidance/mod.rs:105`).

## Audit verdict · `sec:assayer:spec-audit-verdict`

**Summary (What this audit found)** · `summ:assayer:spec-audit-verdict`

The rewritten specification is structurally sound and substantively close to the implementation everywhere except Part IV, where the July rewrite renamed and reorganised a layer the code never followed. Its identity scheme is one revision old and already broken in seven places, its own index of that scheme is empty, and its numbering has three holes inside a single revision. Its 1,727 tag occurrences are the conversion burden; its 117 in-document locators are a rounding error beside them. It carries roughly 200 environments in prose clothing and no environment heads at all. And it is, today, invisible to the checker.

**Requirement (What the outline wave must carry forward)** · `req:assayer:spec-outline-inputs`

The labelled outline (`plan:assayer:specification-rewrite-contract`) is bound by eight findings of this audit.

1. Take the structure, not the numbering. The Part and chapter skeleton of (`tab:assayer:spec-inventory-chapters`) survives; every section number is discarded, and the three numbering holes of (`obs:assayer:spec-numbering-holes`) are the evidence that they must be.
2. Mint from the 456 existing tag identities, not from scratch. Each already names a concept an author judged citable; the outline's task is to assign each a registry kind, not to invent the inventory. The kind estimates of (`tab:assayer:spec-implicit-kinds`) are the starting distribution, and the restraint of (`cav:assayer:spec-implicit-overreach`) bounds it.
3. Resolve the seven dangling identities of (`reg:assayer:spec-dangling-tags`) deliberately: five want mint sites the rewritten text will supply, one wants a configuration table (`obs:assayer:drift-rendering-config`), and one is correct as it stands.
4. Decide Part IV against (`warn:assayer:drift-landscape-unimplemented`) before writing a line of it. The outline must record, per environment, whether it describes the shipped system or unbuilt work, and the outline review is where that record is checked against the code.
5. Carry the loss candidates of (`reg:assayer:spec-divergence-losses`) as open questions into the outline review — in particular the deleted interface-map section and the missing computation order, both of which read as accidents rather than edits.
6. Do not transcribe the restating chapters (`obs:assayer:spec-restatement-chapters`), and treat Appendix H as imported material rather than as Assayer-owned environments.
7. Declare binding force at every head. The specification today marks it nowhere (`obs:assayer:spec-no-modal-vocabulary`); the partition of (`tab:assayer:spec-partition`) is the audit's reading and the outline is where it becomes a decision.
8. Schedule the removal of the specification from the pre-calculus list (`obs:assayer:spec-nonparticipation`) inside the concatenation entry (`spec:spec:measurement-judgement-interpreter`), and configure the legacy lint to the shapes of (`req:assayer:spec-legacy-lint-shapes`).

**Caveat (What this audit did not do)** · `cav:assayer:spec-audit-limits`

The divergence pass is at chapter granularity: two texts of six thousand lines each were compared by structure, by distinctive string, and by reading the sections where the structure differed, not line by line, so a sentence-level loss inside a section that survives would not appear here. The drift check is a spot check across the surfaces the deferral register does not cover, weighted toward the derivation layer where the divergence was found; it establishes that Part IV is unimplemented and that eight named claims hold, and it makes no claim of exhaustiveness over Chapters 5 through 12. Both limits are inherited by the outline review (`plan:assayer:specification-rewrite-contract`), whose cross-reference against the code is the adversarial pass this audit is not.
