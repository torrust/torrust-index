# Clock · `rec:clock:decay-time-governance`

This record decides what time means to the system: two domains kept apart by type, one clamp that no caller can step around, and one funnel every decay flows through. It realises the temporal choice (`dec:operational:temporal-governance`), which named two expected owners and was given one of its own instead, on the reasoning at (`dec:assayer:record-clock-record`).

It is a small record with a wide reach. Three decisions and six environments that follow from them are the whole of it, and four other records cite it rather than restating the discipline it fixes — which is what a statement with two owners would otherwise become.

**Decision (Two timestamp domains, separated by type)** · `dec:clock:two-domains`

The system carries two timestamp domains. One is intra-process and monotonic; one is persistent, survives a restart, and is the only domain anything writes to storage. They are distinct types, so a value from one cannot be passed where the other is required (`src/types.rs`).

The separation is structural rather than conventional, and the code holds it that way: no conversion exists in either direction between the two domains, the persistent type's only conversions are to and from wall-clock system time, and the monotonic type appears nowhere in the persistence paths. A convention would have made every call site a place to check; a type makes the mistake unwritable.

**Decision (The clamp is embedded in the timestamp interface)** · `dec:clock:embedded-clamp`

Elapsed time is bounded, and the bound lives inside the interface that reports elapsed time rather than at the sites that consume it. Neither domain exposes an unclamped reading, so a caller cannot obtain one without leaving the timestamp interface altogether (`src/types.rs`).

Both accessors carry it, against one shared constant. The persistent accessor clamps twice — a backward clock jump reads as no elapsed time, and the ceiling applies on the way out — and the intra-process accessor applies the same ceiling from the same constant. That the discipline covers both domains rather than the persisted one alone is the stronger property, and it is the one this record states.

**Decision (All decay flows through shared functions)** · `dec:clock:shared-functions`

Decay is computed in one place. Three functions carry all of it: a mouth that performs the arithmetic, and two thin domain-specific entries — one taking two persistent timestamps, one an elapsed duration — each clamping through its domain's accessor and then delegating (`src/numerics.rs`).

The funnel is what makes the type discipline pay. Both domains converge on one arithmetic, and the mouth re-asserts the clamp invariant as a debug assertion, so the property the types already guarantee is checked once more where it is consumed. The alternative is not a second implementation but many: a decay is three characters of arithmetic, and the sites that want one are spread across seven records' worth of code.

**Rule (Direct exponentiation is prohibited outside the shared functions)** · `rule:clock:no-direct-exponentiation`

No production site outside the funnel may raise a decay base to an elapsed power directly. The single exponentiation in the package's production sources is the funnel's last line, and every other decay is a call into it.

The prohibition is mechanised twice rather than asserted once: a build lint (`ci/lint_assayer.sh`) and an in-tree source audit that runs with the test suite (`src/tests/owner.rs`). The test tree is deliberately exempt and uses the operator directly as an independent oracle, recomputing an expected factor from the stated formula rather than through the function under test. A rule checked only by the thing it governs would prove nothing.

**Convention (Decay is applied lazily, at the point of use)** · `conv:clock:lazy-application`

Decay is not swept. It is applied at the point of use, over the mechanisms the specification distinguishes (`def:temporal:two-mechanisms`) and in the manner it fixes (`alg:temporal:lazy-application`), against the inventory of component rates it tabulates (`tab:temporal:decay-inventory`).

This record states the discipline and does not reproduce the inventory. Which quantity decays at which rate defines the system rather than deciding about it, and the specification owns it; what is decided here is that the application is lazy and that every instance of it goes through the funnel. Where the label path concentrates that application into one point is a separate decision and is owned elsewhere (`dec:ordering:decay-at-head`).

**Corollary (The prohibition is what makes the clamp unbypassable)** · `cor:clock:clamp-unbypassable`

The clamp of (`dec:clock:embedded-clamp`) bounds what the interface reports. It does not, by itself, bound what a caller computes: a site that reads a clamped interval and then exponentiates directly has left the interface and can produce any factor it likes from a legitimate reading.

The rule closes that gap, which is why the two must be read together. The clamp makes the unbounded interval unobtainable; the prohibition makes it unreconstructible. Either alone is a weaker guarantee than the pair, and the pair is what lets other records cite a bounded decay factor as a premise rather than re-establishing it at each site.

**Discussion (The rejected single timestamp type)** · `disc:clock:single-type-alternative`

The alternative was one timestamp type with a convention about which values are safe to persist. It is a smaller vocabulary and it removes the conversions the separation makes impossible, at the cost of moving the distinction from the type system into the reader's attention.

It fails on what it makes possible rather than on what it costs. A monotonic reading persisted and restored is meaningless in a way nothing detects: it does not fail, it produces a plausible number against a clock that no longer exists. The separation exists so that this cannot be written, rather than so that it is caught in review.

**Remark (What durability inherits from this record)** · `rem:clock:persistence-inheritance`

The persistence record owns which domain a checkpoint stores and what a restore recomputes; it does not own the measuring. A restore reads the elapsed interval once through the clamped persistent accessor and then applies the resulting factor once per model family, and the two halves meet at that one line.

The seam is worth naming because it is where a fold would have been tempting. That the interval is clamped and measured through the funnel is this record's; that it is applied exactly once is durability's own decision (`dec:durability:decay-once`). They are adjacent, not identical, and separating them is what keeps a rule binding on seven records out of the record about what survives a restart.

**Remark (The standardisation exception, and where it is owned)** · `rem:clock:standardisation-exception`

Standardisation statistics carry no time-indexed decay at all. They are the one body of state this record's discipline does not reach, and the exception is deliberate: a statistic whose whole job is to describe the distribution a feature is scaled against would be made unstable by decaying it.

The choice is not this record's. It belongs to the record that owns how the statistics are acquired and updated, which states it as (`dec:vector:no-time-decay`) and makes the same choice the specification makes for the class-rate trackers and for the same reason (`dec:weighting:no-time-decay`). It is named here so that a reader who has just been told all decay flows through one funnel learns immediately what does not decay.
