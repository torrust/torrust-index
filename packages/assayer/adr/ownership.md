# Ownership · `rec:ownership:feed-forward-custody`

This record fixes custody. It says what the package holds outright, what it only ever sees as a value already handed to it, and what makes the difference a fact about the program rather than a habit of its authors. It is the smallest of the structural records and the one the others lean on hardest, because the direction information travels is a premise every downstream record assumes rather than argues.

The choices realised here are the boundary direction (`dec:structural:boundary-direction`), its mechanical enforcement (`dec:structural:boundary-enforcement`), and the coordinate width (`dec:structural:coordinate-width`). The specification fixes the concepts and the guarantees; this record fixes what was decided about them and cites rather than restates.

**Decision (Information crosses the boundary in one direction)** · `dec:ownership:feed-forward`

A report crosses from the Sentinel corpus as an owned value. The package holds no handle to any Sentinel, stores no callback, and calls nothing back: once a report is given, the giver is out of the picture, and nothing inside the package can reach for more. What may be consumed is enumerated (`tab:boundary:consumed`), and what may not is enumerated beside it (`tab:boundary:not-consumed`); the two tables together are the boundary, and this record adds no third list of its own.

This is the specification's feed-forward invariant (`inv:guarantee:feed-forward`) realised as a shape rather than as a rule about timing. A design in which the package could call outward would satisfy the same invariant only while nobody called; this one satisfies it while nobody can.

**Decision (The package owns its identity graphs outright)** · `dec:ownership:graph-custody`

The identity graphs are the package's own. No Sentinel holds one, shares one, or is consulted about one, so the only boundary they have is the external one the feed-forward direction already fixes. Custody is what makes that true: a graph the package merely borrowed would need a protocol for who may mutate it and when, and that protocol would be a second boundary running inward through the middle of the package.

The graphs live behind a dedicated owner and the assessment path never mutates one (`src/identity/maintenance_loop.rs`). The accumulation record owns how that state is keyed and reached; this record owns only the fact that it is ours.

**Decision (Coordinates are carried at one fixed internal width)** · `dec:ownership:coordinate-width`

Every coordinate is carried internally at a single fixed width, and the package is not generic over the coordinate type. The width is the specification's (`dec:encoding:fixed-internal-width`); what is decided here is that no type parameter threads it. One key type reaches every structure keyed by position — the identity graphs, the outcome store, the extraction routing — and the code carries the width as a concrete integer type throughout (`src/identity/competitive.rs`, `src/identity/mod.rs`).

The alternative is a coordinate type parameter, discussed below (`disc:ownership:generic-alternative`). It was rejected on what it costs the structures that would have to carry it and never vary it.

**Decision (The boundary is enforced mechanically, not by review)** · `dec:ownership:enforcement`

The import surface is restricted by an allowlist checked as a test. A module that may not reach the derivation vocabulary cannot import it, and the check fails the build rather than the reader. The specification states what the check must hold (`tab:boundary:verification`); this record decides that a check holds it at all, which is a different question from which way information travels and is answered here rather than inherited.

The reasoning is the campaign's standing rule that a property worth having is worth a tool (`goal:assayer:tools-over-discipline`). A boundary maintained by review degrades silently, because nothing reports the review that did not happen.

**Rule (The import allowlist)** · `rule:ownership:import-allowlist`

The rule as it stands. Every `use` of a crate-internal root from inside the derivation module — which is the figure the first check is drawn around (`dec:derivation:allowlisted-imports`) — must name a root on an explicit allowlist, and the same module may not reach the forbidden roots by fully qualified path either, the second check existing because the first is defeated by writing the path inline. The converse direction is checked too: the core-model roots may not import the derivation layer's policy types, and the core, API and guidance paths may not call the derivation function implicitly. All four run as ordinary tests over the source tree (`src/tests/source_audit.rs`).

A failure reads as a named file and line with the offending root and the allowlist beside it, which is the diagnostic a reader can act on without opening this record.

**Corollary (The guarantee is discharged by what exists)** · `cor:ownership:structural-discharge`

Because no handle exists, the feed-forward guarantee holds at every instant rather than at every call site, and no path needs a check at reception to maintain it. This is why the surrounding records may assume the direction without restating it, and why the deferral of a reception-time check would not be a gap in the guarantee: there is nothing for such a check to catch that the absence of a handle has not already prevented.

The allowlist of (`rule:ownership:import-allowlist`) is therefore not what makes the direction true. It guards a second and narrower thing — that the derivation layer stays out of core state — and the two should not be conflated.

**Remark (Why declared widths lift into the one internal width)** · `rem:ownership:width-lift`

A host declares its own coordinate domains, and they are not all the internal width. They lift into it (`alg:encoding:coordinate-lift`), which is what lets the single key type stand. The lift belongs to the host's integration layer rather than to any Core surface, so the width the host declares reaches the package only as a range check on the identity path; the package sees lifted values and nothing else. That division is why this record can decide a single internal width without also deciding anything about what a host may declare.

**Discussion (The rejected handle)** · `disc:ownership:handle-alternative`

The alternative was to hold a handle or a trait object the package could call when it wanted more than a report carried. It is the obvious design and it was rejected, because it converts the boundary from a fact about what exists into a convention about when to call. Under it, the feed-forward invariant becomes a property of every call site rather than of the type graph, and it stays true exactly as long as every future author remembers it. Nothing would report the first violation, and the corollary above would have no premise.

**Discussion (The rejected coordinate type parameter)** · `disc:ownership:generic-alternative`

The alternative was a coordinate type parameter threaded through every structure keyed by position. It buys the ability to vary a width that nothing varies. The cost is paid by the identity layer, the outcome store and the extraction routing alike: each becomes generic over a parameter it instantiates one way, and every signature that touches a key grows a parameter that carries no information. The width is a property of the encoding the specification fixes, not a degree of freedom the package was asked to preserve.

**Caveat (What the guard does not cover)** · `cav:ownership:lint-scope`

The guard is a build-time source scan, and it is textual. It reads import lines and token occurrences, not the type graph, so a reference constructed by a macro or reached through a re-export is outside what it can see. It runs in the test suite rather than in the compiler, which means it catches a regression at test time and not at the moment it is written. And its subject is the derivation boundary; the Sentinel-handle direction that gives this record its name is discharged by construction and is checked by no test at all, on the reasoning at (`cor:ownership:structural-discharge`).

None of this is narrated away. The gap between a guarantee held by construction and a guarantee held by a textual scan is real, and a reader is owed the distinction rather than a single word covering both.
