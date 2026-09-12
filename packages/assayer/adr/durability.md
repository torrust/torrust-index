# Durability · `rec:durability:checkpoint-and-replay`

This record fixes what survives a restart and what is deliberately rebuilt. It realises the durability choice (`dec:structural:durability`), and it owns the one ordering the record census found decided both ways: whether a submission is sanitised before it is journalled or after.

The specification states the guarantees a restore must preserve; this record decides the arrangement that preserves them, and cites rather than restates.

**Decision (A periodic checkpoint and a self-contained journal)** · `dec:durability:checkpoint-journal`

What survives a restart is two things: a periodic whole-state checkpoint, and a journal of the labels in flight since it. The journal's entries are self-contained — each carries enough to replay without the state that produced it — which is what lets replay proceed forward from the checkpoint without reconstructing any intermediate state to feed it.

Self-containment is the load-bearing half. A journal of deltas against evolving state would be smaller and would make every entry's meaning depend on every earlier entry's replay having been exact; the entry that is a whole submission depends on nothing but itself.

A checkpoint carrying a precision matrix that is not positive definite is refused at restore, and the refusal reaches the host as a construction failure. The checkpoint stores the precision matrix so that the restore does not have to recompute an inverse to rebuild the model, and that saving is exactly why nothing was reading it: the storage boundary's only numerical verdict is symmetry, and a matrix symmetric to the bit can carry a whole negative eigenvalue. So the restore factors what it was handed, once per model, and refuses what does not factor (`dec:posterior:cascade-never-fails`). A host holding that refusal has three real actions — an earlier checkpoint, a rebuild from the journal, or a clean start — and the refusal exists so that it takes one of them knowingly (`test:crate:checkpoint-restore-refuses-the-indefinite-witness`), (`test:crate:checkpoint-restore-admits-a-healthy-precision-matrix`).

The refusal is not a cold start, and the distinction is the point of it. An absent, unreadable or structurally incompatible checkpoint yields a cold start and always has: the first two are a deployment with no state to resume, and the third is a legitimate change to the schema or the templates that this record's restore is explicitly not asked to migrate across. None of those costs the host anything it had. A checkpoint that was read, whose structure agreed, and whose numbers the model may not hold is none of those three. Cold-starting on it would discard every learned parameter silently, at the one moment the host could still act on the artefact, and would report having resumed nothing when what actually happened was that the state it saved was rejected (`test:crate:recovery-refuses-an-indefinite-checkpoint`).

**Decision (The assessment path does no persistence work)** · `dec:durability:assess-no-persistence`

Assessment writes nothing durable. Not a journal entry, not a counter, not a deferred flush enqueued for someone else to perform: the path that answers a request does no persistence work at all. This is the specification's assess-only guarantee (`inv:guarantee:assess-only`), and the writes the path does perform to in-memory state are enumerated exhaustively rather than described (`inv:runtime:enumerated-writes`).

The consequence is that durability costs the read path nothing, and that a restart loses assessments in flight rather than corrupting anything — they were never durable and never claimed to be.

**Decision (Restore re-applies elapsed decay exactly once)** · `dec:durability:decay-once`

On restore, elapsed decay is applied once, to the checkpointed state, before any journalled label is replayed. The replayed labels then see no further decay: they are replayed as of the moment they were submitted, not as of the moment of the restore. The two-domain timestamp discipline this rests on is the temporal layer's (`dec:operational:temporal-governance`), which fixes which domain a checkpoint may store.

Once is the whole decision. Applying decay per replayed label would decay the same interval as many times as there are labels; applying it after replay would decay updates that had not yet happened when the interval elapsed. Neither alternative replays exactly, which is the point of the corollary below.

**Decision (Structural compatibility is checked, numerical change is permitted)** · `dec:durability:structural-compatibility`

A restore checks that the checkpoint's structure matches the instance being built, and refuses to load a structurally incompatible one. A change to a numerical parameter is explicitly permitted across a restore: an operator may retune and restart without discarding learned state, because the parameter is not part of what the checkpoint's shape means. What a structural mismatch does instead of failing is the construction record's decision, not this one.

Structure is what the payload's encoding carries, and the encoding is positional rather than named, so renaming a field moves no byte and the layout generation does not turn on it. That leaves one residual worth stating rather than discovering: a field may keep its slot, its width and its generation while the quantity it counts moves. The per-model counter that once recorded the retired factorisation shift now records the spectral floor's repairs (`dec:posterior:spectral-floor`), so a checkpoint written before that move restores its old count into the new field. This is admitted rather than tolerated, on the ground that makes the rename honest in the first place: the counter has always answered how often this model's precision needed help, both readings are answers to that question, and the alternative — a generation bump — would refuse every checkpoint written before the move and cold-start deployments that had nothing wrong with them, to protect a diagnostic count. A bump buys precision in a monotone counter and costs a deployment its learned state, which is the wrong trade at this field.

**Decision (Cold ramp progress is checkpointed and resumes where it stopped)** · `dec:durability:ramp-resumption`

The cold standardisation ramp's progress is part of the whole-state checkpoint (`dec:durability:checkpoint-journal`): the base moments it mixes against, the sufficient statistics of the sample it has accepted so far, and the count that positions it are written with everything else and read back with it. A restore resumes the ramp at that count. It neither restarts the ramp nor treats a partial transition as a finished one, which are the only two answers available to a checkpoint that carried the published moments and nothing behind them.

This is the one exception to a rule the vector record makes the other way, and it is taken because the cost differs, not because the reasoning there was wrong. A bootstrap accumulator that is lost costs a bounded slot warm-up and is still deliberately not carried (`dec:vector:transient-accumulators`); a ramp that is lost costs the instance its whole coordinate system and reintroduces at every restart exactly the discontinuity the ramp was adopted to remove (`dec:vector:prior-mass-ramp`). What makes carrying it safe is that it is carried whole — the base travels with the sample that is being mixed into it — so a resumed ramp publishes what an uninterrupted one would have published at the same count, rather than one run's sample against another run's assumptions.

The compatibility question is answered already and is not reopened here. A checkpoint whose structure does not match the instance being built is refused (`dec:durability:structural-compatibility`) and the build cold-starts instead (`dec:construction:two-starts`), which begins the ramp at zero with the priors carrying all the mass. There is therefore no path on which a ramp's accepted sample is applied to a layout it was not gathered under, and no new compatibility rule is needed to say so.

**Decision (Both files open with a signature and a generation)** · `dec:durability:file-framing`

Each of the two files opens with a fixed header, and the header's first two fields are the same in both: four bytes of signature, then the generation as a four-byte little-endian integer. The signatures share a three-byte prefix naming the package and differ in their final byte, which names the role — `ASAY` for the checkpoint, `ASAJ` for the journal. A file offered as the wrong one of the two is refused on that byte, before its generation is read and before any byte of its body is interpreted.

Each file keeps its own generation. The two layouts change for different reasons and there is no requirement that they move together, so a bump to one says nothing about the other. What a generation is read for is the structural check (`dec:durability:structural-compatibility`).

The checkpoint's header carries a third field the journal's does not: a four-byte little-endian CRC32 over the payload that follows it. The asymmetry follows from the two shapes rather than from an oversight. A checkpoint is one opaque body written whole and renamed into place, so a corruption inside it has no structure to catch it and only a checksum will. A journal is append-only and each of its records carries its own length prefix, so the damage a crash can do is a truncated tail, which recovery finds from the prefix alone — and a checksum written into the journal's header could not have covered the records appended after it in any case.

**Convention (Sanitisation precedes journalling)** · `conv:durability:sanitise-before-journal`

A submission is sanitised at the boundary before it is journalled. The journal therefore holds sanitised values only, and replay applies exactly what the original run applied — which is what makes the corollary below true, and is the reason this ordering is a decision rather than an implementation detail. Were the raw submission journalled, replay would re-run sanitisation and would agree with the original run only for as long as the sanitisation rule never changed.

Stated here and nowhere else. The label surface states the consequence a submitter observes and cites this convention for the reason (`dec:contracts:label-surface`).

**Corollary (Each header is as wide as its fields)** · `cor:durability:header-width`

Neither header width is chosen. Each is the sum of the fields the framing gives that file (`dec:durability:file-framing`): signature and generation give the journal eight bytes, and the checkpoint's payload checksum gives it twelve. The two widths therefore differ by exactly the size of the one field the two headers do not share, and a field added to either header moves that file's width and nothing else.

The width is load-bearing at one place in each file — it is the offset at which the body begins — and it is read there by both the writer that emits the header and the reader that skips it. Deriving it from the layout rather than choosing it is what keeps those two from agreeing about a number instead of about a format.

**Corollary (A recorded assessment replays exactly)** · `cor:durability:replay-exactness`

From the once-only decay of (`dec:durability:decay-once`) and the ordering of (`conv:durability:sanitise-before-journal`) it follows that a recorded assessment replays to the same output — the specification's replay guarantee (`inv:guarantee:replay`). Nothing further is required of the replay machinery: it is exact because the two things that could make it inexact, a time-dependent factor applied a variable number of times and a boundary transformation applied twice, are each excluded by a decision above.

**Register (The two divergences this record collapses)** · `reg:durability:collapsed-divergences`

Two statements in the legacy corpus were decided both ways, each by a record that asserted the other preserved. The first is the acknowledgement's contents; the second is the order of sanitisation against journalling. Both divergences are recorded with their evidence in the census (`reg:assayer:adr-contradictions-records`).

Both are now settled in one place each. The ordering is (`conv:durability:sanitise-before-journal`), here. The acknowledgement's contents belong to the label surface and are stated there, because what an acknowledgement carries is a property of the call that returns it rather than of the journal it was written to.

**Remark (Why the writes are enumerated rather than described)** · `rem:durability:write-enumeration`

The specification enumerates the assessment path's writes exhaustively rather than characterising them. The difference matters to this record specifically: a characterisation is checked by reading the code and agreeing with it, while an enumeration is checked by finding a write that is not on the list. Only the second gets stronger as the code grows, and only the second makes the assess-only decision of (`dec:durability:assess-no-persistence`) something other than an assurance.

**Caveat (A restore may validate against fields the checkpoint lacks)** · `cav:durability:restore-field-mismatch`

The census found a legacy restore validating against two checkpoint fields that its own checkpoint structure did not contain (`reg:assayer:adr-contradictions-records`). The defect is worth carrying rather than deleting with the record that held it, because it names the failure mode this area invites: a restore's validation and its checkpoint's shape are written in different places, and nothing makes them agree.

What the rewrite must check instead is that every field the restore validates is a field the checkpoint structure declares. That is a mechanical property and the standing preference is that it become a test rather than a sentence (`goal:assayer:tools-over-discipline`); this record states the obligation and does not claim it is currently discharged.
