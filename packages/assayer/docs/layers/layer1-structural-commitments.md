# Layer 1: Structural Commitments · `spec:structural:architectural-commitments`

This document fixes the conceptual choices of the structural layer. It is the first of the five layer outlines that supply the middle term of the projection (`dec:assayer:projection-principle`): the specification fixes the concepts, the layer outlines fix the choices those concepts leave open, and the decision-record set is derived from the choices. It is not a summary of records. The record census found that the layer documents had only ever summarised the record set by number (`obs:assayer:adr-layers-not-outlines`), which is why this document was written rather than converted.

A *conceptual choice* here is a question the specification poses but does not answer, where the answer binds every implementation of the layer and could have gone another way. Each choice below is stated exactly once. That discipline is the remedy the census prescribed: every contradiction it found was a drift between copies of one statement rather than a disagreement about what to decide (`obs:assayer:adr-contradiction-mechanism`), so the outline states a choice in one place and everything else cites it.

Each choice names, in prose, the record cluster expected to own its decisions (`plan:assayer:decision-record-architecture`). Most of those clusters have since landed, and a choice whose cluster has landed now carries a citation of the landed record's identity decision alongside the naming; a choice whose cluster remains unlanded is still named only. The tracking relation over every landed record is carried once, by the master register (`docs/adrs.md`), and is not repeated here: this outline connects to its records by citing their identity decisions rather than by a tracking table of its own.

## Scope · `sec:structural:scope`

**Summary (What this layer fixes)** · `summ:structural:scope`

The structural layer fixes how the package is shaped before any quantity is computed: which direction information crosses the boundary, how many writers there are and how their work becomes visible, what survives a restart, what happens when something goes wrong, and what shape the model takes. Everything downstream inherits these; nothing here depends on a numeric result.

Eight choices. The census derived six for this layer (`tab:assayer:adr-disposition-structural`); this outline splits two of them, on the reasoning recorded at each.

**Convention (What this layer does not fix)** · `conv:structural:exclusions`

The structural layer fixes no representation, no algorithm, and no public signature. Where a structural choice constrains one — the model's shape constrains the matrix substrate, the publication discipline constrains the snapshot's composition — the constraint is stated here and the consequence is fixed by the layer that owns it (`summ:representation:scope`).

## The conceptual choices · `sec:structural:choices`

**Decision (Information crosses the boundary in one direction)** · `dec:structural:boundary-direction`

Reports cross the boundary from the Sentinel corpus as owned values, and the package holds no handle to any Sentinel and calls nothing back. The Assayer owns its identity graphs outright, so the boundary is external only: nothing inside the package observes a Sentinel except through a report it has already been given. This is the feed-forward invariant (`inv:guarantee:feed-forward`) discharged structurally rather than by discipline, and it fixes what the Core may consume (`tab:boundary:consumed`) against what it may not (`tab:boundary:not-consumed`).

The alternative was a handle or a trait object the package could call, which would have made the boundary a convention about when to call rather than a fact about what exists. Expected owner: the *feed-forward boundary* record (`dec:ownership:feed-forward`).

**Decision (The boundary is enforced mechanically)** · `dec:structural:boundary-enforcement`

The import surface is restricted by an allowlist checked as a test, not by review. A module that may not reach the derivation vocabulary cannot import it, and the check fails the build rather than the reader. The verification table the specification carries (`tab:boundary:verification`) is the statement of what the check must hold, and the check is what makes the statement true on every commit.

The census grouped this with the direction of information flow. It is separated here because it answers a different question: not *which way does information travel* but *what makes the answer hold* — and the campaign's standing rule (`goal:assayer:tools-over-discipline`) makes that question one the layer must answer explicitly rather than inherit. Expected owner: the *feed-forward boundary* record (`dec:ownership:feed-forward`).

**Decision (Coordinates are one fixed internal width)** · `dec:structural:coordinate-width`

Coordinates are carried at a single internal width fixed once (`dec:encoding:fixed-internal-width`), and the package is not generic over the coordinate type. Host-declared domains of any width lift into that one width (`alg:encoding:coordinate-lift`), so every internal structure keyed by position takes one key type.

The alternative was a type parameter threaded through every keyed structure, at the cost of making the identity layer, the Ledger, and the extraction routing generic over something none of them varies. The census grouped this with the boundary direction; it is separated here because it is a representation-forcing choice that the boundary does not imply. Expected owner: the *feed-forward boundary* record (`dec:ownership:feed-forward`).

**Decision (Learned state reaches readers only as a swapped snapshot)** · `dec:structural:publication`

Learned state is published as an immutable snapshot swapped atomically, and readers load it per request rather than per batch, so a publication that lands mid-batch is visible to the remainder of that batch. Per-Sentinel report indices publish under the same discipline independently of the model. Readers therefore never block on a writer (`inv:guarantee:non-blocking`), and what they may read is bounded in staleness rather than in freshness (`inv:guarantee:staleness`), on the interval the specification fixes (`req:publication:interval`) over the snapshot it defines (`def:publication:model-snapshot`).

The alternatives were a lock over shared mutable state, which trades the non-blocking guarantee for freshness, and a per-batch load, which trades visibility for a marginal saving. Expected owner: the *concurrency and publication* record (`dec:concurrency:snapshot-swap`).

**Decision (One writer, two channels, structure before observation)** · `dec:structural:write-serialisation`

One steward owns the working copy exclusively; there is no shared mutable model state. Work reaches it on two channels — structural commands and observations — and structure preempts observation: the command channel drains before any observation is processed, so a registration is never overtaken by the labels that follow it (`inv:guarantee:lifecycle-publication`). Neither channel may silently drop; overflow is an error return to the caller, never a discarded item (`req:publication:label-queue`). The steward is a dedicated named thread rather than a pool or a task, so its identity is fixed and its stalls are attributable. A compound structural event is drained as one unit (`req:registry:compound-events`).

This choice collapses the census's sharpest structural contradiction: the steward loop was rewritten in one record with an unbiased selection and an immediate exit, while two others decided the biased selection and the command drain, each asserting the others preserved (`reg:assayer:adr-contradictions-records`). There is one loop and one selection discipline, stated here and nowhere else. Expected owner: the *concurrency and publication* record (`dec:concurrency:snapshot-swap`).

**Decision (Durability is a checkpoint and a self-contained journal)** · `dec:structural:durability`

What survives a restart is a periodic whole-state checkpoint plus a journal of labels in flight, each journal entry self-contained enough to replay without the state that produced it. The assessment path does no persistence work at all (`inv:guarantee:assess-only`), and the writes it does perform are enumerated exhaustively (`inv:runtime:enumerated-writes`). Restore re-applies elapsed decay exactly once, and labels replayed from the journal see no further decay, which is what makes a recorded assessment replay exactly (`inv:guarantee:replay`). Structural compatibility is checked on restore; a change to a numerical parameter is permitted across it, a change to the structure is not.

The census found the acknowledgement's contents and the order of sanitisation against journalling decided differently in two records. The order is fixed here: a submission is sanitised at the boundary before it is journalled, and the acknowledgement carries only what is known synchronously — the label surface states the consequence and cites this choice (`dec:contracts:label-surface`). Expected owner: the *durability and recovery* record (`dec:durability:checkpoint-journal`).

**Decision (The Core surface is infallible and degrades in band)** · `dec:structural:failure-posture`

Core assessment does not fail. There is no error channel on the assessment call; anomalies are reported in band as degradation travelling with the result (`schema:output:assessment`), alongside the health the result carries (`schema:output:health-snapshot`). Matrix pathologies retain the best available state and flag it rather than failing the call. Where a call can fail, the partition is fixed: a structural contract violation is a hard error, and a data-quality problem degrades the affected cell and reports it.

The alternative — a fallible assessment — would push every numeric pathology into the host's error handling, where the host has no means to act on it. Expected owner: the *degradation posture* record (`dec:degradation:infallible-core`).

**Decision (The model is dense, dynamically dimensioned, and anchored)** · `dec:structural:model-shape`

The model is dense and dynamically dimensioned, reallocated fresh on a lifecycle change rather than grown in place, so a structural operation is exact on the structure it produces (`inv:guarantee:structural-exactness`) and an axis registration preserves the posterior it extends (`inv:guarantee:axis-lifecycle`). The anchor model (`def:risk:anchor-model`) is never extended and never marginalised: it is the fixed member of the triple (`def:risk:model-triple`), and its dimension is invariant across every lifecycle event. The assessment path reads the covariance only and never the precision matrix.

A sparse representation was available and rejected: the feature vector is dense by construction, so sparsity would pay indirection for no zeros. Expected owner: the *model substrate* record (`dec:substrate:dense-dynamic`).

## Unsettled at this layer · `sec:structural:unsettled`

**Entry (Intra-label parallelism, resolved)** · `entry:structural:parallelism-open`

The write-serialisation choice (`dec:structural:write-serialisation`) fixes one writer and originally left open whether independent model updates within a single label may proceed in parallel underneath it. Profiling met that question's premise, and the concurrency record now fixes two scoped helpers beneath the single steward (`entry:concurrency:label-parallelism`). The matching register entry is retired.
