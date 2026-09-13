# Conventions · `rec:conventions:cited-forms`

This record states the documentation, label and layer conventions the package's own documents cite: which occurrences participate, how a citation crosses an owner boundary and when it may, what a claim's area is and what the register of areas has to say, what became of the test monolith, and the discipline a lint exception is held to. Each stands at the length a citing sentence needs and no further, because a restatement longer than that is a second authority rather than a reading of one.

The doctrine these descend from is the repository's and is named rather than reproduced. ADR-T-014, the label calculus, fixes the occurrence grammar and the resolution rules; ADR-T-017, the test-documentation policy, fixes the claim, its area, and the register the areas are curated into; ADR-T-019, the layer owner graph, fixes reach and what an import may cross.

**Judgment (Participation)** · `judg:labels:participation`

An occurrence participates, or it is text, and nothing about a label is decided before that is.

In prose, occurrences in authored text participate; fenced blocks and double-backtick spans do not. A token shown but not meant is placed in one of those, which is what lets a document name the forms it is retiring without joining the debt it measures. A table cell is authored prose and its occurrences participate exactly as running text's do, so two bare occurrences of one label are two mints even when one of them stands in a cell.

In code, only comments and documentation comments are scanned; string literals, character literals, and fenced examples inside doc comments are not. One comment is one logical region with its leaders resolved away, and delimiter pairing is settled within a region before any span in it is parsed. The defining source mints; indexes and catalogues cite.

**Inference rule (Imported citation)** · `inf:labels:imported-citation`

A citation written bare resolves within its own owner and never outside it. A citation that means to cross owners is written in the imported form — the label qualified by the cited owner's registered prefix — and it resolves to that owner's unique mint of the label.

The qualification is the declaration of the crossing and nothing more. The authority of the imported fact is a property of the owner that mints it, never of the bracket that names the owner. The prefix must be registered, and a self-qualified import is underivable, because an owner naming itself has crossed nothing and a form that admitted it would make the crossing undetectable in exactly the case where it is not one. A local paraphrase is not a substitute for the imported citation: a paraphrase resolves to nothing and dangles at nothing when the fact it paraphrases moves.

**Rule (Every corpus reaches the root's repo-wide policy)** · `rule:layers:root-is-reachable`

Every corpus reaches the root corpus for the purpose of the citation law, whether or not its manifest declares a dependency on the root crate. The root carries the repository's repo-wide policy — its records, its conventions, its kind registry — and policy that binds every member must be citable by every member. This is the upward direction, and it is always open.

The rule rests on a fact rather than a preference. No member declares a manifest dependency on the root crate, the manifest edges involving the root all pointing outward; so without the rule every upward citation in the workspace would be a violation, and repo-wide policy would be unreadable from the repository it governs.

**Rule (An import must be reached)** · `rule:layers:import-must-be-reached`

A participating imported citation naming a prefix is admitted only if the citing corpus reaches the owner of that prefix. An import of an unreached owner is wrong whatever else is true of it: it may resolve perfectly and its target may be a fine head, and the citing corpus has still claimed a dependency its declared reach does not carry. Resolution and admissibility are separate questions, and this is the second one.

The judgment reads the prefix and asks one question of the reach graph. It never inspects the target's kind, its area, or the document it mints in, because those are the cited corpus's business, and a rule that consulted them would be legislating another owner's naming from outside it.

**Convention (Purpose areas)** · `conv:testdocs:claim-areas`

A claim's area is thematic: it names what the statement is about, not where the test lives. Structural home is already carried by the derived label's area, and a second copy of it would say nothing.

The vocabulary starts free. An author mints in whatever area names the subject, the checker censuses the areas it finds and reports them, and the vocabulary is curated into a register as it stabilises (`req:testdocs:area-register`). That order is the one thing about this arrangement that would have been wrong the other way round: an area set fixed before the sentences exist is a taxonomy that has met no sentence, and the set nearest to hand when the sentences are missing is the set of division names the sentences were about to replace.

**Requirement (The area register)** · `req:testdocs:area-register`

An owner minting claims carries one area register in its own prose: an entry per area, whose head prose states the stake — what is lost if claims of this area fail. The stake sentence is the register's whole reason to exist, being the one thing about an area that no census can compute and no claim states.

A register entry is prose and is authored, and nothing here makes the area set adoption data: an unregistered area is a report line rather than a finding. The register's home is the owner's front page, because the stakes of an owner's areas belong where a reader meets the owner rather than in a side file.

## The monolith · `sec:testdocs:monolith`

The package's test tree once carried a scenario matrix: a long catalogue of numbered promises sorted into named divisions, with a hand-kept tracker at its top saying how many of them were witnessed. It was the largest test document in the workspace and the one the test-documentation policy most changes. It does not survive as a document, and every function it performed does.

**Table (Where the matrix's work went)** · `tab:testdocs:monolith-homes`

| What the matrix held | Where it went |
| --- | --- |
| the scenario-to-test index | the generated per-folder matrices, recomputed rather than maintained |
| the scenarios naming no test | authored claim mints in readme prose, where a claim no test cites is an intent and its uncoveredness is a report line rather than a tracker column |
| the division preambles, and the vocabulary they imply | the area register, each area's stake sentence its own (`req:testdocs:area-register`) |
| the strategy column, duplicated against the Rust copy and drifted from it | the tests' own gloss prose, which is the single home that ends the drift |
| the mechanism index | a generated reverse lookup over the claim graph, which is a query and not a document |
| the harness roadmap | the testing plan, which absorbs it and stays alive |

The hand-written coverage tracker is in none of the six because it belongs in none of them. It is derivable cell for cell from the scenarios' own status strings, which makes it an undeclared generated region rather than content, and it is replaced by the coverage report — which is also what proves that nothing was dropped when the matrix retired.

## Lint exceptions · `sec:assayer:clippy-exception-discipline`

A lint exception is taken individually and judiciously, at the item that needs it, carrying a written justification for why that item needs it.

The same discipline covers test and integration code: test material answers to the lint set and the checks that library code answers to, and a test area earns no allowance the rest of the tree would be refused. What this rules out is the module-wide allowance standing at the head of a shared test-support module, because it silences the lint for every item beneath it, so that each later decay under it — a helper left without a caller when the code that called it retires, an import outliving its use — arrives as silence instead of as a finding. A test area exists to notice things, and a blanket allowance is a standing instruction not to.

Where a shared helper module is included by several test binaries and an item is genuinely alive in one of them, the repair is to partition the module so that each binary carries what it uses. Where a single item truly needs the exception, it takes the item-level allowance with its written justification, on the same terms as anywhere else.
