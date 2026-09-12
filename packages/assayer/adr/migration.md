# Migration · `rec:migration:burn-disciplines`

This record states the burn disciplines the package's campaign report and its plans rely on: what a retirement decides, what a register may count, over which trees it counts, how a surface is read, and what a table has to spell before a checker reads it as a tracking declaration. The campaign these governed is finished. The record survives it because the documents that describe the campaign go on citing the disciplines by name, and a description of a retirement that cannot say what a retirement is has stopped being a description.

ADR-T-020, the migration disciplines, is the doctrine these statements descend from, and the four retirements named below are the repository's rather than this package's. What stands here is the part a reader of this package needs in order to follow its own documents.

**Decision (The superseded reference forms)** · `dec:migration:superseded-forms`

The label calculus supersedes four families of reference, and one migration lint reads a document for all four: section-number identities spelled as a mark and an ordinal, records named by a retired series letter and a number, record numbers carrying no series letter at all, and the two-level tag schemes a corpus builds when it wants what the calculus provides. Each is replaced the same way — cite the one existing label whose gloss covers what the sentence meant — and where the old reference pointed at a local ordinal heading and nothing states the gloss, the mark and the ordinal go and the descriptive heading stays, because the reference had no referent and manufacturing a label to cite would be a mint and a category error.

That the count is four is written down rather than left to the recognizers, and this is the load-bearing half of the decision. A fifth family is a fifth retirement decided on the same terms, never a widening of a rule already taken: a recognizer stretched to reach a spelling nobody retired would be legislating a family into existence without deciding one, and the corpora that still keep the spelling would find their own live convention counted as somebody else's debt.

**Requirement (The register is a ratchet)** · `req:migration:burn-ratchet`

A retiring family's surviving occurrences are enumerated per file, and the enumeration may only shrink. A file carrying more occurrences than its row allows fails, naming the occurrence that broke it; a row outliving the occurrences it counted fails too, until it leaves in the commit that emptied it.

The enumeration is per file rather than per total, and that is the whole of its usefulness. A total lets one file lose a reference while another gains one and calls the trade clean, which is exactly the motion a migration must not make. Per file, a commit that migrates one document cannot pay for a regression in another.

The census is computed from the corpus and never edited by hand. Where a register stands as a document, its census region is generated and the preamble around it is authored; where the authority stands in declared data, the document is a view of that data. Every term above survives the move unchanged, an empty enumeration remaining an applicable clean verdict rather than an absence.

**Convention (A family is bounded and literal)** · `conv:migration:burn-family`

A burn family is a syntactic shape, bounded so that zero is reachable. An unbounded rule counts every token of the shape in the corpus — an issue reference and an ordinal in a list among them — and a register counting those could never reach zero, because reaching zero would mean rewriting prose that was never about the retired thing. The bound is therefore load-bearing rather than fastidious, and the register that names the family is where the bound is recorded.

Two bounding devices recur. A family may be bounded by *range*, where the retired document numbered a known interval and the recognizer reads only that interval, the boundaries following the mark's own shape: a mark must open a token, so a mark inside a word belongs to that word and a doubled mark opens nothing, and a longer number is never read as a shorter one carrying a tail. A family may instead be bounded by *enumeration*, where the members are carried verbatim and entire rather than as a pattern — the case when the names were written as sentences rather than as nouns, and no rule shorter than the sentence tells one from ordinary prose on the same subject. Matching entire is what makes such a family safe to count: a literal occurrence is a reference and not a coincidence, which a two-word token drawn from the same sentence would not have been.

**Convention (One recognizer, census and lint alike)** · `conv:migration:burn-one-recognizer`

Where a family is also a rule of the migration lint (`dec:migration:superseded-forms`), its census is taken by that same recognizer rather than by one of its own. A register counting one thing while a gate judged another is a ratchet that cannot hold, and the two would drift the first time either was amended. A family with no corresponding lint rule declares its own recognizer beside its register, which is the ordinary case for a family that is one document's private identity scheme.

Closing a burn therefore takes three things together and no one of them alone: zero occurrences in the declared surfaces, the emptied enumeration retained as a regrowth gate, and the ordinary check clean.

**Convention (Each list declares its surfaces)** · `conv:migration:burn-surfaces`

A list declares the trees it is counted over, and the declaration follows the family rather than a repository-wide default, because the reach is a judgment about what the family *is*. Three shapes of judgment recur. A family that is one corpus's migration debt is counted over that corpus. A family whose retiring document stood outside the documentation tree is counted over the trees that document stood in as well, since a surface stopping at the documentation tree would leave uncounted exactly the references that matter most. And a family that is no single corpus's debt is counted over the workspace.

A census reaching a corpus that never retired the scheme is counting the wrong thing, and it could not reach zero without rewriting documents that never joined the migration. Where a file stands outside every surface a list declares, that is a judgment to make rather than an omission to tolerate: the family's reach is the list's to name, and a file left out by accident and a file left out on purpose are indistinguishable in the census.

**Convention (How a surface is read)** · `conv:migration:burn-surface-reading`

Two kinds of surface, and each is read the way it is written.

Prose is read as its format. A form standing in code font or inside a fenced block is displayed rather than referenced and is not counted, which is the boundary participation already draws for occurrences and is drawn here for the same reason (`judg:labels:participation`): a register's own preamble must be able to name what it counts without entering the census it describes.

Code is read for its comments alone. A form in a string literal is data the program carries rather than a reference the corpus makes, and a form in an identifier is a name that only renaming could retire; a register counting either could reach zero only by rewriting code that was never about the retired thing. One comment is one logical region with its leaders resolved away, so a reference whose mark stands at the end of one comment line and whose locator stands on the next is one reference and not two — a reader that took each line alone would count one reference twice and offer two replacements where one is owed.

**Convention (The tracking-table recognition contract)** · `conv:migration:tracking-columns`

A table is read as a *tracking declaration* exactly when its header names a Head column and a Document column: the head cell carries the label the tracked document's head must mint, and the document cell carries that document. A header naming an Entry column as well adds the outline entry the row answers to, beside the relation rather than in place of it. The spelling is a recognition contract rather than a caption, so a table naming its columns otherwise is not a tracking table with a defect; it is not one.

Making recognition turn on the header is what lets a corpus keep outline tables that are deliberately not tracking declarations. Were an outline's per-record tables read as declarations, every row would verify a claimed head against a document that does not yet exist, and every row would fail — correctly and uselessly. Such a table names its columns otherwise and keeps its labels in double-backtick spans so that no cell mints (`judg:labels:participation`), and it converts to a tracking table in the commit that lands the document it tracked.
