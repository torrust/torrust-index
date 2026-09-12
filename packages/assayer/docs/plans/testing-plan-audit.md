# The Assayer Testing-Plan Audit · `rep:assayer:test-plan-migration-basis`

This is the audit of `docs/plans/testing_plan.md` as found, the first wave of the campaign's last entry (`plan:assayer:test-debt-plan`), whose own rule is that the test-side tooling is specified from an audit of the plan rather than guessed. It counts what the plan contains, checks its claims against the test corpus, states what the linter must gain for the migrated plan to be checked rather than remembered, and fixes the shape the migration wave writes to. Nothing in the plan is edited here.

The label at each head is that environment's mint; a parenthesised label in running text is a citation. Legacy forms of the burning families are written in code font throughout, so that quoting them adds nothing to the registers of (`dec:assayer:burn-lists`). The linter this audit specifies against is a tool the repository carries rather than a part of this package, and its internals are named here by the role they play.

## Method and measurement basis · `sec:assayer:testing-audit-method`

**Convention (What the audit counts)** · `conv:assayer:testing-audit-counting`

A *reference form* is a shape by which the plan names something outside itself: a backtick span, a section mark and locator, a retired record number, a scenario number. The first is counted over backtick spans, the rest by the recognizers the burn census uses, so a count here and a register row there are the same number by construction. Liveness claims name a file and a line and were read, not inferred; the census is the authority, a name being live when the workspace defines a function of it and covered when its derived label stands at its standard place.

**Data (Measurement basis)** · `data:assayer:testing-audit-basis`

Measured 2026-08-19 at `32f64bdf`. Subject: `docs/plans/testing_plan.md`, 105 lines. Second subject, by necessity: `packages/assayer/tests/README.md`, 777 lines, the scenario matrix the plan reviews. The check reports 4,287 covered tests over 698 Rust sources — 2,270 `crate`, 1,355 `integration`, 662 `unit` — every one carrying its derived label with no missing, wrong, misplaced, or colliding asset, the repository clean at 1,483 mints and 2,566 resolved citations.

## The plan as found · `sec:assayer:testing-audit-inventory`

**Observation (The plan is a review, and its subject is elsewhere)** · `obs:assayer:testing-plan-is-a-review`

The document is not a plan in the sense the backlog's other subjects are: it is a review of the scenario matrix at `packages/assayer/tests/README.md`, judging that matrix's strengths, gaps, and priorities. Two consequences bind the migration. Its intents are recommendations *about* a catalogue rather than claims about the system, so recasting them as labelled entries means minting against the matrix's content, not the specification's. And the matrix carries the references in bulk — the plan names twelve tests, the matrix one hundred and sixty-two — so a migration reaching only the plan migrates the review and leaves the reviewed.

**Table (The plan's divisions and their intents)** · `tab:assayer:testing-plan-divisions`

| Division | Lines | Intent |
| --- | --- | --- |
| Overall Assessment | 3–9 | a verdict on the matrix, plus one dated progress note recording five newly landed witnesses |
| What's Strong | 13–29 | four commendations, each naming the tests or sections that earn it |
| Missing Scenarios | 35–47 | six proposed scenarios, lettered A–F, each with a rationale and a spec pointer |
| Gaps in Existing Scenarios | 49–55 | three section-level gap findings, lettered G–I |
| Scenario-Level Observations | 57–65 | four per-scenario critiques proposing reframings |
| Section-by-Section Priority Assessment | 69–84 | a twelve-row table of coverage counts, risk, and recommended priority |
| Structural and Process Recommendations | 88–100 | six numbered process recommendations, two of them already discharged |
| Summary | 104–105 | the verdict restated, with three next actions |

**Table (Every reference form the plan uses)** · `tab:assayer:testing-plan-forms`

| Form | Occurrences | Distinct | Notes |
| --- | --- | --- | --- |
| qualified test path, ``module::function`` | 9 | 9 | all in the progress note and gap findings |
| bare test function name | 3 | 3 | the source audits, all three in `src/tests/source_audit.rs` |
| source-file name | 3 | 3 | two Rust files and the matrix itself |
| crate name | 2 | 2 | the two packages the boundary claim concerns |
| API or type span | 7 | 7 | not references to the corpus; ordinary displayed code |
| section mark and locator | 19 | 9 | eleven into the matrix's own divisions, eight into the specification |
| retired record number | 6 | 5 | ``ADR-L-110``, ``ADR-L-160`` twice, ``ADR-L-170``, ``ADR-L-250``, ``ADR-L-350`` |
| scenario number, marked | 35 | 25 | plus two ranges and one bare list of seven |
| word-form division name | 11 | 3 | ``Section V``, ``Section VI``, ``Section XII`` |

The first five rows are the twenty-four backtick spans of the document, and the section-mark and record-number rows are exactly the register rows the burn lists already carry for this file — 19 and 6.

**Observation (The matrix carries the load, and no surface reaches it)** · `obs:assayer:testing-matrix-uncarried`

The matrix carries 262 linked backtick spans — 153 qualified ``module::function`` paths, 46 file names, 63 bare names — and a legacy load of 33 section marks, 3 retired record numbers, and 128 marked scenario numbers. Nothing in the linter reads it. Its carrier reads a package's `adr` and `docs` directories and no other; the matrix is under neither. Its burn lists declare the Assayer's prose as those same two directories and its `tests` tree only as a *code* surface, whose Markdown they do not read. And the census parses Rust alone. The document the plan exists to review participates in nothing, is censused by nothing, and is counted by nothing.

## Liveness · `sec:assayer:testing-audit-liveness`

**Register (The plan's named tests against the corpus)** · `reg:assayer:testing-plan-spotchecks`

All twelve names the plan gives are live, and every one of them is a covered asset carrying its derived label.

| Name as the plan writes it | Found at | Derived area |
| --- | --- | --- |
| ``assess_integration::assess_batch_matches_forward_and_reverse_singletons`` | `tests/assess_integration.rs:227` | `integration` |
| ``assess_integration::lifecycle_churn_around_cholesky_recompute_keeps_fast_path_valid`` | `tests/assess_integration.rs:466` | `integration` |
| ``learning_convergence::scalar_signal_population_split_converges_directionally`` | `tests/learning_convergence.rs:41` | `integration` |
| ``identity_layer::identity_observation_overflow_is_reported_and_assessments_stay_finite`` | `tests/identity_layer.rs:352` | `integration` |
| ``error::`` five label-channel and owner-shutdown witnesses | `tests/error.rs:292`–`447` | `integration` |
| ``core_model_modules_do_not_import_decision_layer`` | `src/tests/source_audit.rs:290` | `crate` |
| ``resonance_module_does_not_reference_core_model_state`` | `src/tests/source_audit.rs:250` | `crate` |
| ``core_and_api_paths_do_not_call_derivation`` | `src/tests/source_audit.rs:323` | `crate` |

The plan qualifies by the target's file stem, which coincides with the profile's area for the nine integration names and disagrees for the three source audits, written bare and of area `crate`. A migration that transliterated the plan's spelling would derive three wrong labels.

**Data (The matrix's cited names against the census)** · `data:assayer:testing-matrix-liveness`

Of the 172 distinct lowercase identifiers the matrix cites, 162 name a defined function and 10 are module stems written without their extension. Of the 162, exactly 160 are covered assets carrying a derived label — 54 `crate`, 103 `integration`, 3 `unit` — the two exceptions being the assertion helpers `assert_health_clean` and `assert_risk_basis_near`, which carry no test attribute and are correctly outside the census. Nothing either document names has been renamed away.

**Data (Classification of the plan's content)** · `data:assayer:testing-plan-classification`

The twelve-row priority table reproduces the matrix's tracker exactly, row for row, and the headline figures — 91 scenarios, 51.1% weighted — are the tracker's own. Live and carried forward: the six proposed scenarios A–F, the three section-level gaps G–I, the four scenario-level critiques, the twelve priority rows, and four of the six process recommendations — 29 intents. Superseded by landed work: the first two process recommendations, whose own text records the discharge, and the progress note recording it — 3 items, migrating as history rather than as intent. Stale: one count, the fifth process recommendation saying 24 scenarios are unaudited where the matrix and the plan's own summary both say 22. Nothing in the plan is dead.

## The tooling specification · `sec:assayer:testing-audit-tooling`

**Observation (No form in prose can cite a test today)** · `obs:assayer:testing-citation-gap`

Test labels live in code and enter no registry. The engine builds its registries from the mints it harvests out of carrier Markdown and resolves against those alone; the profile pass runs beside it and hands it nothing. Probed against the real tree, all three candidate forms fail: the derived label parenthesised in an Assayer document is an unresolved citation, the same label under the owner's own prefix is a self-qualified import, and a repository test label under the ``INDEX`` prefix is again an unresolved citation. The third is the informative one — the prefix resolved and the owner was found, so the failure is not the signature's but that no registry holds one of the 4,287 derived labels. The grammar is already sufficient, the calculus's own reader parsing a bracketed import that carries a test label; what is missing is resolution, not syntax.

**Proposal (Tool one: derived labels enter the registries)** · `proposal:assayer:testing-inventory-citation`

Seed each owner's minting registry with that owner's covered assets before pass two, and let the ordinary same-owner citation resolve against them. The syntax is then the existing citation form with a derived label inside it, and a repository document reaches an Assayer test through the existing import form with the ``ASSAYER`` prefix, for free and by the same rule.

The change is wiring rather than a new pass. The check already covers the census before it analyses the carrier, so the assets are in hand; `analyze` takes them as a third argument, and the engine adds one graph node per asset, keyed by the owner its package names, before harvesting. Three properties hold without further work. Order independence survives, the seeding completing before pass one rather than during pass two. Nothing can be minted by hand into that space, a bare occurrence of a reserved kind being already a hard failure. And a citation naming a renamed or deleted test already fails as an unresolved citation — which is the point: the migrated plan stops remembering its corpus and is held to it. No new finding code is required; one is merely worth having, a citation of a reserved kind resolving nowhere suggesting the nearest covered name.

**Gate (Acceptance for tool one)** · `gate:assayer:testing-inventory-citation-accepted`

A document under `packages/assayer/docs` citing the derived label of a live Assayer test passes, and the citation is counted in `citations_resolved`. The same citation naming a test that does not exist fails, naming label and location. A repository document citing that test through the ``ASSAYER`` prefix passes. A bare occurrence of a test label in prose still fails as unwarranted. The repository check stays clean at its existing mint and head counts, the profile block is unchanged, and two orders of traversal give the same report.

**Proposal (Tool two: coverage as a report, and no suite profile)** · `proposal:assayer:testing-coverage-report`

Suite-shaped *labelling* is declined on two grounds. The kind ``suite`` stands in the assets convention of the kind registry and is therefore reserved for derivation, so it cannot be authored by hand; and no profile could derive it, the cargo harness exposing no suite asset — an integration target is a file, and the matrix's divisions are prose with no code counterpart at all.

What the plan's content wants is arithmetic, and tool one supplies its substrate. Once derived labels are nodes, every citation of a test from a scenario environment is an edge, and the priority table's questions become graph questions: how many tests each scenario cites, which cite none, which divisions are thin. Extend the graph report with a coverage view over derived nodes, and leave it a report with no verdict, as the campaign's report-and- gate split requires. One caution: the seeded nodes must be excluded from the uncited listing by default, or 4,287 uncited tests bury it.

**Gate (Acceptance for tool two)** · `gate:assayer:testing-coverage-report-accepted`

The report names, per environment that cites at least one derived label, how many it cites; and it names the environments of a nominated document that cite none. It exits 0 whatever it finds. The existing graph report's listings are unchanged in the absence of the new view, and its uncited listing does not grow by the size of the census.

**Proposal (Tool three: the matrix joins the carrier)** · `proposal:assayer:testing-matrix-carried`

The matrix cannot be migrated while the linter cannot see it. Two routes exist and the choice between them is open: relocate it under `packages/assayer/docs`, leaving a pointer where the test tree expects one, or extend the carrier's package directories to admit a readme under a package's `tests` tree. Relocation moves one file and changes no adoption datum; extension changes a datum and reaches every package at once. Either way the same commit must decide the matrix's burn standing, since admitting it makes 33 section marks and 3 record numbers newly visible to registers designed only to shrink.

**Gate (Acceptance for tool three)** · `gate:assayer:testing-matrix-carried-accepted`

The check reads the matrix, reports it among its scanned sources, and either holds it to the head discipline or records it as not yet reached — never silently neither. The burn registers name it with the counts measured above in the same commit that admits it, and the check is clean immediately after.

**Proposal (Tool four: two burn families the registers lack)** · `proposal:assayer:testing-burn-families`

Two legacy families here have no register. The first is the word-form division name — ``Section V`` and its kin, 11 occurrences in the plan — which the section-mark recognizer deliberately does not read, as its own convention states. The second is the scenario number, ``#90`` and its kin: 35 marked occurrences in the plan and 128 in the matrix, an identity scheme the naming ruling (`dec:assayer:naming-schema`) retires with every other numbering. Neither can burn down before it is counted. The existing rows fall as ordinary migration work.

**Gate (Acceptance for tool four)** · `gate:assayer:testing-burn-families-accepted`

Each new family is declared with its recognizer, its surfaces, and its register, and the register is generated rather than written. A new occurrence of either family anywhere in the declared surfaces fails the check. A vanished occurrence fails until its row is regenerated. The existing four registers are unchanged by the addition.

## The shape of the migrated plan · `sec:assayer:testing-audit-shape`

**Proposal (What the migration wave writes)** · `proposal:assayer:testing-migration-shape`

The plan becomes a labelled document of area `assayer` under the record naming ruling (`dec:assayer:record-naming-schema-text`). Its 29 live intents become environments: the six proposed scenarios and the three section-level gaps become entries, each naming the work and the witness it wants; the four scenario-level critiques become observations carrying the reframing they argue for; the twelve priority rows become one register keyed by the matrix's divisions, once those divisions mint. The recommendations that landed become history, recorded in one summary and cited rather than restated.

Every test reference becomes a citation of the derived label in the profile's spelling rather than the plan's: nine integration names carry over mechanically, the three source audits take area `crate`. Every section mark into the specification becomes a citation of the spec label at that division; every section mark and word-form name into the matrix becomes a citation of the matrix's own mint, which is why the matrix must be reached first. Every scenario number becomes a citation of the scenario's mint. What retires: the ``ADR-L-NNN`` namings, replaced by the informal reference plus a citation of the landed record's identity head; the scenario numbering, whose identity moves into the labels; and the stale count.

**Requirement (What the migration wave inherits from this audit)** · `req:assayer:testing-migration-inputs`

Four things bind. The migration cannot begin before tool one lands, its whole product being citations that would otherwise fail. It must reach the matrix, not only the plan: the references live there, and every section-mark and scenario-number pointer needs a mint in the matrix to land on. It must derive labels from the census rather than transliterate the plan's spelling, on the evidence of the three source audits. And it must land its burn consequences in its own commits — the plan's 19 section marks and 6 record numbers to zero, the two new families counted before burned.

## Audit verdict · `sec:assayer:testing-audit-verdict`

**Summary (What this audit found)** · `summ:assayer:testing-audit-verdict`

The plan is live: 12 of 12 named tests exist and are covered, its priority table matches its subject row for row, one count has drifted. It is a review, and its subject — the 777-line matrix carrying 162 live test names — is invisible to the carrier, the burn census, and the profile alike. The tooling needed is smaller than the entry feared and differently shaped: one wiring change makes 4,287 derived labels citable from prose under the grammar the calculus already has, a report view answers the coverage questions the plan's content asks, a carrier decision admits the matrix, and two burn families are declared before they can fall. No new label kind is needed, and no profile beyond the two that exist.

**Question (What the matrix migration must settle)** · `q:assayer:testing-audit-questions`

Three. Does the matrix move under `docs`, or does the carrier grow to reach it where it lives? Does the migrated plan stay one document reviewing another, or do its live intents move into the matrix and the plan retire as history once emptied — the projection principle (`dec:assayer:projection-principle`) would ask that of any other document. And is there a concordance from the retired record numbers to the landed records, or does the migration derive one: the master register is keyed by identity head and carries no legacy number, so the five the plan names must be matched by judgment.

**Caveat (What this audit did not do)** · `cav:assayer:testing-audit-limits`

It did not audit the matrix's 91 scenarios against the code: liveness here is of names, not of claims, and whether a scenario's witnesses prove what it claims is the migration's judgment or a later conformance question. It did not run the test suite, and it did not decide the matrix's own migration shape, which needs the answer to the first question above.
