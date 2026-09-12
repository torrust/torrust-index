<!-- Assembled from packages/assayer/docs/spec/ under packages/assayer/docs/spec/assembly.md. Edit the parts, not this file. -->

# Assayer: An Interpreter Between Measurement and Judgement · `spec:spec:measurement-judgement-interpreter`

This specification defines three components and the contract between them and their host. The Core is a stateful Bayesian risk estimation engine that learns from labelled outcomes. The Derivation Function is a stateless, deterministic transform from risk assessments to posture-indexed decision landscapes. The Companion Tracker is a standalone challenge-effectiveness estimator. The host composes the three, decides, acts, and reports what happened.

Two upstream systems meet the Core at deliberately different boundaries. Mudlark supplies the dual-tree value-stratified index that the Core owns inside each identity dimension (`preview:architecture:mudlark`). Spectral Sentinel is the independent measurement instrument whose detached batch reports the Core reads (`preview:architecture:spectral-sentinel`). The background appendix gives enough orientation to both systems for this specification to stand alone; its imported citations lead to the owning corpora for their mechanics.

**Convention (Upstream references)** · `conv:spec:upstream-references`

An upstream corpus owns its mechanisms. This specification states only the orientation and interface facts needed to explain the Core, and each such account cites the owner that carries the full statement. A cross-owner citation uses the imported form of the label grammar — the label qualified by the upstream owner's registered prefix (`inf:labels:imported-citation`) — so the qualification declares the crossing itself. An upstream division number is not copied here as an identity, and a local paraphrase never substitutes for the imported citation.

The two background overviews are therefore Assayer-owned accounts of the boundaries this specification depends on, not replacement specifications for Mudlark or Sentinel. Where an overview and its cited owner could be read differently, the owner controls its mechanism; this document controls what the Core supplies, what it consumes, and what it deliberately cannot reach.

## Part (Foundations) · `part:spec:foundations`

Part I fixes what the rest of the document assumes: what the three components are for and which of them owns what, the contract the host discharges before any Sentinel exists, the asymmetry that structures every learning claim made later, and the Gaussian algebra that makes a changing feature space exact rather than approximate.

### Chapter (Purpose, Scope, and Principles) · `chap:spec:purpose-and-principles`

The chapter is the document's front door. It motivates the split into three components, states the three principles the split serves, fixes the one structural guarantee that holds unconditionally and the sign convention every later chapter reads, draws the architecture and says who owns what, states the five standing constraints the design rests on, and separates what the Core assumes of a Sentinel's report from what it checks on receiving one.

**Motivation (The three questions)** · `mot:architecture:three-questions`

A Sentinel is a universal measuring instrument: it accepts any stream of integer coordinates, maintains an adaptive spatial partition, and produces a multi-scale statistical portrait, with nothing in its machinery specific to any data source. A host operating a real system has many such sources — addresses, agent strings, inter-arrival timings; amounts, merchant categories, geographic codes; sensor readings, equipment identifiers, process parameters — each with its own coordinate space and its own notion of anomalous departure.

From those measurements the host needs three things, and the three are computationally distinct. A calibrated risk estimate — how anomalous this observation looks across every available source, as a probability with uncertainty — requires memory: Bayesian updating over labelled outcomes, forgetting of stale evidence, class-balanced learning, persistent spatial outcome history, and feature composition across a changing source set. A decision landscape — across all levels of caution, what the cost structure implies — requires none: given a risk probability, its uncertainty, the host's declared preferences, and a challenge-effectiveness posterior, the crossover structure is closed form. A challenge-effectiveness estimate — how often a challenge identifies an adverse source — requires a conjugate counter over sparse binary evidence.

Three needs with three characters, so three components. The Core is stateful and learns. The Derivation Function is stateless and pure. The Companion Tracker is standalone and conjugate. The Core does not decide; the Derivation Function does not learn; the Companion does not interpret risk. The host composes them, and the Derivation Function and the Companion are consulted, not consulted back.

**Principle (Identity preservation)** · `prin:principle:identity-preservation`

Structure is preserved through the information pathway rather than summarised away at its entrance. Each Sentinel produces a tree of scores indexed by spatial scale and statistical axis. The Core preserves per-Sentinel identity through named feature slots, preserves per-axis identity through structured extraction, and summarises the collective measurement state only where a summary is what is wanted — through aggregate features whose meaning does not shift with the number of active Sentinels. The risk model is then presented with a feature vector whose dimension adapts to the active Sentinel set and the active outcome-axis set through exact Bayesian algebra — extension (`thm:gaussian:extension`) and marginalisation (`thm:gaussian:marginalisation`) — so that adapting the structure costs no information. A measurement that arrives with identity keeps it until something downstream has a reason to spend it.

**Principle (Multi-level decomposition)** · `prin:principle:multi-level-decomposition`

Evidence is decomposed across levels rather than pooled into one question. Three decompositions do this work. How alarmed is each specialist is separated from how alarmed the specialists are collectively — per-Sentinel named slots against aggregate features, both inside the Core. How inherently dangerous this is is separated from how much damage is getting through — the sister and operational models of the risk triple (`def:risk:model-triple`), with a counterfactual anchor supplying coarse directional coverage where the sister model is starved. And how dangerous this is is separated from what should be done about it — the Core produces a risk assessment, and the Derivation Function maps it onto the posture axis as a crossover structure, with uncertainty attributed to its two evidence sources. Classification belief remains a Core quantity; action boundaries are the Derivation Function's. All three separate concerns with different convergence rates, different data requirements, and different stability properties.

**Principle (Measure, not decide)** · `prin:principle:measure-not-decide`

The Core measures and the host decides. The Core produces risk assessments, outcome-axis predictions, and diagnostic reports. The Derivation Function produces posture-indexed decision landscapes and exploration signals. Neither selects an action, overrides the host, or controls the observation stream — neither has an interface through which it could. The host receives the Core's assessment, obtains the Derivation Function's landscape, reads the Companion's estimate, decides, acts, and reports what happened; and then, and only then, the Core learns. The separation is not deference. A component that both estimated risk and acted on its estimate would have no way to distinguish what it observed from what it caused, and every claim this document makes about calibration would lose its meaning.

**Invariant (Feed-forward)** · `inv:guarantee:feed-forward`

Information flows in one direction through the measurement, interpretation, and decision stack:

$$\text{Sentinels} \longrightarrow \text{Core} \longrightarrow \text{Host}$$

The Core reads Sentinel batch reports. It never writes to a Sentinel, never modifies a Sentinel's spatial partition, never queries a Sentinel for per-request scores, and never pushes configuration to a Sentinel. The Core's internal state — its models, its outcome memory, its identity layer, its pending buffer — is invisible to every Sentinel. The Derivation Function reads the Core's output and the host's declared policy, and writes nothing: no state, no side effects, no persistent resources.

The only cycle in the system passes through the host: the Core assesses, the host decides and acts, the host reports what happened, the Core learns. That cycle is the intended feedback path, and no other exists. A Sentinel's behaviour is therefore identical whether the Core exists or not, and the Core cannot degrade any Sentinel's measurement quality. Failures in the Core — miscalibrated models, stale outcome entries, label corruption — propagate upward to the host, where they belong, and never downward into the measurement layer.

**Sign convention (Valence)** · `conv:valence:sign`

Valence is the host-reported outcome magnitude. Positive valence is the adverse direction — fraud, policy violation, harm; non-positive valence is benign or neutral. The risk model reads the sign alone, and the magnitude enters only where a declared outcome axis is what is being predicted. The convention is fixed here, at its first statement, and every later chapter that needs it cites this environment rather than restating it: a sign convention restated in two places is a sign convention that will eventually disagree with itself.

**Figure (Architecture overview)** · `fig:architecture:overview`

```text
                    ┌──────────────────────────────────────────────────────────┐
                    │                       The Core                           │
                    │                                                          │
  Sentinel₁ ───────►│  ┌─────────────┐                                         │
  Sentinel₂ ───────►│  │ Interpreter │   ┌─────────────┐                       │
  Sentinel₃ ───────►│  │             │──►│  Estimator  │  ┌──────────────────┐ │
      ...           │  │ Per-sentinel│   │             │─►│  Outcome Ledger  │ │
  Sentinel_n ──────►│  │ extraction  │   │ Risk triple │  │                  │ │
                    │  │ Aggregation │   │ Outcome     │  │ Per-sentinel     │ │
  Host ────────────►│  │ Identity    │   │ axis models │  │ spatial outcomes │ │
  Context+Signals   │  │ Signals     │   │             │  │                  │ │
                    │  └──────┬──────┘   └──────┬──────┘  └──────────────────┘ │
                    │         │                 │                              │
                    │         └────────►  RiskAssessment                       │
                    │                          │                               │
                    └──────────────────────────┼───────────────────────────────┘
                                               │
              ChannelPolicy ─────►┌────────────────────────┐
              Derivation config ─►│  Derivation Function   │─► DecisionLandscape ─► Host
                                  │  (stateless, pure)     │
                                  └───────────▲────────────┘
                                              │
              ┌────────────────────┐          │
              │  Companion Tracker │──────────┘
              │  (Beta-Binomial)   │◄── Challenge results from host
              └────────────────────┘
```

The drawing carries the derivation configuration as an input in its own right, beside the channel policy. It is drawn because it is supplied: the display and exploration parameters reach the Derivation Function on every call, and a drawing that omitted them would show a transform of two arguments where the system has three.

**Table (Component ownership)** · `tab:architecture:ownership`

Seven internal components across three ownership boundaries. Each row binds: the component named owns exactly the state named, and no other component reads or writes it. This is the document's only enumeration of the ownership partition, and the boundary appendix's verification of the interface is checked against it.

| Component | Owner | Responsibility | State |
| --- | --- | --- | --- |
| Sentinel Registry | Core | Track active Sentinels, manage lifecycle, dimension map | Registry of identifier, name, and state |
| Outcome Axis Registry | Core | Track active outcome axes, manage lifecycle, per-axis prediction models | Registry of identifier, name, model, compression, and eligibility |
| Interpreter | Core | Per-Sentinel extraction, aggregation, identity management, feature composition | Identity layer, signal cache, feature normalisation, per-Sentinel extractors |
| Estimator | Core | Risk triple, outcome-axis models, learning, drift detection | Model parameters at the current dimension, pending buffer |
| Outcome Ledger | Core | Per-Sentinel spatial outcome memory | Per-Sentinel, per-cell outcome statistics |
| Derivation Function | Derivation | Posture-indexed decision landscape from risk assessment and host policy | None; a pure function |
| Companion Tracker | Companion | Challenge-effectiveness estimation from sparse challenge outcomes | Beta pseudo-counts and a timestamp |

**Example (Compositional dividends)** · `ex:architecture:compositional-dividends`

The three-component design buys five things a monolith cannot express, and they are one argument in five movements rather than five separate claims.

Multiple channels, one assessment: the Core assesses a request once, and the host derives a landscape per channel — login, interface, transaction — each with its own action set and cost structure. The Core's quadratic cost is paid once, and each further channel is negligible. Assessment-only deployment: a host with its own decision logic consumes the calibrated probability directly, and needs neither the Derivation Function nor a Companion. External derivation: the risk basis can be shipped to a separate service that derives under a different policy, so that decision policies are compared without retraining models — and because the landscape depends on no presentation parameter, such a comparison isolates the cost structure exactly. Independent Companion lifecycle: the conjugate tracker can be replaced by any other challenge-effectiveness estimator, an external analytics pipeline included, leaving the Core and the Derivation Function untouched. Derivation replay: a stored risk assessment, policy, and challenge posterior reproduce their landscape exactly, without model state — which is what makes audit, compliance, and debugging possible without access to the models.

The third and fifth movements rest on the landscape being a function of evidence and policy alone. Where a presentation parameter reaches the derived output, the isolation the third claims and the reproducibility the fifth claims both weaken to the extent that it does.

**Assumption (Dynamic registries)** · `assum:constraint:dynamic-registries`

The number and identity of active Sentinels may change at any time. A Sentinel is added when a data source becomes available, removed when one is retired, and temporarily absent when it has no data for a given request. The Core absorbs these changes through exact Bayesian extension (`thm:gaussian:extension`) and marginalisation (`thm:gaussian:marginalisation`): no retraining, no matrix surgery, no label loss. Lifecycle transformations are unconditionally exact; per-observation updates are exact per step under a posterior-dependent effective precision, with conservative departure from any fixed generative model's posterior (`thm:gaussian:two-level-guarantee`). The algebra is developed in (`chap:spec:mathematical-foundation`); the registry mechanics that invoke it belong to the registry chapter.

**Assumption (Valence asymmetry)** · `assum:constraint:valence-asymmetry`

Outcomes are observed on one side of the action only. The system learns the inherent risk of observations the host allows to proceed, and nothing about those it restricts. Any host policy that restricts observations on the strength of the system's risk estimates simultaneously starves the system of the evidence those estimates need. The tension is structural and irreducible, and the Core cannot resolve it; what the architecture can do is make the cost of censoring visible, measurable, and actionable. The constraint is developed in (`chap:spec:valence-asymmetry`).

**Assumption (Crossover structure)** · `assum:constraint:crossover-structure`

The decision surface is a crossover structure, not a threshold. The conventional pipeline classifies first and decides second; this design dissolves the sequencing. The Core produces a scalar probability with uncertainty, and the Derivation Function maps it onto the posture axis as a crossover structure — the postures at which the optimal action changes, each carrying uncertainty from two evidence sources. Classification belief is carried alongside as a Core quantity rather than consumed by the mapping. The derivation is developed in the landscape chapters.

**Assumption (Encoding dependence)** · `assum:constraint:encoding-dependence`

The measurement layer's quality is determined entirely by the host's encoding decisions, and everything downstream inherits it. Every Sentinel analyses integer coordinates under the assumption that shared prefixes imply shared context. A Sentinel fed data without hierarchical positional structure produces scores that are statistically valid and semantically vacuous, and the Core has no mechanism to detect this and no ability to compensate for it. The responsibility is the host's, and it is the single most consequential decision in a deployment (`req:encoding:host-contract`). The constraint is developed into a full contract in (`chap:spec:encoding-contract`).

**Assumption (Outcome axes)** · `assum:constraint:outcome-axes`

Beyond the primary risk signal, the host may register any number of secondary outcome axes at runtime, and the Core does not interpret any of them. Each axis receives its own Bayesian prediction model, and each axis's outcome history feeds back as features into the risk models and into every other axis's prediction model. The Core predicts and reports with calibrated uncertainty, and says nothing about what an axis means. Axis lifecycle shares the extension and marginalisation algebra that governs Sentinels, so registering an axis at runtime costs no more information than registering a Sentinel does.

**Table (Assumed Sentinel properties)** · `tab:architecture:sentinel-properties`

Six properties of the batch-report interface that the Core assumes and does not check. They bind the Sentinel, not the Core: a report violating one of them is outside the contract, and the Core's behaviour on such a report is unspecified. They are properties of the interface, never of a Sentinel's internal mechanism — the Core does not depend on, and does not reason about, any Sentinel's tree structure, competitive dynamics, spatial lifecycle, or warm-up pipeline. Those mechanisms determine what appears in the report; the Core consumes the report.

| Property | Statement |
| --- | --- |
| Well-formed reports | Each reported cell carries well-formed scores, baselines, drift state, and maturity across all four axes, and ancestor-chain reports are complete from cell to root |
| Deterministic report lookup | A coordinate maps deterministically to report data for feature extraction; ancestor reports may also cover the coordinate at shallower depths |
| Coordination reports reference reported cells | Coordination context members are a subset of the reported competitive cells |
| Contour snapshot present | Each report includes aggregate structural metadata: cell count, plateau count, total importance, and structural mutation summaries |
| Feed-forward compatibility | The Sentinel's operation is independent of the Core's state, and no Core output influences any Sentinel |
| Consistent declared width | Coordinates and cell intervals are interpreted in the Sentinel's declared domain width, which the Core normalises to its internal width on reception (`def:encoding:domain-width`) |

The one place the Core operates a graph of the Sentinel's kind directly is the identity dimension layer, whose graph instances the Core owns, operates, and observes into. That is a structural relationship and not an observational one; the relationship between the Core and each Sentinel is purely observational.

**Precondition (Report validation)** · `pre:architecture:report-validation`

Two further properties of the batch report are not assumed but enforced. On reception, before any extraction, the Core checks the report's structure and rejects it with a named error where the structure fails.

The root cell — depth zero, covering the full domain — must be present in every batch report; a report without one is rejected as missing its root cell. And reported cells must be dyadic at their claimed depth and must not overlap, so that a coordinate falls into at most one reported cell at any depth; a cell whose interval is not dyadic at its depth is rejected as a non-dyadic interval, and two cells sharing a position and depth are rejected as a duplicate cell.

The distinction matters for where a defect surfaces. An assumed property that fails produces wrong numbers with no diagnostic; an enforced one produces a named rejection at the boundary, which is the report's author's to repair. These two are enforced because they are cheap to check and catastrophic to violate: every routing decision the Core makes downstream reads the dyadic hierarchy, and the root is the entry point of every ancestor chain.

**Remark (Inherited guarantees)** · `rem:architecture:inherited-guarantees`

A Sentinel's internal guarantees reach the Core through the report and are never referenced by it. The summation invariant, tip-only eviction, frozen benchmarks, the depth bound on the ancestor chain, budget enforcement, and competitive stability are all properties the Core relies on having, and none of them is a property the Core can name, query, or verify. They arrive as the shape of a well-formed report and in no other way.

This is what the assumed-properties table means operationally, and it is worth stating separately because the reading it rules out is the tempting one. The Core does not inherit these guarantees by depending on the Sentinel's implementation; it inherits them by depending on the interface, and would inherit them equally from any other producer of conformant reports. A Sentinel that changed every one of those mechanisms while continuing to emit conformant reports would change nothing about the Core.

### Chapter (The Encoding Contract) · `chap:spec:encoding-contract`

The chapter develops the encoding constraint (`assum:constraint:encoding-dependence`) into a contract. It states what the host owes before a Sentinel exists, gives the guidance a host needs to discharge it, and fixes the one width relationship the Core and the host must agree on. The contract binds; the guidance between it and the width does not, and is not written as though it did.

**Requirement (The host's encoding contract)** · `req:encoding:host-contract`

Every Sentinel analyses a stream of integer coordinates in its declared domain under the assumption that shared prefixes imply shared context. The Core combines the statistical output of several Sentinels, each watching a different facet of the data, possibly at different widths. The quality of the Core's risk estimates is bounded above by the quality of the Sentinels' measurements, which is bounded above by the quality of the host's encoding decisions.

The encoding decision is the host's, and it is the single most consequential decision in a deployment. Before a Sentinel is created, the host answers the three questions of this chapter for the data source that Sentinel will watch, and declares its answers at registration: what the coordinate space is, what a shared prefix asserts about two coordinates in it, and how deep the meaningful structure runs. The declarations exist for three reasons — to force engagement with the encoding question before a Sentinel exists rather than after it has been running for a quarter, to make the deployment's encoding assumptions auditable by someone who did not make them, and to give a diagnosis of an uninformative Sentinel somewhere to start.

The Core has no mechanism to detect a bad encoding and no ability to compensate for one. Two things partially compensate and neither substitutes: the health report's association reading (`def:monitoring:slot-association`) raises an early warning where a block's coordinates have stopped telling the outcome anything, and the model eventually stops leaning on features that carry no information (`rem:encoding:downstream-fallback`). A partial compensation that is mistaken for the contract is worse than no compensation, because it converts a decision the host must make into one the host believes has been made for it.

**Definition (Hierarchical structure)** · `def:encoding:question-hierarchy`

The first question is whether the data source has hierarchical positional structure at all. Values whose leading bits encode progressively finer categorical or spatial membership — where a shared prefix implies shared context — are suitable. Values with pseudo-random, encrypted, hashed, or uniformly distributed bit patterns are not.

The failure mode is silent, which is why the question comes first. Processing unsuitable values produces a Sentinel that runs without error, whose spatial partitioning is arbitrary, whose multi-scale analysis is noise, and whose four-axis scores are statistically valid and semantically vacuous. Nothing in the system reports an error, because nothing has failed: the instrument measured exactly what it was given.

**Definition (What a shared prefix asserts)** · `def:encoding:question-meaning`

The second question is what the prefix hierarchy means for this source. A Sentinel's partitioning creates cells at dyadic boundaries, and a cell at a given depth groups all values sharing a prefix of that many bits. The host must ensure that this grouping corresponds to a grouping that is meaningful for the data source.

For network addresses the correspondence is automatic: a routing prefix corresponds to a cell at the matching depth, grouping addresses in one subnet. For geographic codes it depends on the encoding scheme. For categorical data the host must choose an encoding under which the categories' own tree structure maps onto the binary prefix tree, rather than assuming that any integer encoding will do. For multi-dimensional data the host must choose a space-filling curve (`tab:encoding:curve-guidance`), and the choice determines which notion of proximity the prefix hierarchy captures.

**Definition (Depth of meaningful structure)** · `def:encoding:question-depth`

The third question is whether the hierarchical structure runs deep enough to justify multi-scale analysis. A Sentinel's value proposition is that it discovers structure at every resolved depth and provides ancestor-chain coverage through models at every ancestor level. Where a source has only two or three meaningful hierarchical levels, the Sentinel creates a few spatial levels before the prefix structure stops meaning anything: the ancestor chain is short, the coordination tier has few contexts, and the system works while providing modest value over a simple per-group anomaly detector.

Sources with deep, meaningful hierarchy exploit the architecture fully — network addresses, whose prefix structure runs to the full address width; fine-grained geographic codes, with twenty levels or more; richly branching categorical taxonomies. The question is not whether the source has many distinct values but whether it has many meaningful levels.

**Table (Suitable sources)** · `tab:encoding:suitable-sources`

Sources whose prefixes carry meaning, and what the meaning is. The table is guidance for answering (`def:encoding:question-meaning`), not an enumeration of what the Core accepts.

| Source | Natural hierarchy | What a shared prefix means |
| --- | --- | --- |
| Network addresses | Network prefix tree | The same network, allocation, or autonomous system |
| Hierarchical geographic codes | Spatial containment | The same region, at progressively finer scale |
| Categorical taxonomies with positional encoding | Category tree | The same branch of the taxonomy |
| Temporally bucketed sequences with bit-interleaved timestamps | Temporal proximity | The same time window |
| Multi-dimensional data mapped through a space-filling curve | Spatial proximity, curve-dependent | The same spatial neighbourhood, with the curve's locality properties |

**Table (Unsuitable sources)** · `tab:encoding:unsuitable-sources`

Sources whose prefixes carry no meaning, and why. Each fails (`def:encoding:question-hierarchy`), and each fails silently.

| Source | Why unsuitable |
| --- | --- |
| Cryptographic hashes | Deliberately destroy positional structure |
| Random session tokens | No prefix semantics |
| Uniformly distributed synthetic identifiers | No hierarchy |
| Raw floating-point values without quantisation | The bit-level structure of the representation does not correspond to value proximity in the way the analysis requires |
| Categorical data with arbitrary integer labels | Two adjacent labels sharing a prefix is an artefact of the encoding, not a semantic relationship |

**Table (Space-filling curve guidance)** · `tab:encoding:curve-guidance`

For multi-dimensional data mapped to a single coordinate stream, the curve determines which notion of proximity the prefix hierarchy captures.

| Curve | Locality property | Best for |
| --- | --- | --- |
| Bit-interleaved ordering | Axis-aligned rectangles are contiguous | Data with axis-aligned structure |
| Hilbert ordering | Compact regions are contiguous | Data with isotropic spatial structure |
| Double-referenced interleave | Vocabulary-change-aware rank encoding | Joint structure discovery between two one-dimensional vocabularies |

A wrong curve does not produce errors. It produces a Sentinel whose partitioning cuts across the data's natural structure rather than along it, reducing multi-scale analysis to noise at exactly the scales where the misalignment falls.

**Remark (Downstream learning is not the remedy)** · `rem:encoding:downstream-fallback`

The Core accepts whatever the host provides. It extracts features, learns weights, and produces risk estimates from any Sentinel's output. If a Sentinel's measurements are noise, the Core will eventually learn from labelled outcomes that its features are uninformative, and downweight them.

This self-correction is real and slow, and it is worth being precise about what it does, because the obvious reading of it is wrong. What converges to nothing is the block's contribution to the score — measured, the removal cost of a hash-fed Sentinel fell from $0.125$ of the score's loss towards $0.0005$ as the model converged (`def:monitoring:slot-contribution`). What does not converge to nothing is the weight the model carries there: standardised-space ridge weights on uninformative coordinates settle at noise scale, and the mean absolute slot weight of that same Sentinel stayed within a tenth of a perfect control's throughout (`def:monitoring:encoding-effectiveness`). The learning stops relying on the features; it does not empty their weights.

The correction costs hundreds of labels and wastes model capacity while it runs, and during that window the deployment is paying full price for a Sentinel contributing nothing. The association reading (`def:monitoring:slot-association`) is intended to surface the failure earlier than either of those, because it is model-free: it reads the stream rather than what the model made of it, and so has no convergence to wait for.

Neither is the remedy. The correct remedy is not to rely on downstream learning to compensate for upstream encoding failures; the correct remedy is to get the encoding right. Downstream learning is what limits the damage of an encoding error, not what licenses one.

**Definition (Domain width)** · `def:encoding:domain-width`

Each Sentinel analyses coordinates in its own native domain $[0, 2^{N_s})$, whose bit-width $N_s$ the host declares per data source: 32 for a four-octet network address, 128 for a sixteen-octet one, 40 or 48 for a bit-interleaved geographic code. Different Sentinels in one Core may declare different widths; the Core places no uniformity constraint across the registry.

The Core's internal coordinate width is fixed at 128 bits. Coordinates are stored and compared at that width in every per-Sentinel spatial structure — the report index, the outcome memory, the pending buffer's coordinate store — and on every internal routing path, which fixes a single arithmetic width across the Core.

The declared width is an upper bound the Core checks. A registration declaring a width greater than the internal width is rejected at registration (`alg:registry:sentinel-registration`), and that is the only width constraint the Core imposes. Every other consequence of the declared width is the host's: the Core does not verify that reported coordinates lie within the declared domain, and does not verify that the declaration describes the source at all.

**Algorithm (Coordinate lift)** · `alg:encoding:coordinate-lift`

For a Sentinel whose declared width $N_s$ is narrower than the internal width, coordinates are left-justified into the internal representation at the integration boundary:

$$x_{\text{internal}} = (x_{\text{native}} \text{ as u128}) \ll (128 - N_s)$$

The lift is the responsibility of the integration layer that delivers batch reports to the Core. The host, or a thin adapter, widens the batch report by left-justifying every reported cell's start and end by the shift above, and lifts the entity coordinates used for per-Sentinel routing at assessment and label time identically. The Core's report-reception surface accepts a post-lift report, matching its internal representation; the Core does not perform the lift, and has no way to tell a lifted coordinate from a natively wide one.

Cell-depth semantics are preserved exactly. A native cell at a given depth covers a dyadic interval of a width set by the declared domain; after lifting it covers a dyadic interval of the corresponding width in the internal representation, at the same depth. The receiving-cell relationship, the ancestor chain, and the all-layers routing of outcome memory are properties of the dyadic hierarchy, and the shift preserves that hierarchy, so none of them is affected.

**Decision (Fixed internal width)** · `dec:encoding:fixed-internal-width`

The internal coordinate width is fixed rather than generic over a per-Sentinel coordinate type. Making it generic would require either enum erasure or trait-object dispatch across every per-Sentinel container — the report index, the outcome memory, the pending buffer — adding dispatch cost and structural complexity for a use case that never arises: no operation in this architecture compares coordinates across Sentinels, and the feed-forward invariant (`inv:guarantee:feed-forward`) is part of why none can.

The cost of the fixed width is bounded. A Sentinel declaring a narrow domain leaves the high bits of its internal coordinates unused, but its storage footprint is dominated by cell count rather than by coordinate width, so the waste does not scale with anything that grows. The fixed width is the cheaper design, and it fixes one arithmetic width across the Core rather than distributing a width question over every internal path.

**Remark (The declared width is unchecked)** · `rem:encoding:unchecked-width`

The declared domain width is an assertion, not a measurement. Beyond rejecting a declaration wider than the internal width, the Core does not check it: it does not verify that reported coordinates fall inside the declared domain, does not verify that the lift was performed at the declared shift, and cannot detect a declaration that is simply wrong.

A wrong declaration is therefore silent in the same way a wrong encoding is. If the lift shifts by the wrong amount, every coordinate lands at the wrong place in the internal representation, every cell interval is dyadic at the wrong depth, and the outcome memory accumulates against a hierarchy that does not correspond to the source's. Nothing reports an error, because at the internal width the values remain well-formed.

The correspondence between the declared width, the lift, and the source is the host's to maintain. This is the same division of responsibility the contract states (`req:encoding:host-contract`), applied to the one number in it the Core consumes arithmetically.

### Chapter (The Valence Asymmetry) · `chap:spec:valence-asymmetry`

The chapter develops the valence constraint (`assum:constraint:valence-asymmetry`). It argues for the design rather than constraining it: it states the asymmetry and the shape of its cost, enumerates the architectural layers that manage it, traces the contamination loop the asymmetry creates when it meets a persistent spatial memory, and follows the same loop through the outcome axes. Nothing here binds the implementation; what binds is stated where the mechanisms are.

**Principle (The censored asymmetry)** · `prin:valence:censored-asymmetry`

Learning about danger requires exposure to danger. The system learns the inherent risk of observations the host allows to proceed; an observation rejected before it reaches an outcome produces no outcome signal, and the system never discovers whether it would have been adverse or benign. Any host policy that restricts observations on the strength of the system's risk estimates simultaneously starves the system of the evidence those estimates need.

This is the censored bandit problem, and it is structural. It cannot be resolved by better models, more data, or smarter features; any system that both estimates risk and informs restriction decisions faces it. The Core cannot solve it. What the architecture is designed to do instead is make the cost of censoring visible, measurable, and actionable at every level.

The asymmetry has a specific shape, and the shape is what makes it costly rather than merely inconvenient. Benign observations that are rejected are invisible: the host pays an opportunity cost, a good case denied, and the system learns nothing from it. Adverse observations that are allowed are visible: the host pays an adverse-outcome cost, and the system learns something it could not have learned otherwise. So the cost of learning is asymmetric, the information value of observations is asymmetric, and the two asymmetries point in opposite directions — the observations worth most to the model are the ones the host has most reason to refuse.

**Discussion (Mitigation layers)** · `disc:valence:mitigation-layers`

Five architectural layers address the asymmetry, each detailed where its mechanism is specified.

Structural separation (`def:risk:model-triple`). Two risk models within the Core train on different observation sets. The operational model trains on all labelled outcomes, including those of restricted observations, where the host's intervention may have influenced the result. The sister model trains only on unconfounded outcomes — observations allowed to proceed without intervention. The divergence between their estimates reveals how much value the host's interventions are providing, and the sister model's estimate is the system's best judgment of inherent risk independent of intervention posture, subject to the coverage qualification stated with the model.

Coarse directional coverage (`def:risk:anchor-model`). A fixed-dimension anchor model uses only aggregate features — the loudest alarm across Sentinels, cross-Sentinel concordance, identity history — so that a risk estimate exists where the sister model is starved. Having fewer parameters it converges faster and degrades gracefully under starvation. It cannot capture what the sister model learns and is not meant to: it is a floor of competence beneath the sister model, not a substitute for it, and the label count at which that floor becomes useful is about thirty, the count the convergence budget derives from the anchor's dimension (`bound:resource:convergence-budget`).

Embedded exploration signalling (`def:fragility:definition`). The Core's assessment uncertainty and the Derivation Function's decision fragility jointly quantify the exploration cost of restriction: fragility is high exactly when the labelled outcome could change the optimal action, and its variance decomposition says whether the deficit is risk evidence or challenge evidence. The system does not force the host to incur that cost; it measures it and reports it.

Active label guidance (`sig:guidance:interface`). The Core ranks pending requests by model uncertainty and identifies regions starved of eligible feedback, directing the host's investigation budget at the observations that would most improve the estimates.

Host investigation (`conv:eligibility:ground-truth`). The host may investigate any observation, restricted ones included, and report ground truth under a flag marking the label unconfounded whatever action was taken — which bypasses the censored bandit problem for that observation entirely. The system's ability to learn from restricted observations is therefore a function of the host's willingness to investigate, not a limit of the architecture.

**Algorithm (The contamination loop)** · `alg:valence:contamination-loop`

The asymmetry interacts with the system's persistent spatial outcome memory to create a self-reinforcing loop. The outcome memory (`def:ledger:purpose`) maintains, for each Sentinel cell, a running average of past outcomes that decays over time; a label updates the entry for the cell covering its coordinate, and the running average is extracted as a feature in the risk model's input vector. That is all the mechanism the loop needs.

The loop runs in eight steps. A transient adverse event inflates a cell's adverse-outcome rate. The event ends and the Sentinel's scores normalise, which the Core observes as falling scores in subsequent reports. The outcome entries covering the affected range nonetheless retain their elevated averages — under all-layers routing, the deepest entry and every ancestor entry alike. Those elevated features produce an elevated risk estimate for observations routed through the cell. The estimate causes the host to restrict them. Restriction reduces the flow of eligible labels to the cell. Reduced label flow slows the decay of the averages. And the loop closes: stale reputation sustains starvation, and starvation sustains stale reputation.

Absent decay indexed by time rather than by label arrival, the loop's half-life is dominated by label arrival and is unbounded in a fully starved cell — a cell that has stopped receiving labels has stopped forgetting. The quantitative timescales are worked in (`tab:ledger:loop-timescales`), and this chapter takes its numbers from there rather than restating them.

Four mitigations address the loop, each detailed where it is defined. Time-indexed decay of the outcome memory (`def:ledger:time-decay`) erodes the averages in proportion to wall-clock time rather than to label arrival — at the default rate of $0.999$ per hour, a half-life of roughly 29 days — so stale reputation dissolves regardless of label flow. This is the primary mitigation and the only one that bounds the loop's persistence at all: it converts a half-life the host cannot predict into one the host can read off a configured constant. The measurement-only anchor feature (`prop:risk:anchor-measurement-only`) gives the anchor model one input drawn purely from measurement, bypassing the outcome memory, so alarm remains detectable where the memory is contaminated. Exploration signalling elevates fragility for requests in contaminated regions — precisely because the system is uncertain there — making the cost of continued restriction visible. And starvation-relief guidance (`def:guidance:starvation`) identifies entries starved of eligible feedback and recommends them for investigation.

The opposite concern — an adversary exploiting the forgiving half of the same symmetric decay — is a distinct exposure, disclosed as a limitation (`cav:limitation:laundering`) rather than mitigated here: a decay that dissolves stale adverse reputation dissolves stale adverse evidence by the same mechanism, and the design accepts that.

**Discussion (The axis pathway)** · `disc:valence:axis-pathway`

Where an outcome axis is registered with spatial features enabled, that axis's per-cell running average creates a contamination pathway parallel to the primary one. The mechanism is identical: an extreme axis value inflates the cell's axis average, the elevated average contributes to an elevated risk estimate through the Core's learned weights, the elevated estimate drives restriction, restriction starves eligible labels, and starvation slows the decay. Axes registered without spatial features hold no per-cell state and are not subject to the pathway at all (`disc:registry:spatial-policy`).

The graded axis signal can perturb the memory more per event than the binary adverse rate does: a single extreme value can saturate the compressed average to its bound, where the binary rate shifts by the complement of its smoothing factor, approximately $0.001$ per event at the default. The risk impact of that larger perturbation is nonetheless bounded by the Core's learned weight on the axis feature, which reflects the feature's actual predictive value — a feature that moves a great deal and predicts nothing is weighted accordingly.

The mitigations are the same mitigations. Time-indexed decay resolves binary and graded signals on the same 29-day timescale, because it is indexed by time and not by what the signal is; the measurement-only anchor feature bypasses every feature drawn from the outcome memory, axis averages included; and exploration signalling and starvation-relief guidance operate at the cell level, regardless of which of the cell's features are contaminated.

### Chapter (Mathematical Foundation) · `chap:spec:mathematical-foundation`

The chapter states the algebra the whole lifecycle rests on. A Bayesian linear model maintains a Gaussian posterior over parameters $\theta \in \mathbb{R}^p$:

$$\theta \sim \mathcal{N}(\mu, \Sigma), \qquad B = \Sigma^{-1}$$

where $\mu$ is the posterior mean, $\Sigma$ the posterior covariance, and $B$ the posterior precision. Observations evolve these through a Sherman–Morrison update (`alg:update:sherman-morrison`). Each per-observation update is the exact Bayesian update given the current posterior and the step's effective observation precision; because the leverage bound makes that precision posterior-dependent, the composite posterior is not the posterior of any fixed generative model (`cav:gaussian:fixed-model-departure`). The exact identities for the two operations that change the dimension of $\theta$ are finite, and this chapter states them as results so that the lifecycle chapters may cite them rather than re-derive them. Marginalisation regularises one identity and may fall back from it, so the exact theorem and the numerical contract are kept separate below.

**Theorem (Extension)** · `thm:gaussian:extension`

Adding $r$ new dimensions with an independent prior:

$$\mu' = \begin{pmatrix} \mu \\ \mu_0 \end{pmatrix}, \qquad \Sigma' = \begin{pmatrix} \Sigma & \mathbf{0} \\ \mathbf{0} & \Sigma_0 \end{pmatrix}, \qquad B' = \begin{pmatrix} B & \mathbf{0} \\ \mathbf{0} & B_0 \end{pmatrix}$$

The existing posterior is the exact marginal of the extended posterior over the original dimensions. No information is lost and no approximation is introduced. The new dimensions are independent of the existing ones under the prior; future observations populating both old and new features create the cross-terms through the ordinary update. Extension is what makes registering a Sentinel or an outcome axis at runtime cost nothing: the model that existed before the registration is recoverable from the model that exists after it, exactly.

**Theorem (Marginalisation)** · `thm:gaussian:marginalisation`

Removing $r$ dimensions by integrating them out. Partition the indices into kept ($k$) and removed ($r$):

$$\mu' = \mu_k, \qquad \Sigma' = \Sigma_{kk}, \qquad B' = B_{kk} - B_{kr}\,B_{rr}^{-1}\,B_{rk}$$

The marginal mean is the subvector, the marginal covariance the submatrix, and the marginal precision the Schur complement. All three identities are exact for the current joint posterior. They do not describe the counterfactual posterior that training without the removed features would have produced, because those features may have changed observation leverage along the way. The lifecycle uses the regularised algorithm below, so its numerical precision is an approximation to this theorem whenever the coupling is non-zero and $\delta>0$.

**Definition (Gathered removed partition)** · `def:gaussian:gathered-partition`

The removed set need not be contiguous. The dimension map (`def:dimension:layers`) supplies contiguous ranges for physical blocks and slots and exact range lists for composite lifecycle scopes, which lets the removed partition be identified from a Sentinel identifier, an outcome-axis identifier, an identity-dimension identifier, or a competitive-cell identifier even where the removed indices do not occupy one slice of the layout.

The Schur algebra requires only the partition into kept and removed, never that the removed indices be adjacent. An implementation may therefore gather or permute the removed indices transiently, apply the Schur complement, and compact the surviving layout afterwards. The gathering is a matter of arrangement and changes no result: the marginal is determined by which dimensions are removed, not by where they sat.

**Remark (Why precision and covariance are both tracked)** · `rem:gaussian:dual-tracking`

Maintaining both $B$ and $\Sigma$ accumulates small floating-point drift between periodic Cholesky recomputations. In exact arithmetic $\Sigma_{kk} = (B')^{-1}$; in floating-point the extracted covariance and the Schur-complement precision inherit independent errors from their two parent matrices, so the pair can drift apart while each remains individually plausible.

The remedy is scaled to the operation. For general marginalisation of larger lifecycle blocks, recompute $\Sigma' = (B')^{-1}$ by Cholesky factorisation after the Schur complement, re-establishing exact dual tracking at that point. For low-rank marginalisation with small $r$ — competitive cell exit (`alg:keyspace:cell-exit`) being the case that occurs often — the covariance path may extract $\Sigma_{kk}$ directly. The discrepancy from the regularised low-rank step depends on the offset relative to the removed block's spectrum and on its coupling to the kept block; $\delta$ alone is not an error bound. The ordinary periodic recomputation schedule clears the resulting dual-tracking discrepancy, not the marginalisation bias that produced it.

**Definition (The Schur complement)** · `def:gaussian:schur-complement`

The Schur complement has a precise meaning for this posterior, and it is not merely the algebraic residue of eliminating a block. Information that the removed dimensions carried about the kept dimensions — encoded in the off-diagonal blocks — is transferred to the kept dimensions' precision through the correction term $B_{kr}\,B_{rr}^{-1}\,B_{rk}$.

Under the exact complement, what is learned is therefore not lost with what taught it. Information gained from a removed Sentinel's features about the relationship between its alarm patterns and the identity features survives that Sentinel's deregistration through the change to the surviving precision. The regularised complement attenuates that change, and the fallback discards it.

**Algorithm (Regularised Schur complement)** · `alg:gaussian:regularised-schur`

Before a replenishment floor intervenes, uniform forgetting by $q=\gamma^t$ scales both a removed block and its coupling: $B_{rr}=qA$ and $B_{kr}=qC$. The exact correction then scales as $qC A^{-1}C^\top$, not as a quantity of order one. Once diagonal replenishment intervenes the two blocks need not share even that scaling. The numerical risk is a small or ill-conditioned factorisation and accumulated matrix drift, not a correction that uniform decay makes diverge.

On marginalisation, compute the regularised form:

$$B' = B_{kk} - B_{kr}\,(B_{rr} + \delta I)^{-1}\,B_{rk}$$

with $\delta = \lambda_\text{prior} \cdot \varepsilon_\text{Schur}$, at a default $\varepsilon_\text{Schur} = 10^{-4}$. This bounds the correction's magnitude and keeps the inverted block positive definite.

The offset also changes the answer in exact arithmetic. If $S_0$ is the exact Schur complement and $S_\delta$ the regularised one, then

$$S_\delta-S_0 = \delta B_{kr}B_{rr}^{-1}(B_{rr}+\delta I)^{-1}B_{rk} \succeq 0.$$

Regularisation therefore retains more precision than the exact marginal and produces a narrower covariance. Positive definiteness does not bound this bias: the bias moves the least eigenvalue in the passing direction.

Two guards stand before the correction, and they answer different questions. The upper one is the host's posture. Where the raw condition number $\kappa(B_{rr})$ exceeds a host-declared ceiling, default $10^8$, the block is refused as a source: the deployment has said it will not fold information of that quality into its surviving models, and no arithmetic makes that judgement for it. The lower one is an arithmetic limit a deployment cannot move. Where the regularised block the correction is actually solved against has $\kappa(B_{rr}+\delta I) > 10^{12}$, the solve carries no significant digits and the correction is declined on arithmetic grounds alone. Both set $B' = B_{kk}$, discarding the transferred information rather than computing it from a block that cannot support the computation, and the diagnostics name which of the two refused.

The separation is forced by the offset itself. Regularisation changes the condition number to

$$\kappa(B_{rr}+\delta I) =\frac{\lambda_{\max}(B_{rr})+\delta}{\lambda_{\min}(B_{rr})+\delta},$$

so at the reference prior a raw $10^8$ becomes a regularised $10^4$. A single threshold read before the lift therefore refuses blocks the regularised solve would handle comfortably, and one read after it says nothing about the quality of the information being folded in; the two questions have different answers on the same matrix. Both figures come from one spectrum of $B_{rr}$, since adding $\delta I$ shifts every eigenvalue by $\delta$ and does nothing else.

The condition number here is of the spectrum and not of the diagonal. A ratio of diagonal entries is a lower bound on it and no more: a block of equal diagonal can be arbitrarily ill-conditioned and reports a ratio of one, so a guard reading the ratio refuses far less than it appears to. Where the spectrum cannot be computed the ratio stands in, and what it then certifies is only what a lower bound certifies.

Either path is followed by the verification of (`req:gaussian:positive-definiteness`), and a failure there falls back to $B' = B_{kk}$ likewise. Every path reports what its approximation cost (`req:gaussian:marginalisation-error-reported`).

**Requirement (Positive definiteness of the marginalised precision)** · `req:gaussian:positive-definiteness`

After either path of (`alg:gaussian:regularised-schur`), the marginalised precision $B'$ is verified positive definite before it is adopted, and the verification is a factorisation of $B'$ rather than a reading of its diagonal: a Cholesky attempt on $B' - \phi I$ completes if and only if $\lambda_{\min}(B') > \phi$, where $\phi = 8k\varepsilon_\text{mach}\max(U, 0)$ is the finite-resolution floor taken at an outward-rounded bound $U \geq \lambda_{\max}(B')$ (`const:assayer:schur-verification-resolution`). Where the factorisation does not complete, the correction is discarded and $B'= B_{kk}$ is adopted instead.

The requirement is stated of the eigenvalue and not of the diagonal because the two are not the same test. A positive minimum diagonal entry is necessary for positive definiteness and is not sufficient for it: a matrix with strictly positive diagonal can carry a negative eigenvalue through its off-diagonal structure. In exact arithmetic an SPD parent and positive $\delta$ already give $S_\delta \succeq S_0 \succ 0$; this verification detects numerical error or an invalid parent, not the approximation bias. A diagonal test in this position passes cases the verification is meant to catch.

**Requirement (Verification of the rank-one marginalised precision)** · `req:gaussian:rank-one-verification`

The rank-one path removes a single dimension by forming $\widehat S = B_{kk} - \widehat Q$ for an entrywise-computed $\widehat Q_{ij} = \text{fl}(\text{fl}(cu_i)u_j)$ with $c = 1/(a+\delta)$. It verifies $\widehat S$ before adopting it, at the same finite-resolution boundary the general path uses (`req:gaussian:positive-definiteness`), and the verdict is read from a certificate of $\widehat S$ itself. Two certificates discharge it, and a strictly positive diagonal is neither of them.

The first is an outward-rounded Gershgorin bound. For row sums $r_i \geq \sum_{j \neq i} |\widehat s_{ij}|$ and

$$L = \min_i(\widehat s_{ii} - r_i), \qquad U = \max_i(\widehat s_{ii} + r_i),$$

Gershgorin gives $\lambda_{\min}(\widehat S) \geq L$ and $\lambda_{\max}(\widehat S) \leq U$, so at width $k$ the test

$$L > 8k\varepsilon_\text{mach}\max(U, 0)$$

reaches the same decision the shared verifier would reach (`const:assayer:schur-verification-resolution`). Every rounding in it is directed away from acceptance: the row sums round up, $L$ rounds down, and $U$ and the resolution floor round up. A row sum accumulated in round-to-nearest is not a bound and certifies nothing. The certificate is sufficient and not necessary — a dense positive definite matrix need not be strictly diagonally dominant — so an inconclusive result is not a refusal.

The second is the factorisation, taken exactly as the general path takes it. It decides whenever the first is inconclusive.

A non-positive diagonal entry refuses without either. A diagonal entry is the Rayleigh quotient at a coordinate vector, so $\lambda_{\min} \leq \min_i \widehat s_{ii} \leq 0$ while the resolution floor is never negative. That is the one direction in which the diagonal remains evidence, and it is a proof of refusal rather than a licence to adopt.

Adoption on a positive diagonal alone is forbidden, and the prohibition is not a precaution. The path's exact-arithmetic theorem is sound — for $B \succ 0$ and $\delta \geq 0$ the regularised scalar Schur complement is positive definite — and the path does not compute it. The rounded $\widehat Q$ has neither rank one nor the nullspace of $cuu^\top$, so rank-one interlacing no longer confines the risk to a single eigenvalue. A represented parent that is exactly positive definite, with $a + \delta = 1.00001$ and an exact result whose least eigenvalue is about $+9.62 \times 10^{-19}$, reaches this point as a $\widehat S$ whose minimum diagonal is about $10^{-7}$ and whose least eigenvalue is about $-4.12 \times 10^{-19}$. The diagonal admits it; this requirement refuses it. Neither the positive denominator nor the scalar pivot's condition number is the missing margin, both being premises of the exact proof rather than certificates of the computed result.

A refusal falls back to $B' = B_{kk}$ and reports the whole of the discarded correction (`req:gaussian:marginalisation-error-reported`). Every verification reports which certificate decided it, so the share the cheap certificate settles is measured rather than assumed; that share is what the arrangement's cost advantage rests on, and no analysis supplies it in advance.

**Requirement (The marginalisation error is computed and reported)** · `req:gaussian:marginalisation-error-reported`

Every marginalisation reports the approximation it committed, as a figure computed at that event rather than as a tolerance declared in advance. The quantity is $\text{tr}(E_\delta)$ for $E_\delta = S_\delta - S_0$, the precision the regularisation retained over the exact marginal, reported together with the share $\text{tr}(E_\delta)/\text{tr}(C_0)$ of the exact correction $C_0$ that did not survive. Each figure is labelled as a measurement or as an upper bound, and a bound is never presented as a measurement.

The offset alone cannot be that report. The absolute bound

$$\lVert E_\delta\rVert \leq \frac{\delta\lVert B_{rk}\rVert^2}{a(a+\delta)}, \qquad a=\lambda_{\min}(B_{rr}),$$

contains the removed block's least eigenvalue and the coupling, so a value of $\varepsilon_\text{Schur}$ fixes no error at all until those two are known; and the relative error depends further on $\lambda_{\min}(S_0)$, which nothing in this corpus bounds below. The measurement is available whenever the exact correction is numerically reachable — the solve producing it having more resolution than the difference it is taken across — and the bound above stands in otherwise. That is the ill-conditioned regime, which is precisely where $B_{rr}^{-1}$ is untrustworthy and precisely what a raised posture ceiling admits, so the labelling is not a formality.

A discarded correction reports the whole of itself. As $\delta$ grows without bound the correction vanishes and $S_\delta \to B_{kk}$, so a fallback is this same approximation at its maximum rather than an exact path standing beside it, and the share it reports is one.

The per-event figures fold into a cumulative report across lifecycle events. That report is a monitor and not a bound: the corpus composes no law over repeated marginalisations (`inv:guarantee:structural-exactness`), the events acting on different partitions and separated by observation and forgetting. What it answers is whether a deployment is accumulating approximation at a rate that argues for a recompute, which is a question no single event can be asked.

**Table (Operation costs)** · `tab:gaussian:operation-costs`

Costs of the dimension-changing operations and of the per-observation update, at the current parameter count $p$.

| Operation | Cost | When |
| --- | --- | --- |
| Extension by $r$ dimensions | $O(p \cdot r)$ array growth, plus prior initialisation | Sentinel or outcome-axis registration |
| General marginalisation of $r > 1$ dimensions | $O(p'^2 r + p'r^2 + r^3 + p'^3)$ | Sentinel or outcome-axis deregistration |
| Low-rank marginalisation of $1 + T_5$ dimensions | $O(p^2)$ where the cheap verification certificate carries, $O(p^3)$ where it falls through to the factorisation | Competitive cell exit; rank 2 at the default $T_5 = 1$ |
| Per-observation update | $O(p^2)$ Sherman–Morrison at the current $p$ | Every label |

At the reference configuration (`tab:resource:reference-configuration`), where $p = 638$, and with $r = 68$ — one Sentinel's slot together with its interaction dimensions — the Schur complement costs approximately 45M operations. That total is the sum across the three matrices maintained: roughly 21M for the precision update, with the covariance and mean updates supplying the remainder. At 1 GFlop/s this is approximately 45 ms. General marginalisation then performs the Cholesky recomputation of $\Sigma' = (B')^{-1}$ that restores dual tracking, adding $O(p'^3/3)$ work — about 56 ms at the surviving $p' \approx 570$. Registration and deregistration are deployment events, not per-request events, and these figures are to be read against that.

The low-rank row carries two costs because its verification carries two certificates (`req:gaussian:rank-one-verification`). The quadratic figure is the one the frequent path is meant to pay: forming the correction, subtracting it, accumulating the outward-rounded row sums, and extracting the covariance block are all $O(p^2)$. The cubic figure is what an inconclusive certificate costs: a Cholesky factorisation at the surviving width, $O(p^3/3)$ and so the same work as the covariance recomputation named above, of the order of tens of milliseconds at reference width. Which figure a deployment actually pays is not derivable here — strict diagonal dominance is a property of the precision matrices it produces, not of the algorithm — which is why the share is reported per event rather than asserted.

**Theorem (The two-level posterior guarantee)** · `thm:gaussian:two-level-guarantee`

The posterior carries two guarantees at two levels, and the distinction between them is the chapter's central result.

At the level of lifecycle events, extension produces a joint posterior whose marginal equals the pre-extension posterior. The unregularised Schur identity produces the exact marginal over the surviving dimensions. Marginalisation instead uses the declared regularised approximation and may fall back to the kept block (`alg:gaussian:regularised-schur`), so neither unconditional exactness nor zero accumulated approximation is promised (`inv:guarantee:structural-exactness`).

At the level of individual observations, each update is exact given the current posterior as prior (`inv:guarantee:per-observation-exactness`), with an observation precision determined by the importance weight, the ceiling, and the leverage bound. Forgetting and importance weighting are fixed, data-independent components of the observation model and are well-defined generative parameters. The leverage bound is not: the effective precision depends on the current posterior through the observation's leverage, so the sequence of observation models is self-referential — the model generating observation $n$ depends on what was learned from its predecessors.

The per-observation statement and the lifecycle statement have different qualifications. Every extension and every unregularised marginalisation obeys its exact identity; a regularised or fallback marginalisation is a deterministic approximation. Every per-observation update remains exactly the Bayesian update it claims to be. What does not follow, and what is stated separately below, is that their composite is the posterior of any one fixed model.

**Caveat (Departure from the fixed-model reading)** · `cav:gaussian:fixed-model-departure`

The posterior is not the posterior of any fixed generative model. No single model, stated before observing the data, produces this posterior when updated sequentially over the observations, because the leverage bound makes each step's observation precision depend on what the previous steps produced.

What the posterior is remains well defined: the exact result of a determined recursive computation, in which each step applies a Bayesian update with a specific observation precision to the previous posterior. The resulting Gaussian has a mean — a weighted combination of all observations under exponential discounting and class-balanced importance — and a covariance reflecting the accumulated information, widened in the directions where the leverage bound has fired. Both are useful for their intended purposes.

The consequence to disclose is that standard Bayesian calibration guarantees — credible-interval coverage, posterior predictive checks — do not apply directly to this posterior. Platt calibration (`def:platt:regimes`) supplies an empirical calibration that absorbs the leverage bound's effect on point estimates. It does not absorb its effect on uncertainty estimates, and nothing in the system claims otherwise.

**Proposition (The departure is conservative)** · `prop:gaussian:conservatism`

The leverage bound's effect on uncertainty is always conservative: the posterior is wider than the fixed-model posterior would be, never narrower, because the bound discards precision rather than adding it. The system therefore reports more uncertainty than a fixed-model reading would, in every direction and at every step.

The magnitude varies with how often the bound fires. In directions where it fires frequently — high-importance-weight observations in uncertain directions — the posterior may be substantially wider. In directions where it rarely fires — well-populated features in steady state — the posterior is indistinguishable from the fixed-model posterior.

Every downstream consumer of uncertainty inherits the conservatism: the blend weight, the crossover intervals, decision fragility. Because the direction is guaranteed and the magnitude is not, the magnitude is measured rather than assumed: the empirical coverage diagnostic (`alg:monitoring:empirical-coverage`) reports the realised inflation factor, so that a host can read how conservative the reported uncertainty currently is instead of inferring it from this proposition.

**Data (Binding frequency)** · `data:gaussian:binding-frequency`

Measured rates at which the leverage bound actually fires. In steady state it fires on approximately 2–8% of positive-valence labels, the amplification coming from the importance weight, and on less than 0.1% of negative-valence labels. During early convergence it fires on 30–60% of all labels.

The distribution is what makes the departure of (`cav:gaussian:fixed-model-departure`) tolerable in practice. It is concentrated in the convergence transient and in rare feature directions; after convergence with well-populated features, the departure is negligible for the point estimate and modest for the uncertainty. These are empirical figures, not guarantees, and a deployment whose label mix differs materially from the one they were measured under should expect different ones.

**Algorithm (Synchronisation monitoring)** · `alg:gaussian:synchronisation-monitor`

The dual maintenance of $B$ and $\Sigma$ accumulates floating-point error between periodic Cholesky recomputations. The dominant conditioning risk is not the per-step amplification from forgetting — modest even over a thousand steps — but the condition number of $B$ itself, which grows when feature directions receive no observations while forgetting erodes their precision exponentially. For a feature appearing in a fraction $f$ of labels the steady-state precision is approximately $f \cdot \bar{w} / (1 - \gamma)$, and $\kappa(B)$ ranges from order $10$ where every feature is well populated to $10^5$ and beyond for rare interactions between seldom co-active Sentinels.

The reading before a recomputation is taken whether or not the recomputation happens, and it is what decides whether it does. At each visit to a model on the cadence's interval — the cheap arms go to the recomputation directly (`alg:gaussian:condition-adaptive-recompute`) — compute the synchronisation error (`def:monitoring:synchronisation-error`) between the maintained covariance and the precision matrix the model holds, separate it into its two components, and read each against the threshold. Both under it ends the visit: nothing is recomputed, the reading is recorded, and the cadence is told that the visit found nothing to do. Either over it proceeds to the recomputation, which then takes the second reading, between the recomputed covariance and the precision matrix the model will hold, after it. The two differ in what follows: a residual over the threshold is drift the recomputation removes and the interval shortens for, while a prior-induced component over it is the replenishment clamp's contribution, which the recomputation absorbs and which no interval reduces. The recomputation is where the spectral floor is applied (`dec:posterior:spectral-floor`), so those two matrices differ by the floor's own addition and by nothing else, and both readings are against a matrix the model actually holds — before and after respectively. What is excluded is a reading against a matrix the model will never hold: a factorisation that shifts internally and reports its own answer against the shifted copy certifies only that its derivation was consistent. The quantity is the Frobenius norm of the departure of a product from the identity, as its definition states; this chapter cites that definition rather than restating it, so that the monitored quantity and the reported quantity cannot drift apart.

The second reading decides adoption. The recomputed covariance replaces the maintained one only where the reading after is no worse than the reading before and is below the threshold, and otherwise the maintained covariance is kept and the interval goes to its floor. What that failure means follows from when the recomputation ran: it ran because the first reading found real drift, so a recomputed pair that is no better is a model whose drift is real and whose fresh inverse is worse than the pair it holds — an alarm rather than a routine decline (`dec:posterior:measured-adoption`), (`dec:posterior:adaptive-cadence`). The precision matrix and the covariance are adopted together or not at all, because the floor the recomputation applied belongs to both. Adoption on the measurement rather than on the factorisation's verdict is what separates a covariance that is the inverse of the matrix the model will hold from one that is the inverse of some other matrix: a factorisation that succeeded under a further internal shift reports success and returns that matrix's inverse, and the reading after says so. Accepting one reading equal to the other rather than requiring a strict improvement is deliberate — a recomputation of a matrix that has not moved reproduces the covariance already held, and that is not a worse answer.

At each visit, separate the reading into the part the prior put there and the part the arithmetic did, and report all three figures — the measured reading, the prior-induced component, and the residual between them floored at zero — in the health snapshot, together with the resolution the reading carries and whether the residual stood at or below it (`def:monitoring:synchronisation-error`). Where either component exceeds the threshold, recompute; where the residual was the one that exceeded it and the recomputation is adopted, shorten the interval. The separation is what makes the shortening mean something: the prior-induced component is the replenishment clamp's contribution since the last adopted recomputation seen through the covariance, it is not drift, and a recomputation cannot reduce it, so a cadence reacting to the whole reading shortens for ever on a model whose coordinates rest at the floor. The alternative repair is to move the clamp to the recomputation as the spectral floor is, which would make the pair consistent by construction; it is refused because the clamp is required after each forgetting step (`req:gaussian:prior-replenishment-floor`) and deferring it lets a coordinate decay below the floor within an interval. The monitor exists to make the drift visible before it is large enough to matter, so a monitored value that never reaches a report is a monitor that has not been built.

**Algorithm (Condition-adaptive recomputation)** · `alg:gaussian:condition-adaptive-recompute`

Rather than recomputing on a fixed interval, let the conditioning set the interval: recompute when the labels absorbed reach the interval the last recomputation's outcome fixed, or before that when the matrix has moved away from the state that recomputation certified. A fixed interval is either too frequent for a well-conditioned model or too rare for an ill-conditioned one, and which of the two a deployment has is a property of its feature occupancy rather than of its schedule.

What the per-label test fires is a measurement, and the recomputation follows only where that measurement finds drift worth acting on (`dec:posterior:recomputation-trigger`). The test is $O(p)$ and every part of it is relative to the last visit, so that a visit — measurement or recomputation alike — resets everything the test can fire on. Resetting on the measurement is what keeps the arms usable now that most visits stop there: an arm relative to a record only a recomputation could write would fire on every label of a model whose measurements keep finding it healthy, which is the same trap in a new place. The cheap diagonal ratio (`def:monitoring:condition-number`) is compared against the ratio recorded at that recomputation, and growth past it by a factor $\kappa_\text{growth}$, default 2, calls for another; the count of dimensions at the replenishment floor is compared against the count recorded there (`req:monitoring:replenishment-floor`), and a change calls for another, because the evidence structure the recomputation's spectrum was read from has moved. A diagonal entry that is not finite or not positive recomputes immediately, which is the one absolute reading and is absolute because it is not a matter of degree.

A test against a fixed threshold is what this replaces, and it is unusable for a structural reason rather than a tuning one. The recomputation rebuilds the covariance from the precision matrix and does not alter the precision matrix, so any test of a property of that matrix against a constant, once satisfied, stays satisfied through every recomputation it calls for. With the replenishment floor pinning the ratio's denominator, that state is reached as soon as one dimension rests at the floor, and the engine then recomputes on every label for the rest of the deployment's life. Substituting the true condition number for the ratio changes nothing: the trap is the fixed threshold on an unchanged quantity, not the reading's crudeness.

The true condition number is read at the recomputation, where the decomposition it needs is already being paid for, and it is the reported conditioning of the model. The cheap ratio is a trigger input and a published lower bound, never the reported conditioning (`req:monitoring:conditioning-bound-named`).

**Requirement (Prior replenishment floor)** · `req:gaussian:prior-replenishment-floor`

After each forgetting step, clamp each diagonal entry of the precision matrix at a floor: $B_{jj} \geq \lambda_\text{floor}$, at a default $\lambda_\text{floor} = 0.01 \cdot \lambda_\text{prior}$. This arrests coordinatewise diagonal decay. It does not bound $\kappa(B)$: equal positive diagonal entries can coexist with an arbitrarily small eigenvalue through the off-diagonal structure.

The floor injects a small amount of artificial diagonal precision, and the injection is deliberate rather than tolerated. It is equivalent to a weak coordinatewise prior that does not fully decay: a feature coordinate included in the model represents a structural prior belief that it might matter, and no amount of elapsed time without observations is evidence that it does not. A spectral conditioning guarantee still requires a spectral guard.

## Part (The Data Model) · `part:spec:data-model`

Part II defines what the Core sees. Three registries govern the shape of the feature vector, each Sentinel's report is compressed to a fixed width, a key space is partitioned into ranges the models can weigh, the host's own signals are declared, and the blocks are assembled into one vector with a map over it. Nothing here references actions, channels, posture, or cost.

### Chapter (Registries and Lifecycle) · `chap:spec:registries-and-lifecycle`

The chapter makes the dynamic-registry assumption (`assum:constraint:dynamic-registries`) operational. Three registries — Sentinels, outcome axes, identity dimensions — each admit runtime addition and removal, and each discharges the change through the Gaussian algebra rather than through retraining. The chapter states what is declared at registration, what each lifecycle event does to which model set, and what happens when several events arrive together.

#### The Sentinel registry · `sec:registry:sentinel`

The division carries no material of its own. It collects the registration record, the lifecycle operations, the two protocols, and the coverage, hibernation and scale statements that belong to Sentinels.

**Schema (Sentinel registration record)** · `schema:registry:sentinel-record`

A Sentinel is registered with five fields: an identifier, a name, a description, a declared domain width, and its coordinate semantics.

| Field | Carries |
| --- | --- |
| Identifier | The registry key, assigned by the host and unique across the registry |
| Name | The identifier used in declared cross-Sentinel interaction templates; unique across currently registered Sentinels |
| Description | What this Sentinel watches, in the host's own words |
| Declared domain width | The width $N_s$ of the Sentinel's native coordinate space $[0, 2^{N_s})$ (`def:encoding:domain-width`) |
| Coordinate semantics | What the coordinate space is, how it is encoded, and whether hierarchical structure is asserted |

The coordinate semantics carry three things in turn: a description of the domain, an encoding description — native hierarchical with the meaning of depth stated, a space-filling curve with its type and source dimensions (`tab:encoding:curve-guidance`), a categorical encoding with its taxonomy, or some other encoding the host describes — and the host's assertion that shared prefixes imply shared context.

The Core records the coordinate semantics and does not act on them. It does not refuse a registration that asserts no hierarchical structure, and it does not alter extraction or model weights on the strength of an encoding description. The three declarations that carry the encoding contract — description, declared width, and coordinate semantics — exist for the three reasons the contract gives (`req:encoding:host-contract`): to force engagement with the encoding question before the Sentinel exists, to make the deployment's assumptions auditable by someone who did not make them, and to give the diagnosis of an uninformative Sentinel somewhere to start.

**Table (Sentinel lifecycle operations)** · `tab:registry:sentinel-operations`

Three operations against the four surfaces a Sentinel touches. Full feature-space models are the operational model, the sister model, and every active outcome-axis prediction model; the fixed-dimension anchor model is neither extended nor marginalised by a Sentinel event.

| Operation | Interpreter | Estimator | Outcome memory | Dimension map |
| --- | --- | --- | --- | --- |
| Add | Create the extractor and the routing indices | Extend every full feature-space model by $r$ dimensions (`thm:gaussian:extension`) | Create the spatial outcome map | Rebuild with a contiguous slot and the instantiated interactions |
| Remove | Destroy the extractor and the routing indices | Marginalise every full feature-space model by $r$ dimensions (`thm:gaussian:marginalisation`) | Drop the spatial map | Rebuild after removing the owned ranges |
| Offline | No extraction this cycle | Occupancy zero, features zero | No update | Unchanged |

Lifecycle operations publish atomically with respect to assessment and label calls: an in-flight assessment reads pre-event state, and one starting after publication reads post-event state. Where marginalisation and time-indexed decay fall in the same call, marginalisation runs first, which preserves the wider removed block for the Schur computation (`alg:gaussian:regularised-schur`).

**Remark (Anomaly dilution)** · `rem:registry:anomaly-dilution`

Registering a Sentinel dilutes established anomaly profiles rather than distorting them. An entity whose other Sentinels have been alarming sees its aggregate anomaly features fall in proportion to one over the number of reporting Sentinels, because the new Sentinel contributes nothing until it has observed anything.

The dilution is conservative — it lowers risk estimates rather than raising them — and it is self-correcting within roughly twenty assessments per entity, as the new Sentinel's own features begin to carry signal. No compensating mechanism is needed, and one would have to be undone later.

**Algorithm (Sentinel registration)** · `alg:registry:sentinel-registration`

On registration, in order:

1. **Validate the declared width.** Reject a declaration of zero, and reject one greater than the internal width (`dec:encoding:fixed-internal-width`). This is the only structural validation the Core performs on a registration; everything else the record declares is an assertion the Core records and does not check (`rem:encoding:unchecked-width`).
2. **Allocate the slot.** Assign the next contiguous block of $q + 1$ indices, the occupancy indicator and the extraction width (`def:extraction:slot`).
3. **Instantiate interactions.** For each applicable template, create the features and assign their indices.
4. **Create the routing indices.** Allocate the per-Sentinel current report index and the spatial outcome index. The declared width is recorded alongside them: it is required to interpret reported cell depths, and it fixes the shift at which the integration layer lifts native coordinates into the internal representation (`alg:encoding:coordinate-lift`).
5. **Extend every full feature-space model.** Extend the mean with zeros and the precision and covariance with the prior at the new block, with zero off-diagonals (`thm:gaussian:extension`).
6. **Extend standardisation.** Append entries at the feature-class priors.
7. **Update the dimension map.**

**Algorithm (Sentinel deregistration)** · `alg:registry:sentinel-deregistration`

On deregistration, in order:

1. **Identify the dimensions to remove.** The Sentinel's slot together with every interaction feature involving it.
2. **Marginalise every full feature-space model.** Apply the regularised Schur complement (`alg:gaussian:regularised-schur`) to the operational model, the sister model, and every active outcome-axis model.
3. **Remove the standardisation entries.**
4. **Destroy the routing indices.** The spatial outcome index's cell state ends with the Sentinel; hibernation keeps the parameter block and never the outcome memory (`alg:registry:hibernation`).
5. **Update the dimension map.**

Deregistration removes the same structure whatever the Sentinel's history (`thm:gaussian:two-level-guarantee`). The exact Schur identity transfers the whole correction into the surviving precision; the specified regularisation attenuates it, and a kept-block fallback discards it.

**Table (Coverage states)** · `tab:registry:coverage-states`

Not every Sentinel produces a report for every observation. A timing Sentinel has nothing until the second observation in a session; a categorical Sentinel has nothing for an observation lacking the attribute. The Core handles this structurally rather than by imputation, through four states.

| State | Occupancy | Features | Meaning |
| --- | --- | --- | --- |
| Not registered | Dimensions absent | — | The source does not exist |
| Active, no batch report | 0 | Zero | The source is offline this cycle |
| Active, no data for the request | 1 | Zero | The source is active but silent for this observation |
| Active, reporting | 1 | Populated | The source is reporting |

The occupancy indicator is what separates the second state from the third. Both present zero features, and the model must be able to tell "this source said nothing" from "this source is not there", because the two license different inferences from the same numbers.

**Algorithm (Hibernation)** · `alg:registry:hibernation`

Hibernation is an optional lifecycle state. On deregistration, the Sentinel's parameter block — its sub-vectors of the mean, the relevant rows and columns of the precision and covariance, and its standardisation entries — may be stored against its identifier instead of being dropped. Where the same Sentinel re-registers before the store expires, the block is decayed by the time-indexed rate over the elapsed wall-clock interval and by the label-indexed rate over the labels processed meanwhile, and the models are re-extended with the decayed block rather than with the prior.

Hibernation preserves the Sentinel's self-structure: the parameter relationships among its own $q + 1$ features. Cross-feature relationships — how this Sentinel's features correlated with identity features or with another Sentinel's — are not preserved, because extension always writes zero off-diagonal blocks. Those must be relearned. Nor is the outcome memory preserved: the store holds parameters and standardisation entries, and a cell's accumulated averages are decay-only state that ends with the entity that fed them. The marginalisation path of (`alg:registry:sentinel-deregistration`) remains the fallback wherever the stored block has been reclaimed or the Sentinel's configuration has changed.

**Definition (No online Sentinel)** · `def:registry:no-online-sentinel`

Where no Sentinel has data for a request, every Sentinel slot and every aggregate feature is zero. Roughly forty per cent of the feature vector remains nonzero — the identity block, the signal block, and the bias — and the posterior reflects the rest correctly as high uncertainty rather than as evidence of normality.

The condition is reported rather than inferred. The assessment carries a flag saying that no Sentinel was online, so a host reading a low risk estimate can tell an estimate made from measurement from one made in its absence.

**Table (Sentinel count and label rate)** · `tab:registry:sentinel-scale`

The architecture imposes no ceiling on the number of registered Sentinels. The practical limit is convergence: each Sentinel adds dimensions, and dimensions need labels. Comfortable counts by label rate:

| Label rate | Comfortable count | Rationale |
| --- | --- | --- |
| 1,000 per day | 15–20 | Convergence within two to three days |
| 200 per day | 8–12 | Convergence within one to two weeks |
| 50 per day | 3–6 | Convergence within two to four weeks |

The figures are guidance rather than limits. A deployment that registers more Sentinels than its label rate supports is not in error; it is slow, and the slowness is visible in the posterior's uncertainty rather than hidden in it.

#### The outcome axis registry · `sec:registry:outcome-axis`

The division carries no material of its own. It collects what an outcome axis is, the registration record, the lifecycle operations and protocols, partial reporting, scaling, hibernation, and the spatial-feature policy.

**Definition (Outcome axis)** · `def:registry:outcome-axis`

An outcome axis is a secondary outcome quantity the host observes at label time and declares to the Core: financial loss, investigation depth, an external confidence score, a compliance metric, any domain scalar. Each registered axis receives its own Bayesian prediction model at the current full dimension, and each axis's outcome history feeds back as features into the risk models and into every other axis's model.

The Core does not interpret any axis (`assum:constraint:outcome-axes`). It imposes no assumption about what an axis means, which direction of it is good, or how a prediction should influence a decision. It predicts, reports calibrated uncertainty, and says nothing further. This is the measure-not-decide principle (`prin:principle:measure-not-decide`) applied to a quantity the Core has no way to understand: interpreting an axis would require domain knowledge the Core does not have and cannot acquire from the axis's values.

**Schema (Outcome axis registration record)** · `schema:registry:axis-record`

An axis is registered with an identifier, a name unique across currently registered axes, a description, a training-eligibility mode, an initial compression scale, a forgetting rate, and a spatial-feature policy.

| Field | Carries |
| --- | --- |
| Identifier | The registry key |
| Name | Unique across currently registered axes |
| Description | What the axis measures, in the host's own words |
| Eligibility | Train on unconfounded labels only, as the sister model does, or on all labels |
| Initial compression scale | The scale at which the axis's values are compressed into a bounded training target |
| Forgetting rate | The axis model's label-indexed forgetting rate |
| Spatial-feature policy | Whether per-Sentinel spatial outcome features are carried for this axis (`disc:registry:spatial-policy`) |

Axes are maintained in registration order, and an axis's zero-indexed position in that order is its index. The dimension map uses that index, and no other name, to locate the axis's features within each identity dimension's block, so the registration order is part of the record rather than an artefact of it.

**Table (Outcome axis lifecycle operations)** · `tab:registry:axis-operations`

Three operations across the three surfaces an axis touches. Full feature-space models here are the operational model, the sister model, and every *other* active axis model; the anchor is untouched.

| Operation | Interpreter | Estimator | Outcome memory |
| --- | --- | --- | --- |
| Add | Add per-axis features to every registered identity dimension, and to every Sentinel extractor where the spatial policy is enabled | Extend the full feature-space models by $r_a$ dimensions and create the axis model | Add per-axis running averages to every cell, where spatial |
| Remove | Remove the axis's features from the extractors | Marginalise the surviving full feature-space models by $r_a$ dimensions and destroy the axis model | Drop the per-axis averages, where spatial |
| Value absent for a label | — | The axis model does not update; its existing features are unchanged | No update for that axis |

**Algorithm (Outcome axis registration)** · `alg:registry:axis-registration`

On registration of an axis, in order:

1. **Compute the new dimensions.** Where the spatial policy is enabled, each active Sentinel gains two outcome-memory features for the axis; where it is disabled, none. Each registered identity dimension gains three. The count is $r_a = 2n \cdot \mathbb{1}[\text{spatial}] + 3D$ for $n$ Sentinels and $D$ identity dimensions, plus any declared interaction features.
2. **Extend every full feature-space model** by $r_a$ dimensions, with the prior at the new block and zero off-diagonals (`thm:gaussian:extension`).
3. **Create the axis prediction model** at the full dimension after extension, at the prior.
4. **Initialise per-axis cell state** in the identity layer always, and in the spatial outcome memory where the policy is enabled. Every new running average starts at zero.
5. **Extend standardisation.** Append entries at the feature-class priors.
6. **Initialise the compression scale** from the record.
7. **Update the dimension map.**

Every vector the map governs is extended by the same $r_a$ at the same event, whatever the identity dimension count. The map, the model parameters, and the standardisation statistics describe one dimension at every published version (`inv:dimension:version-consistency`), and a registration that extended some of them and not others would leave the three disagreeing with nothing to detect it.

**Algorithm (Outcome axis deregistration)** · `alg:registry:axis-deregistration`

On deregistration of an axis, in order:

1. **Identify the dimensions to remove.** The axis's three features in each registered identity dimension, its two outcome-memory features per Sentinel where the spatial policy was enabled, and any interaction features.
2. **Marginalise every surviving full feature-space model** by the regularised Schur complement (`alg:gaussian:regularised-schur`).
3. **Destroy the axis prediction model.**
4. **Drop the per-cell running averages.** The axis's rows leave every outcome-memory entry together with the features they fed, and nothing is kept for a later return.
5. **Remove the standardisation entries.**
6. **Update the dimension map.**

The averages end with the axis, and hibernation does not reach them: what an archive holds is the entity's own parameter block and its standardisation entries, not the outcome memory (`rem:registry:axis-hibernation`). This is a position rather than an omission. An average that survived the gap would describe a feature population and a standardisation regime that no longer exists once the marginalisation has run — the Schur complement folds the axis's block into the survivors, and step 5 removes the standardisation entries the average was accumulated against — so restoring it would mean reading a number against a vector it was never measured in. A re-registered axis therefore starts cold by design, at the zeroed cell state its registration creates (`alg:registry:axis-registration`), and the warm-up stages are what carry that condition (`tab:warmup:stages`). The cost of the alternative is larger than the cold start it would save: an axis identifier is reusable, and a surviving average would let a new axis read its predecessor's history as its own warm start.

**Requirement (Partial outcome reporting)** · `req:registry:partial-outcome-reporting`

Not every label carries a value for every axis. Where an axis's value is absent from a label, three things follow and no others: that axis's prediction model does not update, that axis's per-cell running averages do not update, and that axis's existing features remain in the vector at their current values.

No information is lost by the omission. The other models continue to see whatever outcome history the axis has accumulated, and the axis model simply learns nothing from a label that told it nothing. The absent value is not imputed, and a label reporting no axis values at all is a valid label for every model except the axis models.

**Table (Outcome axis scaling)** · `tab:registry:axis-scaling`

Each axis adds features and one model. The marginal feature count is $2n \cdot \mathbb{1}[\text{spatial}] + 3D$; at eight Sentinels and two identity dimensions:

| Spatial policy | Features per axis | Per-label cost per axis |
| --- | --- | --- |
| Enabled | $2n + 3D = 22$ | $O(p^2)$ |
| Disabled | $3D = 6$ | $O(p^2)$ |

The per-label cost of an axis model is quadratic in the dimension regardless of the policy, so the count of models rather than the growth of the dimension is the dominant driver. The policy reduces the dimension the cost is quadratic in:

| Axes | Policy | $\Delta p$ | $p$ | Per-label operations | Against no axis |
| --- | --- | --- | --- | --- | --- |
| 0 | — | 0 | 616 | 0.76M | 1.00× |
| 1 | enabled | 22 | 638 | 1.22M | 1.61× |
| 3 | all enabled | 66 | 682 | 2.33M | 3.07× |
| 5 | all enabled | 110 | 726 | 3.69M | 4.86× |
| 5 | all disabled | 30 | 646 | 2.92M | 3.85× |
| 10 | all enabled | 220 | 836 | 8.38M | 11.05× |
| 10 | all disabled | 60 | 676 | 5.48M | 7.22× |

There is no architectural ceiling on the axis count. The binding constraint is convergence: each axis model needs labels that both are eligible and carry the axis's value, and that is a scarcer thing than a label.

**Remark (Outcome axis hibernation)** · `rem:registry:axis-hibernation`

An axis hibernates under the protocol Sentinels hibernate under (`alg:registry:hibernation`), with the same preservation scope: the axis's own parameter block is decayed by elapsed time and by intervening labels and re-extended on re-registration, and its cross-feature relationships are relearned. The scope being the same is also what settles the outcome memory: the per-cell running averages dropped at deregistration (`alg:registry:axis-deregistration`) are not among what returns, and a hibernating axis comes back to a zeroed cell state exactly as a fresh one does. Nothing about an axis changes the protocol, which is why it is stated once and referred to here rather than written twice.

**Discussion (When to enable spatial features)** · `disc:registry:spatial-policy`

The spatial policy governs whether the axis carries two per-Sentinel features drawn from the spatial outcome memory (`tab:extraction:ledger-features`). Those features say: requests routed through this cell in this Sentinel have tended to produce axis values of this magnitude.

Enable the policy where the axis value correlates with where the request falls in a Sentinel's coordinate space — financial loss, whose patterns concentrate spatially; response latency, which is network-dependent; jurisdiction-dependent policy metrics. Disable it where the value depends on the entity rather than the position — investigation depth, an external confidence score, a satisfaction measure. When uncertain, enable: the model learns to downweight an uninformative feature, and the cost of carrying one is dimensions rather than error.

Disabling is an optimisation for deployments carrying many axes, where the dimension saving is material. It has a second consequence worth stating: an axis registered without spatial features holds no per-cell state in the outcome memory, and is therefore outside the contamination pathway that reaches spatially-enabled axes (`disc:valence:axis-pathway`).

#### The identity dimension registry · `sec:registry:identity-dimension`

The division carries no material of its own. It collects what an identity dimension is and the two lifecycle protocols; the layer itself is the subject of (`chap:spec:key-space-identity`).

**Definition (Identity dimension)** · `def:registry:identity-dimension`

An identity dimension is a host-declared structured key space over which the Core tracks identity: an organisational hierarchy, a tenant space, a geographic prefix space, a device fingerprint hierarchy. Each registered dimension gets its own graph instance over the declared integer domain, from which the layer selects competitive cells and reports range-membership indicators as features (`mot:keyspace:purpose`).

The registry manages dimensions through the algebra that governs Sentinels and axes, so a dimension may be registered or deregistered at runtime without retraining and without disturbing what the models have learned about anything else.

**Algorithm (Identity dimension registration)** · `alg:registry:dimension-registration`

On registration of a dimension, in order:

1. **Create the graph instance** at the declared budget parameters.
2. **Determine the initial competitive set**, which is empty until observations accumulate and cells promote.
3. **Extend every model but the anchor** by the dimension's competitive indicators — initially none — and by its per-dimension feature block. Where this is the first registered dimension, extend also by the cross-dimension aggregate block (`tab:keyspace:cross-dimension-features`). Then recompute the anchor's projection indices during the map rebuild, so that the anchor's identity features resolve to the current positions of the cross-dimension block: at the first dimension they change from absent to present, and otherwise they may still shift, because the cross-dimension block follows every per-dimension block.
4. **Extend standardisation.** Append entries at the feature-class priors.
5. **Update the dimension map.**

**Algorithm (Identity dimension deregistration)** · `alg:registry:dimension-deregistration`

On deregistration of a dimension, in order:

1. **Identify the dimensions to remove.** Every competitive indicator of the dimension, its per-dimension feature block, and every competitive interaction feature involving it. Where this was the last registered dimension, remove the cross-dimension aggregate block as well.
2. **Gather the removed partition and marginalise every model but the anchor.** The pre-event map supplies the per-dimension range, the dimension's indicator sub-range, and the interaction indices. These are not contiguous in the feature vector, so they are gathered into one removed partition (`def:gaussian:gathered-partition`) and discharged by a single regularised Schur complement per model (`alg:gaussian:regularised-schur`), after which the survivors are compacted into the post-event layout.
3. **Recompute the anchor's projection indices** during the map rebuild. The anchor's identity features become absent where no dimension remains, and otherwise resolve to the shifted cross-dimension positions.
4. **Remove the standardisation entries.**
5. **Destroy the graph instance.**
6. **Update the dimension map.**

The gather is what makes the removal one operation rather than several. A dimension owns three disjoint stretches of the vector, and marginalising them separately would apply three Schur complements where one is exact (`def:gaussian:schur-complement`).

#### Lifecycle interaction · `sec:registry:lifecycle-interaction`

The division carries no material of its own. It collects the serialisation invariant across the three registries and the treatment of compound events.

**Invariant (Model sets under lifecycle events)** · `inv:registry:model-set-serialisation`

Every registry event touches a determinate set of models, and the sets are stated once here. Adding or removing a Sentinel extends or marginalises the operational model, the sister model, and every outcome-axis model. Adding or removing an axis extends or marginalises the operational model, the sister model, and every *other* axis model. Adding or removing an identity dimension extends or marginalises every full feature-space model. The anchor model is in none of these sets: it is fixed-dimensional, and identity and axis events reach it only through its projection indices.

All three registries publish through one protocol, and lifecycle changes are never visible mid-call. Each assessment reads one self-consistent published version, so no assessment can observe a model set part-way through an event — which is what allows the sets above to be stated as facts about the published state rather than as facts about an instant.

**Requirement (Compound events)** · `req:registry:compound-events`

Where several lifecycle events must occur together — removing a Sentinel while registering an identity dimension, say — they are processed sequentially as state updates, each seeing the state the previous one left.

The order does not affect the final posterior, because extension and marginalisation commute on disjoint index sets — extension (`thm:gaussian:extension`) adds a block the other's removed partition does not meet, and marginalisation (`thm:gaussian:marginalisation`) removes one the other's extension did not touch. Order does affect the intermediate work: marginalisation should precede extension where both apply to the same model, so that the extension's cost is paid at the smaller dimension rather than the larger. The commutation is a property of the algebra and not of the implementation, and an implementation that processes compound events in some other order owes the same final posterior.

### Chapter (Per-Sentinel Structured Extraction) · `chap:spec:structured-extraction`

The chapter fixes what the Core takes from one Sentinel's batch report. Every width here is a width the extraction produces, in the order stated. The extraction is identical for every Sentinel and deterministic given a report.

**Setup (From batch report to fixed width)** · `setup:extraction:from-batch-report`

A batch report offers, for each reported competitive cell that processed observations, scores at every level of the cell's ancestor chain, across the four scoring axes, together with coordination context and a structural summary of the report as a whole (`tab:architecture:sentinel-properties`). That is a variable-length structure: it grows with the chain's depth, with the number of reported cells, and with the coordination tier's occupancy.

The extraction compresses it to a fixed-width vector per Sentinel, preserving per-axis identity, multi-scale structure, coordination signal, and outcome history — the identity-preservation principle (`prin:principle:identity-preservation`) applied at the point where variable length has to become fixed width. Six groups are appended in a fixed order: chain z-scores, chain CUSUMs, chain structure, coordination, outcome-memory features, and batch context. The order is part of the contract, because every offset downstream is derived from it.

**Table (Chain z-scores)** · `tab:extraction:chain-z-scores`

Six complementary views over the per-axis maximum z-scores along the ancestor chain, four axes each, twenty-four features. The per-axis maxima are the maximum-z-score field the Sentinel algorithm document publishes at its section 14.4 — the z-score of the per-sample maximum, not that of the batch mean, and the two are separate fields of the report. The distinction preserves sensitivity to an isolated extreme: in a cell taking ten thousand observations in a batch, one anomalous observation still produces a high maximum z-score, where the batch mean is diluted by the rest.

| View | What it captures | Width |
| --- | --- | --- |
| Cell | Local anomaly, at the receiving cell | 4 |
| Root | Global anomaly, at the whole domain | 4 |
| Max | The worst anomaly at any scale, element-wise across levels | 4 |
| Mean | The average across scales — sustained multi-scale signal | 4 |
| Gradient | Cell minus root — positive means the anomaly is localised | 4 |
| Spread | The standard deviation across levels — one loud level against a sustained one | 4 |

Per-axis identity survives all six views: the novelty column tracks novelty at every view and never mixes with displacement. The mean and spread views carry depth-dependent statistical properties — a mean of two from a three-level chain is noisier than the same value from a seven-level chain — and the chain-length feature of (`tab:extraction:chain-structure`) is what lets the model disambiguate them.

**Remark (The maximum view's order-statistic bias)** · `rem:extraction:order-statistic-bias`

The max view is biased upward under the null. The expected maximum of $d$ independent draws grows as $O(\sqrt{\ln d})$, so a deep chain produces a larger maximum than a shallow one from identical per-level distributions, and the difference is the chain's depth rather than the data's anomaly.

The chain-length feature is supplied in the same block precisely so that the model has the information to learn the correction, rather than the extraction applying one. A deployment needing precise null-hypothesis calibration of the max view should apply an extreme-value correction at the point of use; the extraction does not, because a correction applied here would be applied to every deployment on the strength of an assumption only some of them make.

**Remark (The optional batch-mean extension)** · `rem:extraction:batch-mean-extension`

Where the distinction between one loud observation and a sustained batch-level elevation is diagnostically important, a deployment may add the per-axis batch-mean z-scores at the cell level as four further features per Sentinel. At eight Sentinels this is thirty-two features, roughly a five per cent increase in the dimension.

It is a configuration option and not a default. The default extraction reads the maximum because that is the view that survives dilution, and a deployment that wants both is paying dimensions for a second view of the same axis — a trade worth making only where the two have been observed to disagree.

**Table (Chain CUSUMs)** · `tab:extraction:chain-cusums`

Three views over the per-axis cumulative-sum statistics along the chain, four axes each, twelve features. Where the z-scores say how far the current observation departs, the CUSUMs say how long a departure has been accumulating.

| View | Width |
| --- | --- |
| Cell CUSUM, four axes | 4 |
| Root CUSUM, four axes | 4 |
| Max CUSUM, element-wise across levels, four axes | 4 |

**Table (Chain structure)** · `tab:extraction:chain-structure`

Eight features describing the shape of the chain rather than its scores. Two of them are read from the Sentinel's maturity record and not computed by the Core, as the Sentinel algorithm document's section 14.6 defines it.

| Feature | Width |
| --- | --- |
| Cell rank against its cap | 1 |
| Root rank against its cap | 1 |
| Cell energy ratio | 1 |
| Root energy ratio | 1 |
| Cell maturity | 1 |
| Root maturity | 1 |
| Chain length, $\log_2(1 + d)$ normalised by the maximum chain depth | 1 |
| Report staleness, $\log(1 + \Delta t_\text{report})$ | 1 |

The chain length is the block's most-used feature elsewhere: it conditions the depth-dependent views of (`tab:extraction:chain-z-scores`) and it is what makes the order-statistic bias learnable rather than structural.

**Definition (Report staleness)** · `def:extraction:report-staleness`

Report staleness is the wall-clock time since the Sentinel generated the cached batch report, entered logarithmically. Because the Core reads batch summaries rather than per-request scores, the information lost to staleness depends on how far the Sentinel's baselines have moved since the report — a function of elapsed time and of observation volume, of which the Core observes only the first.

The batch context feature (`tab:extraction:batch-context`) supplies a proxy for the second. A cell with a high sample count in the cached report is likely under high throughput, and its baselines are moving proportionally faster, so the model can learn a throughput-adjusted discount from the product of the two features. The proxy breaks under a sudden change in observation rate — a burst arriving after the report — which is exactly the case where staleness matters most. That is a property of a batched measurement interface rather than a defect of the proxy: the Core cannot observe Sentinel state between reports at all, and a deployment needing sub-report sensitivity must shorten the interval rather than ask the extraction for what the interface does not carry.

**Table (Coordination features)** · `tab:extraction:coordination`

Twelve features read from the batch report's coordination summary, which the Sentinel algorithm document specifies at its section 14.9. Each carries a default for the case where the report has no coordination context, and the default is zero throughout, which is the value the absence means rather than a value chosen to be safe.

| Feature | Width | Default when absent |
| --- | --- | --- |
| Per-axis maximum coordination z-score | 4 | 0 |
| Per-axis maximum coordination CUSUM | 4 | 0 |
| Coordination concordance: the fraction of active contexts above the coordination threshold | 1 | 0 |
| Active context fraction | 1 | 0 |
| Peak context depth, normalised by the maximum chain depth | 1 | 0 |
| Root context maximum z-score across axes | 1 | 0 |

Coordination context members are a subset of the reported competitive cells, so these features never reference a cell the rest of the extraction has not seen.

**Table (Outcome memory features)** · `tab:extraction:ledger-features`

Three base features and two per spatially-enabled outcome axis, read from the per-Sentinel outcome memory (`def:ledger:purpose`) rather than from the batch report. Each Sentinel cell holds a decaying history of what happened to requests routed through it, and these features are that history at the request's cell.

| Feature | Width | Condition |
| --- | --- | --- |
| Cell adverse-rate running average | 1 | Always |
| Compressed valence running average | 1 | Always |
| Raw valence running average | 1 | Always |
| Compressed axis running average | 1 | Per active axis with spatial features enabled |
| Raw axis running average | 1 | Per active axis with spatial features enabled |

Whether an axis contributes its two features is the spatial-feature policy of (`disc:registry:spatial-policy`), so the block's width follows the axis registry rather than the Sentinel's report. The convergence and steady-state materiality of these averages are analysed where the memory is specified (`data:ledger:attenuation`); the extraction reads them and asserts nothing about them.

**Table (Batch context)** · `tab:extraction:batch-context`

One feature, the logarithm of the reported cell's sample count in the current batch.

| Feature | Width |
| --- | --- |
| $\log(1 + \text{cell sample count})$ | 1 |

Alone it says how busy the cell was. Its value is in combination: it is the throughput proxy that makes report staleness interpretable (`def:extraction:report-staleness`), and one feature is a cheap price for turning another feature from ambiguous to conditional.

**Definition (Extraction width and the per-Sentinel slot)** · `def:extraction:slot`

The total extraction width per Sentinel is the sum of the six groups:

$$q = 24 + 12 + 8 + 12 + (3 + 2m_s) + 1 = 60 + 2m_s$$

where $m_s$ is the number of active outcome axes with spatial features enabled. Each active Sentinel occupies a contiguous slot of $q + 1$ positions in the feature vector: an occupancy indicator, then the extraction.

$$\text{slot}_s = \bigl[\underbrace{1}_\text{occ} \;\big|\; \underbrace{\mathbf{g}_s}_{q}\bigr] \in \mathbb{R}^{q+1}$$

The occupancy indicator is one whenever the Sentinel is active, whether or not it has data for this particular request, and zero when it has no batch report in the current cycle. It is the prefix that makes the coverage states (`tab:registry:coverage-states`) legible to the model: without it, a silent Sentinel and an offline one present identical zeros.

**Convention (Extraction and slot offsets)** · `conv:extraction:offsets`

Two offset conventions are in use, and both are needed. The extraction reference (`tab:extraction:reference-index`) indexes features within the extraction itself, starting at zero, which is the convention extraction logic is written in. The dimension map (`def:dimension:layers`) reports offsets within the slot, where the occupancy indicator sits at zero and extraction position $i$ sits at slot offset $i + 1$, which is the convention assembly and lifecycle operations are written in.

The two differ by exactly one, which is why they must be named rather than inferred. An offset quoted without its convention is ambiguous by one position in a vector where every position means something different, and the two conventions are kept apart here so that neither site has to guess.

**Remark (Why the extraction stays lean)** · `rem:extraction:lean-extraction`

The extraction is a compression and not a relay. It carries what a risk model can use and leaves the report's structural detail in the report, because a feature the model cannot learn a weight for costs a dimension and returns nothing.

The lever this leaves a deployment is real: dropping the spread and mean chain z-score views saves eight features per Sentinel, giving $q = 52 + 2m_s$, and is worth taking where the label rate is the binding constraint. It is a deployment choice rather than a default, and the choice is between a slightly poorer view of multi-scale structure and a materially faster convergence.

**Remark (Single-precision extraction, double-precision models)** · `rem:extraction:single-precision`

The extraction is computed and stored in single precision; every model that consumes it holds its parameters in double. The split is stated here because it is a property of the interface between the two, and neither side states it alone.

The choice is deliberate on both sides. Extraction values are scores and running averages whose useful precision is a few significant figures, and they are stored per Sentinel per pending request, so their width is a memory cost paid at volume — the same reasoning that fixes the pending buffer's storage precision (`def:runtime:storage-precision`). Model parameters accumulate over hundreds of thousands of updates and are inverted, so their precision is a numerical requirement rather than a preference. Values are upcast where they cross, and nothing downstream may assume the extraction carries more precision than it was stored at.

### Chapter (The Key Space Identity Layer) · `chap:spec:key-space-identity`

The chapter specifies the layer that finds significant ranges in a host-declared key space and reports membership in them as features. It states what the layer is for and what it refuses to do, what a dimension declares, how an observation is processed, what the competitive set is and is not, which features each dimension contributes, what state a competitive cell holds and how it decays, and what entering and leaving the competitive set costs. Dimension lifecycle is not restated here. Registration follows (`alg:registry:dimension-registration`) and removal follows (`alg:registry:dimension-deregistration`), both under the algebra that governs Sentinels.

**Motivation (What the identity layer is for)** · `mot:keyspace:purpose`

Consider a deployment watching network traffic, tracking entity identity by source address. It needs to learn that requests from a particular subnet have historically been associated with adverse outcomes — and it needs that grouping to emerge from observation volume rather than from a static configuration, since nobody knows in advance which subnet will matter. The identity layer does this: it maintains a graph over the address space, discovers significant ranges by competitive ranking, and hands the models an indicator saying that this request comes from a range the system considers significant. The model learns each range's risk weight from labelled outcomes.

Three things the layer does not do. It does not track entities. It does not maintain per-entity state as a primary function. And it interprets nothing: it partitions integer ranges, ranks cells by observation volume, and reports which competitive cells contain a coordinate. The host reads a cell as an entity, a group, an organisation, a device family, or a region, and the mechanism reads it as a range.

**Schema (Identity dimension registration record)** · `schema:keyspace:dimension-record`

A dimension is registered with an identifier, a name, a description, a declared domain width, its coordinate semantics, an encoding function from entity keys to coordinates, a budget, and a competitive depth cutoff.

| Field | Carries |
| --- | --- |
| Identifier | The registry key |
| Name | Unique across currently registered dimensions |
| Description | What this key space is, in the host's own words |
| Declared domain width | The width of the dimension's integer domain (`def:encoding:domain-width`) |
| Coordinate semantics | The same declaration a Sentinel makes (`schema:registry:sentinel-record`) |
| Encoding function | The map from an entity key to a coordinate in the domain |
| Budget | The graph's sizing and decay parameters |
| Competitive depth cutoff | The depth beyond which an entry is not competitive |

The budget carries five parameters: the graph's maximum size, the split threshold, the depth at which a cell may be created, the depth at which one is evicted, and the spatial decay rate applied to importance. They are the parameters the underlying graph is sized by, and they are declared at registration rather than derived, because a host that cannot set them cannot control what the layer costs.

The coordinate semantics are the Sentinel's declaration in every respect, including the assertion that shared prefixes imply shared context, and the Core does not verify the assertion here either.

**Requirement (The encoding contract for key spaces)** · `req:keyspace:encoding-contract`

The encoding contract (`req:encoding:host-contract`) applies to an identity dimension exactly as it applies to a Sentinel, and its three questions are asked of the key space rather than of a data source. Does this key space have hierarchical positional structure (`def:encoding:question-hierarchy`)? Network addresses do, through the prefix tree; electronic mail addresses do under a reversed-domain encoding; composite tenant keys do with the tenant in the high bits; session tokens do not. What does a shared prefix assert (`def:encoding:question-meaning`)? For addresses, a shared network; for reversed domains, a shared organisation; for composite keys, the same tenant. And does the hierarchy run deep enough (`def:encoding:question-depth`)? Address spaces do, at thirty-two bits or a hundred and twenty-eight; a domain encoding is moderate; a hashed token has no hierarchy at any depth.

A dimension should not be registered for a key space without hierarchical structure. The mechanism will run without error — that is the difficulty — but the competitive cells will correspond to arbitrary groupings of strangers, the indicators will carry no meaning, and the models will learn weights at or near zero on all of them. The cost is not a wrong answer; it is model dimensions spent, convergence slowed, and a layer that looks like it is working.

**Algorithm (Observation protocol)** · `alg:keyspace:observation-protocol`

At each assessment, for each registered dimension:

1. **Compute the coordinate** by applying the dimension's encoding function to the entity key.
2. **Record the observation** at unit weight for deferred processing by the dimension's graph, which maintains importance and triggers structural operations asynchronously. The assessment does not wait for it: the competitive set is read from the most recently published state, and the deferred queue is drained off the assessment path.
3. **Determine the active competitive cells** — those cells of the competitive set whose range contains the coordinate.
4. **Set the indicators**, one for each active cell and zero for the rest.

The weight recorded at step two is always unit. No anomaly score and no risk estimate reaches it, which is how the feed-forward invariant (`inv:guarantee:feed-forward`) holds through the one graph the Core operates itself: the Core observes into the graph, and nothing the Core computes influences what the graph does with the observation.

Step three is a walk rather than a search. Because dyadic intervals either nest or are disjoint, the active set is a chain under containment, so the mechanism walks from the deepest containing cell toward the root and collects the competitive cells on the path, at a cost set by that cell's depth — bounded by the domain width, and in practice three to eight levels. The number of active indicators per request per dimension is data-dependent and unconstrained, typically none to three.

**Definition (The competitive set)** · `def:keyspace:competitive-set`

The competitive set of a dimension is the set of the dimension's graph entries standing within its configured depth cutoff. There is no further filtering: no state filter, no minimum width, no coverage requirement. At a cutoff of three, with a branching factor of two to three, the set holds roughly ten to twenty-five entries, the exact count set by the observation distribution and by competitive dynamics rather than by configuration. The depth bound is what floors an entry's importance: an entry at depth three carries at least about fifteen per cent of the dimension's total importance.

The set has no spatial invariant, and the absences are deliberate. It is not a partition: cells may nest, and gaps between them are ordinary. It is not a cover: most coordinates lie in no competitive cell at all. It is not required to tile any fraction of the domain. Nesting is permitted, and because dyadic intervals nest or are disjoint, an active set is always a chain and never partially overlapping. And the set's composition changes whenever the underlying tree restructures. Everything downstream is written against a set with these properties, so a reader looking for a guarantee of coverage will not find one here because there is none to state.

**Definition (Competitive indicators)** · `def:keyspace:competitive-indicators`

Each competitive cell contributes one indicator feature, so a dimension's indicator count equals its competitive set's current size and changes when the set restructures.

$$\phi_{\text{dim}_d} = \bigl(\mathbb{1}[x_d \in c_1],\; \ldots,\; \mathbb{1}[x_d \in c_{|\mathcal{E}_d|}]\bigr)$$

The model learns one weight per cell, and a request's identity contribution is the sum of the weights of the cells containing its coordinate:

$$\Delta\hat{\rho}_{\text{dim}_d} = \sum_{c \in \text{active}_d(x_d)} \mu_c$$

The sum has no terms where no competitive cell covers the coordinate, and one term per covering cell where some do. The terms compose additively, which is what decomposes risk across whatever scales the competitive mechanism happened to select (`prin:principle:multi-level-decomposition`): the entity, its enclosing group, and the broader region each contribute their own learned weight where each is competitive.

**Table (Per-dimension features)** · `tab:keyspace:dimension-features`

Every registered dimension contributes a contiguous block of $8 + 3m$ features for $m$ registered outcome axes, combining structure, measurement, and outcome history over the dimension's active competitive cells (`def:keyspace:competitive-set`).

| Feature | Width | Source | Updated |
| --- | --- | --- | --- |
| Coverage depth | 1 | The receiving cell's depth over the domain width | Assessment |
| Active indicator count | 1 | Active cells over competitive set size | Assessment |
| Total dimension importance, logarithmic | 1 | The dimension's total importance | Assessment |
| Maximum alarm across active cells | 1 | Per-range measurement state | Assessment |
| Mean alarm across active cells | 1 | Per-range measurement state | Assessment |
| Maximum suspicion across active cells | 1 | Per-range measurement state | Assessment |
| Volatility of the deepest active cell | 1 | The most specific active cell | Assessment |
| Has a competitive cell | 1 | Whether any cell is active | Assessment |
| Mean compressed axis average, per axis | $m$ | Per-cell outcome state | Label |
| Mean raw axis average, per axis | $m$ | Per-cell outcome state | Label |
| Axis stability, per axis | $m$ | Spread of the compressed averages across active cells | Label |

The block is fixed-width per dimension: it does not change when the competitive set restructures, which is what lets a dimension's indicators change count without the rest of its block moving. The first eight are written at assessment time and the remaining $3m$ at label time. The per-axis features are independent of the spatial-feature policy — that policy governs the Sentinel outcome-memory features (`tab:extraction:ledger-features`), not identity cell state.

**Table (Cross-dimension aggregates)** · `tab:keyspace:cross-dimension-features`

Where at least one dimension is registered, a fixed block of eight features summarises across dimensions by maximum. Six of them are consumed by the fixed-dimension anchor model at named positions in its projection (`def:dimension:anchor-projection`), and five carry a name the default interaction templates refer to them by.

| Feature | Aggregation | Anchor position | Template name |
| --- | --- | --- | --- |
| Maximum suspicion | Across dimensions, over active cells | 7 | Maximum suspicion |
| Maximum alarm | Across dimensions, over active cells | — | Maximum alarm |
| Maximum volatility | Across dimensions, over deepest active cells | 12 | Volatility |
| Maximum adverse rate | Across dimensions, over active cells | 8 | — |
| Maximum absolute compressed valence | Across dimensions, over active cells | 10 | — |
| Maximum absolute raw valence | Across dimensions, over active cells | 11 | — |
| Has any competitive cell | Across dimensions | 9 | Has competitive |
| Maximum coverage depth | Across dimensions | — | Coverage depth |

The block is zero-filled where no cell is active in any dimension. It is created when the first dimension is registered and destroyed when the last is deregistered, and it changes width neither on restructuring nor on any lifecycle event within the layer — which is what allows the anchor's projection to name positions in it at all.

**Table (The signal cache)** · `tab:keyspace:signal-cache`

Entity-persistent signals — authentication status, account age, risk tier — are operationally important and must not be gated on competitive standing. They are held in a per-entity cache independent of the layer's graphs, a hash map with least-recently-used eviction.

| Property | Value |
| --- | --- |
| Capacity | Configurable, default 100,000 entries |
| Entry size | About 80 bytes, ten signals of eight bytes |
| Eviction | Least recently used |
| Total memory | About 8 MB at the default capacity |

The cache is deliberately outside the identity layer proper. An entity's authentication status is a fact about the entity whether or not its range has earned competitive standing, and coupling the two would make an operationally important signal disappear exactly when an entity became uninteresting to the competitive mechanism. Competitive entry and exit are unaffected by cache state, and cache eviction loses evidence without touching the layer.

**Table (Per-range measurement state)** · `tab:keyspace:measurement-state`

Each competitive cell in every dimension carries measurement state: how the requests routed through this range currently look to the Sentinel measurement layer.

| Component | Update rule |
| --- | --- |
| Per-Sentinel maximum composite alarm average | Exponential average at the measurement smoothing factor, over the request's composite alarm from that Sentinel |
| Suspicion | Derived: the mean of the clipped normalised alarms |
| Volatility | Exponential average over the squared change in the alarm average |
| Step count | Incremented once per assessment |

The state is updated at each assessment for the active competitive cells of every registered dimension, taking the current batch reports' composite alarm as its input. It contributes the measurement features of (`tab:keyspace:dimension-features`) rather than a block of its own, and is zero-filled where a dimension has no active cell.

Measurement state exists only in competitive cells. It initialises to zero on entry, is discarded on exit, propagates through neither merge nor split, and is held by no non-competitive cell. It is a feature of competitive attention rather than a property of the key space, and at the default smoothing factor its averages converge within about twenty assessments, so the warm-up after entry is short.

**Table (Per-cell outcome state)** · `tab:keyspace:outcome-state`

Where measurement state tracks what the measurement layer currently sees, outcome state tracks what happened to labelled requests from the range. It is written at label time, and each competitive cell in every dimension holds six components.

| Component | Update rule |
| --- | --- |
| Adverse rate average | Exponential average over the positive-valence indicator (`conv:valence:sign`) |
| Compressed valence average | Exponential average over the compressed valence, saturating at unit magnitude |
| Raw valence average | Exponential average over the reported valence, unbounded |
| Per-axis compressed average | One per registered axis, over the compressed axis value |
| Per-axis raw average | One per registered axis, over the reported axis value |
| Last updated | Overwritten, for lazy time-indexed decay |

A label updates every competitive cell containing its coordinate in each dimension. Because an active set can hold several nested cells, one label can write several cells in one dimension — the outcome applies to the specific entity, to the enclosing group, and to the broader region, wherever all three are competitive.

The state serves two purposes and no third. It supplies the three per-axis features each dimension contributes for each registered axis (`tab:keyspace:dimension-features`), computed across the active cells at assessment time and falling back to zero where no cell is active, or where at most one is for the stability feature. And it supplies warm-start context when the set restructures: a cell entering a region where a parent's outcome state survives can inherit a blended state rather than starting cold.

Under the recommended default, outcome state is not extracted as per-cell features. The per-cell weight learned from labels already captures the cell-specific outcome signal, and the running average is redundant with it: the weight is the more powerful of the two because it conditions on every other feature, and the average is the faster to converge because it conditions on nothing. The three per-axis features are aggregates across active cells, not additional per-cell slots, and that is the whole of what outcome state contributes to the vector.

**Table (Decay in the identity layer)** · `tab:keyspace:decay-rates`

Three mechanisms decay identity state, at three rates and for three reasons.

| What decays | Mechanism | Rate | Purpose |
| --- | --- | --- | --- |
| Cell importance in the graph | Spatial decay, per hour | The dimension's declared spatial decay rate | Governs how the contour evolves and who stays competitive |
| Outcome averages in competitive cells | Time-indexed lazy decay, per hour | Default $0.998$, a half-life near fourteen days | Prevents stale outcome state |
| Measurement averages in competitive cells | Per-assessment exponential average | Default $0.95$ | Tracks the current alarm profile |

Importance decay governs how quickly an inactive entity loses competitive standing: at the default rate an entity unseen for a fortnight has its importance halved, and if that drops it past the cutoff it exits the set, its features are marginalised out, and its cell is absorbed into its parent where the graph evicts it.

Outcome decay is what breaks the contamination loop (`alg:valence:contamination-loop`) in this layer: stale outcome history dissolves on a fourteen-day timescale whatever the label flow, so a starved cell still forgets. That is faster than the spatial outcome memory's own rate (`def:ledger:time-decay`), and deliberately so — identity cell state is the more volatile of the two, describing a range's current occupants rather than a coordinate region's long history.

**Algorithm (Competitive entry)** · `alg:keyspace:cell-entry`

The batching unit for competitive lifecycle is one restructuring event, not one cell. Where an event promotes or demotes several cells, every resulting extension and marginalisation is applied in deterministic event order and followed by a single map rebuild, so the rebuild's linear cost is paid once per event.

On promotion of a cell into the competitive layers:

1. **Extend every model but the anchor** by one indicator dimension plus the competitive interaction dimensions declared for this dimension — one by default — at the prior, with zero off-diagonals (`thm:gaussian:extension`).
2. **Extend standardisation** at the class priors: a binary indicator at mean one half and variance one quarter, an interaction product at mean zero and unit variance.
3. **Initialise outcome state**, at zero or at the parent's blended state where the parent was previously competitive.
4. **Initialise measurement state** at zero.
5. **Record the new membership** for the event's final map rebuild.

Entry costs one pass per model, and the anchor is untouched.

**Algorithm (Competitive exit)** · `alg:keyspace:cell-exit`

On demotion of a cell past the depth cutoff:

1. **Marginalise every model but the anchor** by the same one plus the declared competitive interaction dimensions, removed together in a single step. At the default this is a rank-two correction, quadratic in the dimension per model (`alg:gaussian:regularised-schur`). The covariance path reads its block directly, and the residual discrepancy is cleared at the next scheduled recomputation (`rem:gaussian:dual-tracking`).
2. **Remove the standardisation entries.**
3. **Discard the measurement state.**
4. **Retain the outcome state** in the cell, for the warm start of a future entry from this region, or discard it where the graph has evicted the cell.
5. **Record the removed membership** for the event's final map rebuild.

Exit preserves more than it appears to. The Schur complement transfers what the models learned about the relationship between this cell's indicator and every surviving feature into the survivors' precision (`def:gaussian:schur-complement`): if the models learned that this cell's indicator together with an elevated address alarm predicted adverse outcomes, that correlation's contribution to the address feature's precision survives the removal. What does not survive is the cell's own weight, which starts from zero if the cell later re-enters. At three models and the reference dimension, an exit is roughly 1.2M operations, and the set turns over a few times a day under typical decay parameters, so the lifecycle's total cost is negligible beside the label path.

**Requirement (What the host does)** · `req:keyspace:host-duties`

At construction the host does four things. It decides which facets of entity identity have hierarchical structure worth exploiting. It supplies, per dimension, an encoding function preserving that structure under the same contract Sentinel encoding is held to (`req:encoding:host-contract`). It configures each dimension's budget parameters, which are the parameters the underlying graph's spatial layer defines and which the Sentinel algorithm document declares at its section 3.11 — a host told to configure a parameter and not told where it is defined has been told nothing. And it declares its entity-persistent signals (`schema:signal:shapes`).

At assessment time the host provides the entity key and the current values of any entity-persistent signals. At label time it provides the label, and the outcome update reaches every competitive cell containing the labelled coordinate automatically.

Three ongoing duties. Monitor each dimension's association with the outcome (`def:monitoring:slot-association`) over that dimension's own block: where it sits at the null floor across a sustained stretch of traffic, and the dimension's contribution (`def:monitoring:slot-contribution`) confirms it by sitting below its own threshold, the encoding is returning nothing and the dimension should be revised or removed.

The trigger is two readings and not one because each answers what the other cannot. The association reading is model-free, so it screens: it is what raises, and it cannot be climbed by a model that has run out of residual. The contribution reading is the removal cost itself, so it confirms: it is what says the removal is actually free. Either alone misleads in a direction the other covers. A block can carry the outcome and still cost nothing to remove, because a second block carries the same information; a block can be idle across a stretch of traffic without having stopped mattering to the score. Retiring on the first alone discards dimensions that were earning their width, and retiring on the second alone keeps dimensions that were not.

The two terms are not equally evidenced at this scope, and a host reading them should know which is which. The screening term is measured: driven against a null dimension differing only in whether its coordinate was drawn from the half of the pool the outcome named, a keyed dimension's association stood at least five and a half times the null one's at every point of a schedule that rebuilt the layout halfway through, the keyed figure clear of the alert band and the null figure inside it (`def:monitoring:slot-association`). The confirming term is not: driven against the same pair on the same schedule, its two figures crossed each other in both directions across repeated runs, because unlike the screening term it scales with the operational weights and so with a convergence state a schedule does not fix (`def:monitoring:slot-contribution`). So a per-dimension alert that the screening term raises is an alert on measured ground, and the confirmation the duty asks for beside it is a reading whose discrimination at this scope no measurement yet supports. That is a reason to weigh the confirmation rather than to take a silent one as licence, and it is not a reason to retire on the screen alone: the failure the second reading covers — a block carrying the outcome that a duplicate makes free to remove — is real whether or not the reading has been shown to catch it.

The quantity the duty names is the per-dimension restriction of both readings and not the per-Sentinel one they are the sibling of (`def:monitoring:encoding-effectiveness`) — all of these are taken over disjoint blocks of the vector, so a Sentinel-scoped figure could not say which dimension to revise. The dimension's weight mass (`def:monitoring:dimension-informativeness`) is reported beside them and is deliberately not the trigger. It says how much weight the model carries on the dimension's block, which a block predicting nothing earns about as readily as one that predicts: driven in two worlds differing only in whether the entity predicted the outcome, a null dimension read between $67\%$ and $143\%$ of the keyed one's figure and crossed it in both directions, and the later measurement that established the association reading's separation found the same crossing on its own pair over a wider range. A duty triggered on it would send hosts to revise dimensions at random with respect to whether the dimension was returning anything. The dimension's cell weights are reported beside it on the same footing and for the same reason a second reading is wanted at all: the block readings are over the dimension's block and not over its competitive indicators, and a dimension can carry weight in the one and none in the other. Monitor competitive set stability: frequent churn suggests the split threshold or the decay rate wants adjusting. Monitor coverage: a set covering less than a tenth of assessment volume suggests a cutoff too restrictive or a threshold too high.

Everything else is the mechanism's. Which entities earn competitive standing, what the ranges are, how the weights decompose risk across scales, when cells enter and leave, and what marginalisation preserves when they do — none of it is the host's to decide. The host supplies the key space, the encoding, and the budget.

### Chapter (Host Signals) · `chap:spec:host-signals`

The chapter specifies the one block of the feature vector the host fills directly. Everything else the Core sees is derived: from a Sentinel's batch report, from a key space's competitive structure, or from products of the two. Host signals are the channel for what the host already knows and no Sentinel can observe — whether the request authenticated, how old the account is, what tier the customer sits in, what hour of the day it is. The chapter states what a declaration carries, how much of the vector each shape occupies, and when the schema is fixed.

**Schema (Signal declaration)** · `schema:signal:shapes`

A signal is declared with a name, a shape, and a persistence mode. The shape says how the host's value becomes features; the persistence mode says where the value is read from at assessment time.

| Shape | The host supplies | Encoded as |
| --- | --- | --- |
| Scalar | A value and a clipping interval | The value, clipped to the interval |
| Log-scaled | A value and a divisor | The signed logarithm of one plus the scaled magnitude |
| Binary | A truth value | One or zero, thresholded at one half |
| Ordinal | A value and its maximum | The value normalised onto the unit interval |
| Cyclic | A value and its period | The sine and cosine of the phase |
| Hashed categorical | A category and a bin count | An indicator at the bin the category hashes to |
| Vector | A fixed-length vector and a clipping interval | Each element, clipped |

Two persistence modes. An entity-persistent signal is a fact about the entity rather than about the request, and is read on the entity key from the signal cache (`tab:keyspace:signal-cache`), which is why its availability does not depend on the entity holding competitive standing anywhere. A request-scoped signal arrives with the assessment and needs no storage at all. A signal absent at assessment time is zero-filled under either mode, so a declaration is a promise about shape and never a promise about presence.

**Remark (The shapes occupy different widths)** · `rem:signal:shape-widths`

Four of the seven shapes occupy one position, and three do not. The width rule is stated here because a reader who has only the shape names will assume seven signals cost seven positions, and will be wrong by one for every cyclic signal declared.

| Shape | Width |
| --- | --- |
| Scalar, log-scaled, binary, ordinal | 1 |
| Cyclic | 2 |
| Hashed categorical | The declared bin count |
| Vector | The declared length |

The signal block's width is the sum of the declared widths, in declaration order. The cyclic shape is the one that surprises: a phase cannot be carried by a single feature without a discontinuity at the wrap point, where the last moment of one cycle and the first of the next would sit at opposite ends of the range while denoting adjacent instants, so the sine and cosine are carried together and the model learns a weight on each.

**Requirement (The signal schema is fixed at construction)** · `req:signal:schema-fixed`

The schema is declared once, at construction, and never changes. The signal block's width is fixed by that declaration, and each signal's position is derived by composing the block's start with the signal's declaration order — the schema is the authority for the order, and no position is stored per signal.

This is the block that stands still. Sentinel slots are created and destroyed with their Sentinels, per-dimension blocks widen and narrow with the outcome axis registry, and competitive indicators come and go with the competitive set; the signal block does none of these things, and its features keep their meaning for the life of the deployment. The cost is that a signal not foreseen at construction cannot be added later, and the benefit is that the one block whose semantics are entirely the host's own is also the one block no lifecycle event can move underneath it.

### Chapter (The Feature Vector) · `chap:spec:feature-vector`

The chapter assembles Part II. Everything the preceding chapters define — each Sentinel's fixed-width extraction, each identity dimension's competitive features, the host's declared signals — arrives here as a block of one vector, augmented with cross-Sentinel summaries and with products of pairs. The chapter states the vector's structure and its dimension, the aggregate block, the template system that generates the products, the map that gives every position a name, and the standardisation that puts every position on a comparable scale.

**Definition (The structure of the feature vector)** · `def:feature:vector-structure`

The feature vector is the concatenation of eight block types in a fixed order: the bias, the aggregate block, one block per identity dimension, the cross-dimension block, the signal block, one slot per registered Sentinel, the interaction block, and the competitive indicators.

$$\phi = \bigl[\underbrace{1}_\text{bias} \;\big|\; \underbrace{\phi_\text{agg}}_{p_\text{agg}} \;\big|\; \underbrace{\phi_{\text{dim}_1}}_{8+3m} \;\big|\; \cdots \;\big|\; \underbrace{\phi_{\text{dim}_D}}_{8+3m} \;\big|\; \underbrace{\phi_\text{cross}}_{8} \;\big|\; \underbrace{\phi_\text{sig}}_{p_\text{sig}} \;\big|\; \underbrace{\text{slot}_{s_1}}_{q+1} \;\big|\; \cdots \;\big|\; \underbrace{\text{slot}_{s_n}}_{q+1} \;\big|\; \underbrace{\phi_\text{int}}_{p_\text{int}} \;\big|\; \underbrace{\phi_\text{id-comp}}_{C}\bigr]$$

Here $D$ is the number of registered identity dimensions, $n$ the number of registered Sentinels, $m$ the number of active outcome axes, $m_s$ the number of those with spatial features enabled, and $C$ the total competitive indicator count across every dimension. The cross-dimension block is present exactly when $D$ is positive.

The order is not arbitrary. The bias and the aggregate block precede every dynamic block, which is what makes their positions stable under all lifecycle events; the competitive indicators, the most volatile block of all, sit last, where their coming and going disturbs the fewest positions.

**Table (The dimension formula)** · `tab:feature:dimension-formula`

The total dimension is the sum of the block widths.

$$p = 1 + p_\text{agg} + (8 + 3m)D + 8\mathbb{1}[D > 0] + p_\text{sig} + n(61 + 2m_s) + p_\text{int} + C$$

| Block | Width | What changes it |
| --- | --- | --- |
| Bias | 1 | Nothing |
| Aggregate | $p_\text{agg} = 15$ | Nothing (`tab:feature:aggregate-block`) |
| Identity per-dimension | $8 + 3m$ each | Identity dimension or outcome axis lifecycle (`tab:keyspace:dimension-features`) |
| Identity cross-dimension | $8$ when $D > 0$ | The first and last identity dimension (`tab:keyspace:cross-dimension-features`) |
| Signal | $p_\text{sig}$ | Nothing after construction (`req:signal:schema-fixed`) |
| Sentinel slot | $q + 1 = 61 + 2m_s$ each | Sentinel lifecycle; outcome axis lifecycle changing $m_s$ (`def:extraction:slot`) |
| Interactions | $p_\text{int}$ | Every event the declared templates expand over |
| Competitive indicators | $C$ | Competitive set restructuring (`def:keyspace:competitive-set`) |

Three of the eight are fixed for the life of the deployment and five move. Each of the five moves under a named lifecycle event and under no other, which is what makes the dimension a derived quantity rather than a configured one: no deployment sets $p$, and every change to it is the arithmetic consequence of a registration, a deregistration, or a restructuring.

#### Aggregate features · `sec:feature:aggregate`

The division carries no material of its own. It collects the purpose of the aggregate block, the fifteen features it holds, and the recalibration of the thresholds five of those features are counted against.

**Motivation (Why the aggregate block exists)** · `mot:feature:aggregate-purpose`

Per-Sentinel slots record what each specialist reports. The aggregate block records what the fleet reports together: how loud the loudest alarm is, how far the Sentinels agree, how many kinds of anomaly are active at once, and how much of the measurement surface is online at all. These are not recoverable from the slots by a linear model, because a maximum and a standard deviation across slots are not linear in the slots, and a model given only the slots would have to approximate them from weights that also have to serve every other purpose.

The block has fixed width whatever the number of Sentinels or axes, so its positions never move, and it is populated whenever any Sentinel reports at all. That combination makes it the fastest-converging evidence the Core holds: it needs no per-cell outcome history and no per-Sentinel history, only a report, which is why it is one of the pathways that carry detection while the slower layers are still learning (`tab:ledger:low-maturity-detection`).

**Table (The aggregate feature block)** · `tab:feature:aggregate-block`

Fifteen features summarise the current batch across every reporting Sentinel.

| Feature | Width | Carries |
| --- | --- | --- |
| Maximum cell-level maximum z-score across active Sentinels | 1 | The loudest alarm anywhere |
| Mean cell-level maximum z-score | 1 | The average alarm |
| Standard deviation of the cell-level maximum z-score | 1 | Agreement against disagreement |
| Per-axis maximum z-score across Sentinels | 4 | The worst alarm of each kind |
| Per-axis concordance | 4 | The fraction of reporting Sentinels above that axis's threshold |
| Cross-axis product | 1 | Novelty against displacement, compounded |
| Axis breadth | 1 | How many kinds of anomaly are above threshold |
| Coverage | 1 | Reporting Sentinels over registered Sentinels |
| Maximum cumulative sum across every Sentinel and axis | 1 | The worst sustained drift |

The four concordance features and the breadth feature are the only ones read against a threshold rather than reported directly, which is why the thresholds need a calibration of their own (`alg:feature:concordance-recalibration`). The remaining ten are order statistics over the batch and carry no state between assessments.

**Algorithm (Concordance threshold recalibration)** · `alg:feature:concordance-recalibration`

The per-axis threshold is the eightieth percentile of the axis's recent sub-scores, so that concordance reports how many Sentinels are unusually loud for this deployment rather than how many exceed a constant chosen elsewhere.

1. **Accumulate**, on the assessment path, one sub-score per axis per reporting Sentinel into a rolling window holding the last ten thousand.
2. **Recompute** every thousand assessments, taking each axis's threshold as the eightieth percentile of its window.
3. **Publish** the thresholds as state of their own, outside the model snapshot.

Thresholds start at one half and move only at a recomputation. They are held apart from the model snapshot deliberately: a threshold change alters what the concordance features mean but not what any model parameter means, so it triggers neither a calibration recomputation nor a drift reset. The accumulation is an auxiliary write on the assessment path (`inv:runtime:enumerated-writes`) and nothing on that path reads a model to perform it.

#### The interaction template system · `sec:feature:interactions`

The division carries no material of its own. It collects the five template types, the default set built from them, their convergence behaviour, what lifecycle events do to them, and the interaction dimension the default set implies.

**Motivation (Interactions as pairwise products)** · `mot:feature:interaction-overview`

The core models are linear in the feature vector (`cav:limitation:linear`), so a conjunction — this Sentinel is loud *and* this entity is already suspect — cannot be represented by weights on the two features alone. An interaction feature is the product of two features already in the vector, which gives the model exactly the conjunctions it is declared to want and nothing else.

Interactions are declared as templates rather than as features because the features they stand for do not exist yet. A template names two operands symbolically and a rule for expanding over the current registries; the expansion produces the features. Five types differ in what they expand over, and therefore in how their count scales and how fast each feature converges: a template expanding over nothing produces one feature that every labelled request touches, and a template expanding over pairs of Sentinels produces many features that few labelled requests touch. The choice among them is a choice about where to spend model capacity.

**Definition (The per-Sentinel template)** · `def:feature:template-per-sentinel`

A per-Sentinel template names one feature within a Sentinel slot and one context feature — identity, signal, or aggregate — and expands to one feature per registered Sentinel, each the product of that Sentinel's slot feature with the shared context feature. The count grows linearly in the number of Sentinels.

The template's Sentinel operand is written with a quantifier rather than a name, and the quantifier is what binds it to each expansion in turn. This is the type that asks whether a *particular* source's alarm means more when the entity is already suspect than the same alarm from another source would, and it is the cheapest way to ask that question, since the alternative — one declared template per Sentinel — would have to be edited every time a Sentinel is registered.

**Definition (The named-pair template)** · `def:feature:template-named-pair`

A named-pair template names two Sentinels explicitly and expands to exactly one feature, the product of a feature from each. Its count does not grow with the number of Sentinels at all. The feature exists only while both named Sentinels are registered: it is created when the second of the pair registers and destroyed when either deregisters.

This is the type for a conjunction the operator already suspects — two specific sources whose simultaneous alarm is known to mean more than either alone. It buys the cross-Sentinel question at constant cost by requiring the pair to be named in advance, which is the whole of the trade: a named pair costs one dimension and answers one question, and a pair nobody thought to name is not asked about.

**Definition (The wildcard template)** · `def:feature:template-wildcard`

A wildcard template names no Sentinel and expands to one feature for every unordered pair of registered Sentinels, so its count grows as the square of the number of Sentinels. At ten Sentinels a single wildcard template produces forty-five features.

The type is to be used sparingly, and the quadratic count is only half the reason. Each of those forty-five features is the product of two specific Sentinels' values, so it is informative only on requests where both of that pair report — a low rate for most pairs — and the convergence of a feature is governed by how often it is non-zero in labelled data (`cav:limitation:interaction-type-three`). A wildcard template therefore buys a large block of model capacity that takes months to exploit, and it buys it uniformly across pairs, including every pair whose joint behaviour means nothing.

**Definition (The aggregate template)** · `def:feature:template-aggregate`

An aggregate template names one feature of the aggregate block and one context feature, and expands to exactly one feature regardless of how many Sentinels or dimensions are registered. Its count is constant.

This is the fastest-converging interaction type, and for the same reason the aggregate block itself converges fastest: the aggregate operand is populated on every request with at least one reporting Sentinel, so the product is non-zero on a large fraction of labelled requests rather than on the narrow slice where one particular Sentinel or one particular pair happened to report. It asks the cross-source question without naming a source, which is a weaker question than the pair types ask and a very much cheaper one.

**Definition (The competitive template)** · `def:feature:template-competitive`

A competitive template names one context feature and expands to one feature per competitive cell per identity dimension, each the product of that cell's indicator with the context feature. Its count is the total competitive indicator count, so it grows and shrinks with the competitive sets rather than with any registry.

This is the only template type whose expansion changes without a lifecycle event: competitive entry (`alg:keyspace:cell-entry`) adds one feature per template per dimension and competitive exit (`alg:keyspace:cell-exit`) removes one, each under the same extension and marginalisation the indicator itself takes. It asks whether alarm from this particular range means more than the same alarm elsewhere — the per-range refinement of the identity layer's own additive decomposition, learned per cell rather than per dimension.

**Table (The default interaction set)** · `tab:feature:default-interaction-set`

The recommended set declares five per-Sentinel templates, eight aggregate templates, one competitive template per identity dimension, and no wildcard template at all.

| Type | First operand | Second operand |
| --- | --- | --- |
| Per-Sentinel | Cell maximum z-score | Identity maximum suspicion |
| Per-Sentinel | Cell maximum z-score | Identity coverage depth |
| Per-Sentinel | Cell maximum z-score | Identity has-competitive |
| Per-Sentinel | Cell cumulative sum | Identity volatility |
| Per-Sentinel | Cell adverse rate | Cell maximum z-score |
| Aggregate | Maximum alarm | Identity maximum suspicion |
| Aggregate | Maximum alarm | Identity volatility |
| Aggregate | Concordance | Identity coverage depth |
| Aggregate | Coverage | Identity coverage depth |
| Aggregate | Coverage | Maximum alarm |
| Aggregate | Axis breadth | Identity maximum suspicion |
| Aggregate | Cross-axis product | Identity maximum alarm |
| Aggregate | Maximum cumulative sum | Identity maximum alarm |
| Competitive | The cell's indicator | Maximum alarm |

The competitive row is declared once and expands per dimension; the other thirteen are declared once each. Every identity operand is drawn from the cross-dimension aggregate block (`tab:keyspace:cross-dimension-features`), never from a per-dimension block, which is why the per-Sentinel and aggregate template counts do not change when a dimension is registered. The set is a recommendation and not a constraint: the templates are declared by the host at construction, and a deployment that knows its own Sentinels may name pairs the default set cannot know to name.

**Table (Interaction convergence)** · `tab:feature:interaction-convergence`

An interaction feature converges at a rate set by how often it is non-zero in labelled data, so the types converge on quite different timescales.

| Features | Co-occurrence | At two hundred labels a day |
| --- | --- | --- |
| Raw per-Sentinel features | Near certain | About a day |
| Aggregate templates | 5 to 20 per cent | Two to seven days |
| Per-Sentinel templates | 0.5 to 5 per cent | One to eight weeks |
| Competitive templates | 1 to 10 per cent | One to eight weeks |
| Named-pair templates | 0.1 to 1 per cent | Two to twenty weeks |
| Wildcard templates | 0.1 to 1 per cent | Two to twenty weeks |

The ordering is the design. While the pair types are still converging, the faster layers already cover most of what they will eventually cover: raw features carry per-source alarm within days, aggregate templates carry cross-source alarm without source identity within days, and the per-Sentinel and competitive templates carry alarm against identity state within weeks. What the pair types add over all of that is the one thing none of the others can see — a joint signature across two named sources, each of which looks unremarkable alone (`cav:limitation:cross-sentinel-gap`).

**Algorithm (Interactions under lifecycle events)** · `alg:feature:lifecycle-interactions`

Every lifecycle event that changes what a template expands over changes the interaction block, and each change is discharged through the same algebra as the block that caused it.

1. **On Sentinel registration**, add one feature per per-Sentinel template, one per named-pair template whose other Sentinel is already registered, and one per wildcard template for each Sentinel already registered.
2. **On Sentinel deregistration**, remove exactly the features the registration would have added, identified from the resolution map rather than by scanning (`alg:registry:sentinel-deregistration`).
3. **On identity dimension registration or deregistration**, add or remove every competitive template's features for that dimension, in one batch.
4. **On competitive entry or exit**, add or remove one feature per competitive template for the owning dimension.
5. **Re-resolve and recompile** the declared templates against the registries as they now stand.

Declarations are never re-read at runtime and never change; only their expansion does. That is what allows step five to be a total recomputation from the registries rather than a reconciliation against the previous expansion.

**Equation (The default interaction dimension)** · `eq:feature:default-interaction-dimension`

Under the default set, the interaction block's width follows from the template counts and the expansion rules:

$$p_\text{int} = 5n + 8 + C$$

The five per-Sentinel templates contribute one feature each per registered Sentinel, the eight aggregate templates contribute one feature each whatever the registries hold, and the single competitive template per dimension contributes one feature per competitive cell, which totals the competitive indicator count. There is no quadratic term because the default set declares no wildcard template; adding one would add a term in $n(n-1)/2$, and the formula is written to make that addition visible rather than to hide it inside a constant.

#### The dimension map · `sec:dimension:map`

The map is the isomorphism between semantic identity and dense vector index. Every parameter, feature, and standardisation statistic lives at a position; the map is what gives each position a name and each name a position, and it is what turns "marginalise this Sentinel" from a phrase into an index set. The division collects its three invariants, its compaction and rebuild, its layers and conventions, its compilation pipeline, its competitive and anchor structure, and the record that carries it.

**Invariant (Covering)** · `inv:dimension:covering`

Every index below the total dimension is owned by exactly one semantic entity. No index is unassigned and no index is assigned twice.

The invariant is what makes a learned weight interpretable at all: a weight whose index had two owners would be a weight about two things, and the principle that identities survive into the model (`prin:principle:identity-preservation`) would be false at the level where it matters most. It is also what makes reconstruction at label time well defined (`alg:runtime:reconstruction`), where a vector assembled under one map must be read under another: features whose entity has since been deregistered are absent from the current map, features whose entity registered after the assessment are absent from the buffer and take zero, and every index in the reconstructed vector has exactly one source because every index has exactly one owner.

**Invariant (Per-block contiguity)** · `inv:dimension:contiguity`

Each physical block, each slot, and each template expansion group occupies a deterministic contiguous range of indices.

Contiguity is asserted per block and not per lifecycle scope, and the difference is the point. An identity dimension owns a per-dimension block, a run of competitive indicators, and a set of competitive interaction features, and these sit in three separate regions of the vector; the dimension's removed index set is therefore several ranges and not one. What the map must do is identify that set completely, so the removed partition can be formed either by slicing or by a transient gather (`def:gaussian:gathered-partition`). The Schur algebra requires an identifiable partition, never a physically adjacent one, which is precisely why contiguity can be demanded of blocks and not of owners.

**Invariant (Version consistency)** · `inv:dimension:version-consistency`

At every published version, the map, the model parameters, and the standardisation statistics share one dimension and one index semantics. No assessment ever reads a model whose dimension disagrees with the map it is read under.

The invariant is unconditional, and it is unconditional because there is no useful weaker form: a vector standardised under one index semantics and scored under another is not approximately right, it is meaningless position by position. Publication is atomic for exactly this reason (`def:publication:model-snapshot`) — the map and the parameters and the statistics become visible together or not at all. The obligation this places on every lifecycle path is therefore uniform and unforgiving: a path that extends the models by one count and the standardisation vectors by another has broken the invariant at the moment it publishes, and no reader downstream is in a position to notice.

**Algorithm (Compaction and rebuild)** · `alg:dimension:compaction`

After every marginalisation the affected parameters and standardisation vectors are physically compacted: removed rows and columns are excised, and every higher-indexed entry shifts down to close the gap. Extension appends at the end and disturbs nothing. The rebuilt map's ranges therefore correspond exactly to physical positions, and no translation layer stands between them.

The map is rebuilt on every registry lifecycle event and on every competitive restructuring, and on no other event. A rebuild is a total recomputation from the current registries and competitive sets; no prior map state is consulted, and the old map is used only to identify the marginalisation scope before the rebuild begins. Where one restructuring event promotes or demotes several cells, every extension and marginalisation is applied first and one rebuild follows (`req:registry:compound-events`). Rebuild cost is linear in the dimension plus the pair enumeration of any wildcard templates, which is sub-millisecond at the reference configuration and dominated by that enumeration at high Sentinel counts.

**Definition (Block, slot and feature layers)** · `def:dimension:layers`

The map resolves a position at three depths.

| Layer | Answers | Changes on |
| --- | --- | --- |
| Block | Which of the eight block types owns this range | Lifecycle events and restructuring |
| Slot | Which Sentinel, dimension, cell, or template expansion owns this sub-range | The same events |
| Feature | Which feature sits at which offset within a slot | Nothing; it is derived |

The feature layer is derived rather than stored, because the extraction layout and the per-dimension layout are fixed by specification: the map holds a slot's base address and the reader adds a known offset. Two such offsets are named and held, both slot-relative under the slot convention (`conv:extraction:offsets`). The base outcome-memory offset is the position within a slot at which the Ledger features begin, invariant at fifty-seven under every lifecycle event, since it depends only on the fixed extraction groups preceding them. The per-axis outcome-memory offsets give, for each spatially-enabled axis, where that axis's pair of Ledger features begins; axes absent from it contribute no per-Sentinel Ledger features and do not affect the slot width.

**Convention (The per-dimension axis index)** · `conv:dimension:axis-index`

Within an identity dimension's block, the three features of axis $a$ (`tab:keyspace:outcome-state`) begin at offset $8 + 3\,\text{index}(a)$, where the index is the axis's zero-based position in the outcome axis registry's order.

The registry keeps axes in registration order, and deregistration shifts every later axis down by one (`alg:registry:axis-deregistration`). That order is the authoritative definition of the index, and it is authoritative rather than merely conventional because nothing else records it: the offset is computed from the order at every use site, so a second ordering anywhere would not disagree loudly, it would silently read one axis's history as another's. The lifecycle sequence is fixed accordingly — the pre-event map identifies the indices to remove, the Schur complement is applied at those indices, and the new map is built from the post-event registry.

**Algorithm (The interaction compilation pipeline)** · `alg:dimension:compilation-pipeline`

Declared templates reach the assessment path through three stages, each serving a different consumer.

1. **Declaration.** The symbolic templates as the host wrote them: a type, and two operand selectors naming a slot feature, an aggregate feature, an identity feature, a declared signal, or the owning competitive indicator. Fixed at construction, and retained in the map so a snapshot can rebuild the later stages without the host.
2. **Resolution.** One interaction identity per expanded feature, each mapped to its output index. The identity records what the feature is *of* — which template, which Sentinel or pair, which dimension and cell — so a deregistration derives the affected indices from the entity it removes rather than by scanning the interaction range.
3. **Compilation.** A flat list of index triples, one per feature: output, first operand, second operand. This is what feature assembly iterates (`alg:runtime:assessment-pipeline`), with no name lookup and no template interpretation left in it.

Resolution and compilation are recomputed on every map rebuild; declaration is not. The three stages exist because audit wants names, lifecycle wants identities, and the assessment path wants neither.

**Definition (The competitive range and its ordering)** · `def:dimension:competitive-range`

Competitive indicators occupy the final block of the vector, after the interactions. The map holds the block's extent and a per-dimension, per-cell lookup into it that is a bijection between the cells of every competitive set and the indices of the block.

Within the block, indicators are grouped by identity dimension in registration order, and within a dimension they are ordered by time of competitive entry — the sequence number of the restructuring event that promoted the cell, with ties broken by the cell's canonical dyadic key of depth and start. Surviving cells keep their order across rebuilds. The ordering is deterministic and not spatially meaningful: it records competitive history, not the geometry of the key space. Determinism is the whole requirement, and it is a real one, because an ordering that varied between rebuilds would move learned weights between cells without any event having occurred.

**Definition (The anchor projection)** · `def:dimension:anchor-projection`

The anchor model reads a fixed fifteen-position vector (`def:risk:anchor-model`), and the projection is what holds that width fixed while the vector underneath it changes. Thirteen positions are gathered directly and two are computed.

| Positions | Source |
| --- | --- |
| 0 to 6 | The bias and aggregate blocks; always present |
| 7 to 12 | The cross-dimension block; absent, and read as zero, when no dimension is registered |
| 13 | The maximum over the four per-axis maximum z-score positions |
| 14 | The logarithm of one plus the reporting Sentinel count |

Positions 0 to 6 precede every dynamic block and their indices never move. Positions 7 to 12 move whenever the preceding per-dimension blocks change width, and are toggled between present and absent by the first and last dimension registration; the rebuild updates them. No lifecycle event ever extends or marginalises the anchor model. The projection absorbs the change instead, which is the entire mechanism by which the anchor stays a fixed model over a moving vector.

**Schema (The dimension map record)** · `schema:dimension:map-record`

The map is one record, carrying the total dimension and the structure of every layer beneath it.

| Group | Carries |
| --- | --- |
| Dimension | The total width the parameters and statistics must agree with |
| Block layer | The bias index and the extent of each of the eight blocks, per Sentinel and per dimension where the block is one of many |
| Slot layer | The uniform Sentinel slot width, and the two derived outcome-memory offsets (`def:dimension:layers`) |
| Interaction pipeline | The declarations, the resolution map, and the compiled triples (`alg:dimension:compilation-pipeline`) |
| Competitive | The per-dimension, per-cell index lookup (`def:dimension:competitive-range`) |
| Anchor | The projection (`def:dimension:anchor-projection`) |

Every map in the record is insertion-ordered, and the requirement is not a preference. Insertion order is registration order for Sentinels, axes and dimensions, and dimension-grouped entry order for competitive cells; an unordered map would satisfy covering and fail contiguity, because contiguity is a claim about a deterministic order of entries within a block. The record is authoritative at the block and slot level and deliberately not below it: positions within a slot come from the fixed extraction layout, positions within a dimension block from the axis-index convention (`conv:dimension:axis-index`), and positions within the signal block from the schema's declaration order (`req:signal:schema-fixed`).

#### Online standardisation · `sec:standardisation:online`

The division carries no material of its own. It collects why the features are standardised, where the statistics live and what feeds them, the label-time procedure, the priors a new position starts at and the rule assigning them, the two initialisation mechanisms, and the mismatch that re-standardisation costs.

**Definition (Why standardisation is needed)** · `def:standardisation:purpose`

Each position is centred and scaled by a running mean and variance before any model reads it:

$$\hat{\phi}_j = \frac{\phi_j - \bar{\mu}_j}{\sqrt{\bar{v}_j} + \varepsilon}$$

The models take an isotropic prior — one precision on every position — and an isotropic prior is a statement about scale as much as about belief. Raw, the vector mixes z-scores at natural scale near one, log-scaled features near three to six, and cumulative sums ranging over tens; one prior precision across those would be simultaneously far too tight for the large-scale positions and far too loose for the small-scale ones, and the model would spend its early labels correcting an artefact of units. Standardisation makes the single prior defensible by making the positions comparable.

**Requirement (Where the statistics live)** · `req:standardisation:timing`

The standardisation statistics are part of the published snapshot (`def:publication:model-snapshot`). An assessment acquires one immutable coordinate system, scores against it, and mutates no statistic in it. Three mechanisms feed later snapshots, and all three publish through the model owner:

1. **The cold-start prior-mass ramp** (`alg:standardisation:batch-initialisation`) replaces the cold base moments with accepted raw assessment-time vectors, in equal increments over a finite horizon.
2. **Per-Sentinel bootstrap** (`alg:standardisation:sentinel-bootstrap`) accumulates raw slot values for one new Sentinel and blends them in when complete.
3. **Exponential tracking** advances the working copy at the label-time standardisation step and nowhere else, once cold standardisation is in service.

The first two are observation-authorised. They observe on the assessment path, and that observation is a bounded enqueue or an auxiliary-accumulator write (`inv:runtime:enumerated-writes`) rather than an update to anything an outcome taught. An accepted cold observation may publish only after the assessment that supplied it has fixed its snapshot, so what it moves is a later coordinate system and never the one its own request was answered in. The tracking average is indexed by labels and carries no time-indexed decay: a deployment that stops receiving labels stops moving its coordinate system, which is the correct behaviour for a coordinate system. It is inactive while the cold ramp is waiting or transitioning.

**Algorithm (The label-time standardisation procedure)** · `alg:standardisation:label-time-procedure`

At label time, in order:

1. **Read** the current statistics from the working copy.
2. **Clip** for the statistics update only: where a position deviates from its mean by more than ten standard deviations, the clipped value is used for the update and the unclipped value for the model.
3. **Standardise** the raw features.
4. **Update the models** on the standardised vector (`alg:runtime:update-path`).
5. **Update the statistics** at rate $\gamma_\text{std} = 0.9998$, the mean toward the clipped value and the variance toward its squared deviation from the previous mean.
6. **Apply the variance floor.**
7. **Publish.**

Steps 5 to 7 — and with them the clip at step 2, which exists only to feed them — run if and only if the full-vector standardisation phase is `InService` (`alg:standardisation:batch-initialisation`). In `WaitingForInit` and `Transitioning` the label reads the published coordinate system and does not also move it: the cold observation ramp is the sole owner of full-vector standardisation until it reaches its horizon, and a label advancing the average underneath it would put a label's authority into a transition an observation's authority owns. Steps 1 to 4 run in every phase, so the models learn from the first label onward whatever the coordinate system is doing.

The order carries two decisions. Models update before the statistics, so a label is scored against the coordinate system its prediction was made in rather than one it has already moved. And the clip applies to the statistics alone: an extreme observation is evidence for the model and contamination for the running variance, and treating it as both would let one outlier widen every subsequent standardisation. The bias position is never standardised.

**Table (Feature-class priors)** · `tab:standardisation:class-priors`

Every position starts at the prior mean and variance of its class.

| Class | Prior mean | Prior variance |
| --- | --- | --- |
| Bias | — | — |
| Z-scores | 0 | 1 |
| Cumulative sums | 2 | 25 |
| Rates and fractions | 0.3 | 0.05 |
| Log-scaled | 3 | 4 |
| Binary | 0.5 | 0.25 |
| Hashed categorical | 0 | 0.25 |
| Cyclic | 0 | 0.5 |
| Interaction products | 0 | 1 |
| Occupancy indicators | 0.5 | 0.25 |
| Competitive cell indicators | 0.5 | 0.25 |
| Identity measurement alarm | 0 | 1 |
| Identity measurement suspicion | 0.1 | 0.05 |
| Identity measurement volatility | 0.1 | 0.05 |
| Cross-dimension maximum aggregates | 0 | 1 |
| Cross-dimension binary | 0.5 | 0.25 |
| Outcome axis average, compressed | 0 | 0.25 |
| Outcome axis average, raw | 0 | 1 |
| Outcome axis stability | 0.1 | 0.05 |

Nineteen classes, and the values are approximations of the settled distribution rather than guesses: a binary indicator at even odds has mean one half and variance one quarter exactly, and a z-score is defined to have mean zero and unit variance. At cold start they carry the whole of the standardisation mass; each accepted observation retires exactly $1/N_\text{init}$ of it, and at the ramp's horizon they carry none (`alg:standardisation:batch-initialisation`). Once the phase is in service the label-indexed average tracks later distribution change at the specified rate, and its half-life — some three thousand five hundred labels — describes that continuing adaptation rather than the retirement of the priors, which is finished before the first in-service label.

**Requirement (Feature-class assignment)** · `req:standardisation:class-assignment`

Every position is assigned a class, and the assignment is derived from three authorities and configured by nobody. The dimension map assigns the block and the slot; the fixed extraction layout assigns the class within a Sentinel slot; the signal schema assigns the classes of the signal block, expanded from each declared shape (`schema:signal:shapes`).

The rule is stated because the priors table is worthless without it. A table of class priors and no assignment rule leaves each new position's class to whichever code path creates it, and two paths creating the same kind of position — a lifecycle extension and a batch reset, say — would be free to disagree without contradicting anything written down. Deriving the class from the map rather than storing it also means a position cannot carry a stale class: the class is recomputed wherever the map is, so a class and an index never drift apart.

**Algorithm (Batch initialisation)** · `alg:standardisation:batch-initialisation`

Cold full-vector standardisation is a finite prior-mass ramp over the first $N_\text{init}$ accepted raw assessment vectors (`tab:config:standardisation`).

1. **Start.** Cold construction publishes the feature-class prior moments (`tab:standardisation:class-priors`), the phase `WaitingForInit`, and an accepted-observation count of zero.
2. **Offer after acquisition.** An assessment assembles its raw vector after acquiring its model snapshot, and offers that vector without blocking (`inv:guarantee:non-blocking`). It goes on to standardise and score against the snapshot it already holds.
3. **Accumulate.** The model owner advances a numerically stable empirical mean and variance over the accepted vectors, skipping non-finite entries element by element, and resets at the new width if the dimension changes.
4. **Mix.** At accepted count $n$, write $a_n = n/N_\text{init}$ for the retired share, $(\mu_{0,j}, v_{0,j})$ for the base moments and $(\mu_{e,j}, v_{e,j})$ for the empirical moments of the accepted sample. Every non-bias position publishes

   $$\bar{\mu}_j = (1 - a_n)\,\mu_{0,j} + a_n\,\mu_{e,j}$$

   $$\bar{v}_j = \max\Big(v_\text{floor},\;(1 - a_n)\,v_{0,j} + a_n\,v_{e,j} + a_n(1 - a_n)\,(\mu_{e,j} - \mu_{0,j})^2\Big)$$

   The phase is `Transitioning` while $0 < n < N_\text{init}$, and the bias position takes no part.
5. **Publish progress.** Every accepted advance publishes one immutable model snapshot, increments its snapshot version, and carries the accepted count. A command channel too full to take the observation skips it, reports that load condition in band, and increments nothing.
6. **Complete.** At $n = N_\text{init}$ the prior mass is zero, the published moments are exactly the empirical moments of the accepted sample, the phase becomes `InService`, and the completion event is emitted.
7. **Persist.** A checkpoint carries the phase, the count, the base moments and the empirical sufficient statistics, and a restore resumes the ramp at the count it stopped at.

The third term of the variance is the between-population variance, and it is published rather than dropped. A variance mixed without it would be neither the prior's nor the sample's: it would understate the spread by exactly the amount by which the two populations disagree about the mean, and it would understate it worst in the middle of the ramp, where $a_n(1 - a_n)$ is largest and where a reader has least other evidence about which coordinate system a result came from. At the horizon that term and the prior weight are both zero, so the endpoint is the empirical moments exactly — not within a tolerance, and with no rate left running underneath them.

The accumulation is a numerically stable one-pass update rather than a sum of squares, and it costs storage linear in the dimension for the length of the ramp. The reset on a dimension change is the only sound response available: a part-accumulated vector at one width says nothing about a position that did not exist when accumulation began, and blending the two would attribute one position's distribution to another's index. A lifecycle change therefore never applies an old-layout observation to a new layout — it discards the incompatible sample and rebases the ramp on what the lifecycle published (`req:standardisation:lifecycle-entries`).

Step 2's sample is best-effort under contention: the assessment path may not wait, so an observation the owner cannot take at that moment is skipped rather than waited for. What that costs is exactness of the sample and the pace of the ramp — the moments describe a scheduler-dependent subset of the earliest assessments rather than all of them, so two runs over identical traffic reach the horizon from different subsets and after differently many requests. What it does not cost is a silent gap. A refused observation is reported to the caller in band and advances no count, which is what keeps it inside (`dec:concurrency:no-silent-drop`) rather than exempt from it: under a completion gate the whole accumulation rode on one delivery and a refusal had to be retried or the sample was lost, and under the ramp there is no one-shot left to lose — a refused observation is one share of prior mass retired later, and the host is told which.

**Bound (The re-standardisation mismatch)** · `bound:standardisation:restandardisation`

The pending buffer stores raw features (`def:runtime:pending-entry`), so a label re-standardises them against a later standardisation snapshot than the one its assessment scored in. The mismatch has two components:

$$|\hat{\phi}_j^{(\text{lab})} - \hat{\phi}_j^{(\text{rec})}| \leq \underbrace{\frac{|\Delta\bar{\mu}_j|}{\sqrt{\bar{v}_j}}}_{\text{mean drift}} + \underbrace{\frac{|\phi_j - \bar{\mu}_j| \cdot |\Delta\bar{v}_j|}{2\bar{v}_j^{3/2}}}_{\text{variance drift}}$$

In `InService` the coordinate system moves under the label-indexed average. Write $d_t$ for the clipped observation's displacement and $z_t$ for the stored raw feature's displacement, both in the pre-update coordinate system at step $t$. Its path sensitivity over $\Delta k$ intervening labels is the first-order envelope

$$B_\text{disp} = \sum_{t=1}^{\Delta k}(1 - \gamma_\text{std})\left(|d_t| + \frac{|z_t|}{2}|d_t^2 - 1|\right).$$

This is a local linearisation: for a finite variance change the exact comparison uses the difference of the endpoint transforms, so the sum is not an exact finite-change inequality. With the configured $\gamma_\text{std} = 0.9998$ and clip width ten, the clip-boundary evaluation gives about $0.002\,\Delta k$ from the mean and, coarsely, $0.1\,\Delta k$ from the variance, or about 5.1 over fifty labels. As an illustrative two-standard-deviation variance evaluation, the same sensitivity gives $0.0008\,\Delta k$; retaining the global clip-boundary mean allowance gives about 0.14 over fifty labels. These are worked evaluations of the path sensitivity, not stationary-deployment tolerances or expectations.

During `Transitioning` the movement is not label-indexed at all, and the $\Delta k$ figures do not describe it. What moves the coordinate system is the finite prior-mass ramp: one accepted observation retires exactly $1/N_\text{init}$ of the base mass (`alg:standardisation:batch-initialisation`), and the phase, the accepted count and the snapshot version identify the two coordinate states a mismatch lies between (`schema:output:health-snapshot`). That is a bound on the coordinate system and not on the risk uncertainty: the standardised vector enters directional quadratic forms and a nonlinear probability transform (`def:risk:subspace-blend`), and no scalar factor carries a mass fraction through them.

The large figure needs an extreme feature value and extreme variance evolution at once, and it is bounded in effect as well as in size: the leverage bound (`prop:update:leverage-bound`) limits what any single label's mismatch can do to the posterior, in either phase.

**Requirement (Standardisation under lifecycle events)** · `req:standardisation:lifecycle-entries`

A lifecycle event that adds positions gives each new position its class prior; one that removes positions deletes their entries. The map update and the standardisation update are published together as one atomic lifecycle state change (`inv:guarantee:lifecycle-publication`). If the cold ramp is still active, that publication becomes the ramp's new base: the sufficient statistics gathered under the layout being replaced are discarded, the accepted count resets to zero, and every position that survives the change keeps the moments just published for it rather than reverting to its original prior (`alg:standardisation:batch-initialisation`). The next accepted raw vector must be assembled under the same layout it will be mixed into.

The requirement is one sentence long and it is the sentence the version invariant (`inv:dimension:version-consistency`) rests on. Every lifecycle path must add exactly as many standardisation entries as it adds model dimensions, counted the same way on both sides — per Sentinel and per identity dimension alike, for as many dimensions as are registered. A path that counts one way for the models and another for the statistics publishes a snapshot whose parts disagree, and because both counts are computed from the same registries at the same moment, the two must be derived from a single count rather than written twice.

**Algorithm (The per-Sentinel standardisation bootstrap)** · `alg:standardisation:sentinel-bootstrap`

A Sentinel registered after the system has warmed up starts with class priors that may be far from its actual distribution, and would spend a standardisation half-life — some three thousand five hundred labels — being standardised against them (`cav:limitation:late-standardisation`). The bootstrap shortens that to a hundred assessments.

1. **Open** a per-Sentinel accumulator of capacity one hundred on registration.
2. **Accumulate** the new Sentinel's slot values, occupancy included, from each assessment it reports in, skipping non-finite entries and resetting at the new width if the slot width changes.
3. **Blend** the empirical statistics into the Sentinel's standardisation entries at weight $\alpha_\text{boot} = 0.8$, leaving the remaining fifth on the priors.
4. **Release** the accumulator.

The blend is soft rather than a replacement because a hundred assessments is enough to locate a distribution and not enough to characterise its tail; the retained fifth of the prior is what keeps an unlucky first hundred from fixing a variance the Sentinel will spend the next half-life escaping.

## Part (The Core Models) · `part:spec:core-models`

Part III defines what the Core learns. Three Bayesian linear models estimate risk from the feature vector and one model per declared outcome axis predicts whatever the host asked to be predicted, all of them updated by a single observation procedure; beneath them a per-Sentinel outcome memory accumulates what happened where. Nothing here references actions, channels, posture, or cost: the Part ends at the risk basis, and what a host does with a risk basis is the next Part's subject.

### Chapter (The Core Risk Models) · `chap:spec:core-risk-models`

The chapter is the densest in the document. Three models share one prior and one update procedure and differ in what they are allowed to learn from; two of them are combined into an effective estimate whose weight is a ratio of uncertainties; the estimate is mapped to a probability by a calibration fitted against outcomes. Around that core sit the three policies that decide which observation reaches which model and with what force — the eligibility table, the importance weights, and the leverage bound — and the forgetting rates that decide how long any of it is remembered.

**Definition (The model triple)** · `def:risk:model-triple`

Three Bayesian linear models estimate risk from the feature vector $\phi$. The operational model trains on every labelled outcome, whatever the host did; the sister model trains only on unconfounded outcomes; the counterfactual anchor model trains on the same unconfounded outcomes as the sister but over a fixed low-dimensional projection.

$$\hat{\rho}_\text{opr}(\phi) = \phi^\top \mu_\text{opr}, \qquad \hat{\rho}_\text{inh}(\phi) = \phi^\top \mu_\text{inh}, \qquad \hat{\rho}_\text{anc}(\tilde{\phi}) = \tilde{\phi}^\top \mu_\text{anc}$$

All three are initialised at $\mu = \mathbf{0}$, $B = \lambda_\text{prior} I$, $\Sigma = \lambda_\text{prior}^{-1} I$, at one shared prior precision.

The triple is the direct consequence of the valence asymmetry (`assum:constraint:valence-asymmetry`). The operational model sees everything and therefore learns a confounded picture: its estimate is the joint effect of inherent risk and of the host's own intervention. The sister model sees only what was left alone and estimates inherent risk, but starves whenever the host restricts. The anchor converges some forty times faster on fifteen parameters and covers the sister's starvation coarsely rather than well. Each is individually insufficient and the three are jointly what the decomposition principle asks for (`prin:principle:multi-level-decomposition`).

The sister's target deserves one qualification. Selection into the unconfounded population is itself a function of the system's own estimate, so the sister estimates the adverse rate conditional on having been allowed, which equals inherent risk only where eligible labels actually reach. In regions the host has always restricted, the sister's estimate is prior extrapolation and not measurement, and host investigation (`conv:eligibility:ground-truth`) is the only mechanism that supplies coverage there.

**Definition (The anchor model)** · `def:risk:anchor-model`

The anchor operates on a fixed fifteen-dimensional vector $\tilde{\phi}$ drawn from the bias, the aggregate block, the cross-dimension identity features, and two quantities computed inline. It is never extended and never marginalised, whatever Sentinel, outcome axis, or identity dimension lifecycle events occur; the projection that feeds it absorbs every such change (`def:dimension:anchor-projection`).

| Index | Feature | Block |
| --- | --- | --- |
| 0 | Bias | Bias |
| 1 | Maximum cell-level maximum z | Aggregate |
| 2 | Standard deviation of cell-level maximum z | Aggregate |
| 3 | Cross-axis product | Aggregate |
| 4 | Axis breadth | Aggregate |
| 5 | Coverage | Aggregate |
| 6 | Maximum cumulative sum | Aggregate |
| 7 | Maximum suspicion | Cross-dimension |
| 8 | Maximum adverse rate | Cross-dimension |
| 9 | Any competitive cell present | Cross-dimension |
| 10 | Maximum compressed valence magnitude | Cross-dimension |
| 11 | Maximum raw valence magnitude | Cross-dimension |
| 12 | Maximum volatility | Cross-dimension |
| 13 | Maximum raw z across Sentinels and axes | Computed |
| 14 | Log count of reporting Sentinels | Computed |

Positions 0 to 12 are gathered from $\phi$; positions 7 to 12 evaluate to zero when no identity dimension is registered, which is how identity lifecycle change is absorbed without touching the model. Positions 13 and 14 are computed inline rather than occupying two further aggregate indices, which is what keeps the aggregate block at its declared width (`tab:feature:aggregate-block`). The cross-dimension aggregation is a maximum throughout, so the anchor fires when any identity view is alarming — the principle the aggregate block already applies across Sentinels.

The projection gathers all fifteen positions exactly as tabulated: positions 0 to 12 through the dimension map, then the computed maximum raw z-score and the logarithm of the reporting-Sentinel count at positions 13 and 14.

**Proposition (The measurement-only anchor feature)** · `prop:risk:anchor-measurement-only`

Anchor position 13 — the maximum raw z-score across every Sentinel and axis — is a pure measurement signal that reaches the model without passing through any outcome memory. It is therefore evidence of a kind no Ledger feature and no identity outcome feature can supply: it reports what the measurement layer sees now, and nothing about what was recorded earlier.

The property is what makes the anchor useful under contamination. When stale outcome memory inflates the identity and cell-level features (`alg:valence:contamination-loop`), position 13 does not move with them, so the anchor can distinguish an alarming measurement layer from a silent one that sits behind a remembered reputation. Position 14, the log count of reporting Sentinels, carries measurement depth alongside it, letting the anchor discount its own estimate when little of the surface is online.

The property is a statement about provenance, not about magnitude. One feature of fifteen cannot outvote a contaminated majority, and it is not asked to: what it supplies is a persistent discrepancy that a drift monitor can accumulate even while the contaminated estimate is the one being reported.

**Definition (The subspace-restricted blend)** · `def:risk:subspace-blend`

The sister and anchor are blended into an effective estimate whose weight is set by their relative uncertainty over the features they share. Let $\tilde{\phi}_g$ be the gathered anchor coordinates — positions 0 to 12, the ones that correspond to an index of $\phi$ — and let $\Sigma_{\text{inh},S_gS_g}$ be the sister covariance restricted to them:

$$\sigma^2_{\text{inh},S_g}(\phi) = \tilde{\phi}_g^\top \Sigma_{\text{inh},S_gS_g} \, \tilde{\phi}_g, \qquad w(\phi) = \left(1 - \frac{\sigma^2_\text{anc}(\tilde{\phi})}{\sigma^2_{\text{inh},S_g}(\phi) + \varepsilon}\right)^+$$

$$\hat{\rho}_\text{eff}(\phi) = (1 - w) \cdot \hat{\rho}_\text{inh}(\phi) + w \cdot \hat{\rho}_\text{anc}(\tilde{\phi})$$

The restriction to the shared subspace is the point of the construction. The sister's uncertainty about Sentinel-specific directions has nothing to do with whether the anchor is the better estimator over the features both models hold, and a weight computed from the sister's full uncertainty would activate the anchor every time a new Sentinel arrived. The two computed anchor positions are excluded from the sister-side covariance because they correspond to no index of $\phi$ and could enter only through an explicit linearisation.

All three uncertainties are read from the time-corrected covariance (`def:runtime:time-correction`), so the weight moves with elapsed time as well as with labels. The point estimates do not.

**Remark (How the blend persists)** · `rem:risk:blend-mechanisms`

The weight does not decay to zero. Two structural mechanisms keep the sister's shared-subspace uncertainty above the anchor's in steady state.

The first is the leverage bound differential. The sister's higher dimension makes its leverage larger, so the bound (`prop:update:leverage-bound`) caps its effective weight more often and more severely than the anchor's, and its precision over the shared subspace accumulates more slowly for that reason alone.

The second is marginal covariance inflation. The sister's marginal covariance over the shared features exceeds the inverse of the corresponding precision submatrix, because correlations between anchor features and Sentinel-specific features inflate the margin through the Schur complement structure (`def:gaussian:schur-complement`).

Both persist. The weight converges to a small positive residual — two to ten per cent, depending on how correlated the deployment's features are — so the sister contributes the great majority of the estimate and the anchor keeps a floor under it. Early in convergence the trajectory runs the other way: both models sit at the shared prior, the anchor's quadratic form spans two extra computed directions, and the weight starts near zero, rises steeply as the anchor's fifteen dimensions concentrate, then declines toward its residual as the sister catches up. A Sentinel deregistration can push it transiently to zero again, when the marginalisation briefly raises the sister's shared-subspace precision above the anchor's.

**Proposition (The blended uncertainty)** · `prop:risk:blend-variance`

The blended uncertainty is the variance of the two-component mixture the weight defines:

$$\sigma^2_\text{eff}(\phi) = (1-w)\sigma^2_\text{inh}(\phi) + w\sigma^2_\text{anc}(\tilde{\phi}) + w(1-w)\bigl(\hat{\rho}_\text{inh}(\phi) - \hat{\rho}_\text{anc}(\tilde{\phi})\bigr)^2$$

The first two terms carry within-model uncertainty and the third carries between-model uncertainty: the extra variance from not knowing which of the two is right. The third term is what makes anchor–sister disagreement visible in the reported uncertainty rather than silently averaged away, and it is the term that vanishes at both extremes of the weight.

Two qualifications hold the formula to what it claims. The weight is a heuristic precision ratio and not a posterior model probability, so the mixture-variance form is the correct bookkeeping given that weight rather than a claim of Bayesian model averaging. And the first term uses the sister's *full* uncertainty, not its shared-subspace restriction: the restriction applies to the weight alone, because uncertainty about Sentinel-specific directions is real uncertainty about the estimate even when it says nothing about which model to prefer.

**Decision (Why the floor is emergent rather than declared)** · `dec:risk:anchor-floor`

The persistent residual weight is a consequence of the mathematics, not a designed feature, and the question is whether to codify it as an explicit floor $w \geq w_\text{floor}$. It is not codified, and the reasoning is recorded here because the behaviour it protects is easy to lose by accident.

The residual supplies three properties. It keeps a measurement-only contribution in every assessment, too small to override a contaminated estimate but large enough to leave a persistent discrepancy for a drift monitor (`prop:risk:anchor-measurement-only`). It keeps the between-model variance term alive, so a regime change that invalidates the sister's weights shows up as disagreement rather than as unwarranted confidence. And it responds to lifecycle events without being told about them: a new Sentinel raises the sister's leverage, the bound fires harder, the weight rises, and the anchor covers the re-convergence with no lifecycle-aware scheduling anywhere.

An explicit floor would replicate all three and would add a parameter to a regime that already behaves correctly. The emergent floor is preferred because its magnitude scales with the deployment's own feature structure, which a configured constant cannot do. What the deployment gives up is the guarantee that the floor is there at all (`cav:limitation:anchor-floor`).

**Definition (The risk probability)** · `def:risk:probability`

The effective estimate is mapped to a probability through a sigmoid whose steepness is the calibration parameter:

$$\hat{p} = \sigma\!\left(\frac{\hat{\rho}_\text{eff}}{\kappa_\text{eff}}\right), \qquad \sigma_{\hat{p}} = \hat{p}(1-\hat{p}) \cdot \frac{\sigma_\text{eff}}{\kappa_\text{eff}}$$

where $\kappa_\text{eff}$ is the effective calibration parameter (`def:platt:regimes`), $\sigma_\text{eff}$ the square root of the blended variance, and $\sigma(x) = (1 + e^{-x})^{-1}$.

The uncertainty on the probability follows from the sigmoid's derivative identity and is a first-order propagation, not an exact transformation of the posterior. The $\sigma_\text{eff}$ it propagates is evidence-only along the request's own direction, so the widening is already in the figure the host reads rather than left for a consumer to apply (`dec:risk:evidence-only-uncertainty`). It is reported because a probability without one is not usable by a decision layer that must weigh a confident estimate differently from a tentative one, and the propagation is accurate wherever the estimate is not close to saturation.

**Decision (The verdict's uncertainty is evidence-only along its own direction, and the share it borrowed rides beside it)** · `dec:risk:evidence-only-uncertainty`

The uncertainty reported with a verdict is the uncertainty the evidence supports along that request's own features, and the share of the precision behind it that the models' floors were holding up instead travels beside it on the basis (`schema:risk:basis`). Degradation is a source of uncertainty and it has an angle: a model propped up by its floors is not propped up equally in every direction, and a request asks about one. An aggregate says the model is weak somewhere; only a reading taken along the request's own direction can say whether the weakness is in the way of this answer.

The reading the tempering divides by is the share the posterior already tracks. The precision matrix carries the two floors' contributions as mass — identity-shaped from the spectral floor, coordinate-shaped from the replenishment clamp, both decayed with the matrix they are part of — so along any direction $\phi$ the share of posterior precision the prior rather than the evidence is holding up is $s(\phi) = (f\lVert\phi\rVert^2 + \sum_j c_j \phi_j^2) / (\phi^\mathsf{T} B \phi)$ (`dec:posterior:spectral-floor`). It is exact about the split, because the two masses are the amounts the floors added to $B$ in those two shapes rather than estimates of them, and it is an approximation of the full posterior, because a Rayleigh quotient reads one direction and says nothing about the subspace that direction lies in. Both halves of that are the mass record's and are not re-argued here.

The tempering follows by subtraction rather than by a model. Along $\phi$ the posterior precision is the evidence's plus the floors', so the evidence's alone is $(1-s)$ times the posterior's, and the variance that goes with it is the posterior variance divided by $(1-s)$. Nothing is fitted, and no constant is chosen.

Where the correction is exact and where it is a bound is stated rather than left to be discovered. Let $D = fI + \operatorname{diag}(c)$ be the floors' own contribution, so the exact evidence-only variance along $\phi$ is $\phi^\mathsf{T}(B-D)^{-1}\phi$. Writing $z = D^{1/2}\Sigma\phi$ and $M = D^{1/2}\Sigma D^{1/2}$, the reported value expands as $v + z^\mathsf{T}z + (z^\mathsf{T}z)^2/v + \cdots$ and the exact one as $v + z^\mathsf{T}z + z^\mathsf{T}Mz + \cdots$, and the two agree in their first two terms. From there Cauchy–Schwarz in the inner product $\Sigma$ defines gives $(z^\mathsf{T}z)^2 \leq v \cdot z^\mathsf{T}Mz$, and the log-convexity of the moments $z^\mathsf{T}M^k z$ carries that to every later term by induction. So the reported variance is a lower bound on the exact evidence-only variance, with equality wherever the floors' share is even across the coordinates the direction draws its variance from — one coordinate, or a model whose floors bear on every coordinate alike. The correction therefore understates the widening where the bound is loose, which is the direction that under-reports uncertainty and is named here rather than buried: it is the price of a scalar correction along one direction, and the alternative is a matrix inverse per request. The mass the share is computed from errs the other way, crediting a late-arriving coordinate with identity-shaped mass never applied to it (`dec:posterior:spectral-floor`), and the two errors are recorded as opposing rather than as cancelling, because nothing bounds their ratio.

The share is computed from the published covariance, and that is an identity rather than a substitution. The precision matrix is not published, on an argument about bulk that these masses do not engage (`dec:retention:precision-excluded`). But with $\psi = \Sigma\phi$ the denominator collapses: $\psi^\mathsf{T}B\psi = \phi^\mathsf{T}\Sigma B \Sigma \phi = \phi^\mathsf{T}\Sigma\phi$, which is the variance the blend has already computed. So the share is read along $\psi$, and $\psi$ is not an arbitrary stand-in for $\phi$ — it is the direction whose precision holds that variance up, which is the direction the tempering is about. The two coincide wherever $\phi$ is an eigenvector of $B$, because $\Sigma\phi$ is then parallel to $\phi$ and the share is scale-free. What the identity rests on is that the pair the model holds are actually inverses of one another, which is a property the model maintains rather than assumes: a rebuild is adopted on the drift measured after it and a shifted answer is refused by that measurement (`dec:posterior:measured-adoption`).

The blend's share is the same weights in variance space. The blended variance is a three-term mixture (`prop:risk:blend-variance`), and the borrowed part of it is the borrowed part of each component variance weighted as that variance was weighted: $s_\text{eff} = \bigl[(1-w)\,s_\text{inh}\,\sigma^2_\text{inh} + w\,s_\text{anc}\,\sigma^2_\text{anc}\bigr] / \sigma^2_\text{eff}$. Defining it in variance space rather than by averaging the two shares is what makes the tempering apply consistently to the quantity it is applied to, and it settles the mixture's third term without a separate rule: the disagreement between two models is uncertainty their evidence produced, no part of it was lent by a floor, and so it enters the denominator alone and dilutes the share. The subspace restriction the weight is computed over changes nothing here (`def:risk:subspace-blend`): the restriction decides which model to prefer, and each model's share is read over the coordinates that model actually has.

Each model's share is read from its raw covariance, and the staleness correction commutes with the tempering for that reason. Elapsed time inflates a variance; it moves no mass into or out of a precision matrix. The correction therefore multiplies the evidence-only variance exactly as it multiplies the posterior one (`def:runtime:time-correction`), so applying the tempering before or after it gives the same number — which is what lets the ordinary path and the fourth numeric checkpoint's fallback carry one operation in one order rather than two orders that happen to agree.

The widening saturates at the prior-only variance, and the cap is derived rather than chosen. As the share approaches one the quotient diverges, and what must be reported there is not a large number but a statement: the prior is all that is known along this direction. That statement takes the time-corrected prior variance the fourth numeric checkpoint substitutes when the blend's own variance is unusable (`def:axis:prior-only-prediction`), so the cap is that value and not a tolerance standing beside it, and one expression computes both. The ceiling is taken as the larger of the prior-only variance and the variance being capped, so a blended variance already above prior-only, which the disagreement term can produce, is not narrowed by a bound that exists to limit a widening. The tempering is a widening or it is nothing.

Only the share travels, not a second copy of the uncertainty. A host that wants the untempered figure recovers it by multiplying by $\sqrt{1-s}$, except where the tempering saturated, and the share is the thing a host would need anyway to apply a correction of its own. Two fields carrying one quantity in two states is a consistency obligation on every later edit, and the second field would have no reader that the first plus the share does not already have.

The calibration refit and this correction agree rather than double count, and the argument is what they each correct. The calibration buffer records the probability-space uncertainty as reported (`constr:platt:buffer`), including the tempered one, and the empirical-coverage refit — which measures standardised residuals against the reported intervals and inflates when they were too narrow (`alg:monitoring:empirical-coverage`) — sees wider intervals from degraded models and inflates less on their account. That is the correct interaction. The share corrects what the model can know about itself: exactly how much of the precision along this direction it lent itself, a quantity it tracks. The refit corrects what the model cannot know about itself: whether its uncertainties, borrowed mass and all, actually cover outcomes at the rate they claim. Removing the first would leave the second to discover the same deficit slowly, from outcomes, and to apply it as one factor across every direction alike — which is the aggregate reading this decision exists to replace.

One downstream consequence is recorded rather than decided here. Label guidance ranks pending entries by the uncertainty they carry (`sec:guidance:core`), and that uncertainty is now the tempered one, so guidance leans toward requests whose verdicts rested on borrowed precision. That is the direction the exploration signal was already reaching in — toward what the model knows least — read at the resolution a per-direction share provides rather than at the model's average condition. Nothing about what guidance exposes changes, so the opacity that surface owns is untouched (`dec:surface:guidance-opacity`): the ranking moves, the disclosure does not.

The field is named for what is measured. The floor share is a different quantity: the fraction of a model's least eigenvalue the spectral floor is carrying, one reading per model, taken along whichever direction the model is weakest in (`dec:health:tiered-queries`). This one is taken along whichever direction was asked about and counts both floors. It is called the borrowed share because that is what it is a share of — precision the model lent itself — and because "prior share" would overclaim: the prior precision a model is constructed at is not tracked as mass and is not counted here.

**Definition (Intervention effectiveness)** · `def:risk:intervention-effectiveness`

The divergence between the blended unconfounded estimate and the operational model's realised estimate is reported with every assessment:

$$\Delta(\phi) = \hat{\rho}_\text{eff}(\phi) - \hat{\rho}_\text{opr}(\phi)$$

A positive divergence means the unconfounded estimate exceeds the realised one: the host's interventions are suppressing outcomes that would otherwise occur, which is what an intervention is for. A divergence near zero means they are either unnecessary, because inherent risk is low, or ineffective, because adverse outcomes are happening anyway. The measurement does not distinguish those two, and no quantity available to the Core does.

Two qualifications travel with the number. The blended estimate carries a small persistent anchor contribution, so where anchor and sister substantially disagree the divergence mixes intervention effectiveness with model disagreement. And it inherits the sister's scope: it is interpretable only in regions with unconfounded coverage, and is prior extrapolation elsewhere.

**Schema (The risk basis)** · `schema:risk:basis`

The risk basis is the Core's complete risk output and the only Core state a derivation reads.

| Field | Carries |
| --- | --- |
| Effective estimate $\hat{\rho}_\text{eff}$ | The blended unconfounded risk estimate |
| Effective uncertainty $\sigma_\text{eff}$ | The blended standard deviation, evidence-only along this request's direction (`dec:risk:evidence-only-uncertainty`) |
| Calibration parameter $\kappa_\text{eff}$ | The sigmoid steepness at this blend weight |
| Risk probability $\hat{p}$ | The calibrated probability |
| Probability uncertainty $\sigma_{\hat{p}}$ | The propagated standard deviation, carrying the same widening |
| Borrowed share $s$ | How much of the precision behind that uncertainty the models' floors were holding up, in $[0,1]$ (`dec:risk:evidence-only-uncertainty`) |
| Operational estimate $\hat{\rho}_\text{opr}$ | The realised-risk estimate |
| Intervention effectiveness $\Delta$ | The divergence of the two |
| Blend weight $w$ | The anchor's share of the estimate |
| Sister uncertainty $\sigma^2_\text{inh}$ | The sister's own variance |
| Anchor uncertainty $\sigma^2_\text{anc}$ | The anchor's own variance |
| Sister regime record count | Weighted samples behind $\kappa_\text{sister}$ |
| Anchor regime record count | Weighted samples behind $\kappa_\text{anchor}$ |
| Label count | Labels processed at assessment time |

Fourteen fields, and the boundary they draw is the architectural one: a derivation given this and nothing else can compute a complete decision landscape, which is what makes the derivation replaceable and the Core decision-free (`prin:principle:measure-not-decide`). The borrowed share is present for the same kind of reason one step further in: the uncertainty beside it has already been widened by it, so a host that wants the untempered figure, or a correction of its own, needs the number the widening was computed from rather than a second copy of the uncertainty in its earlier state. The two regime counts are present so that a host can treat an assessment as preliminary while the calibration is still thin rather than discovering later that it was; which labels were admitted at all is the neighbouring question, and it is governed by the host's declared eligibility policy (`req:host:eligibility-policy`). The basis crosses to the derivation (`sig:landscape:derivation-function`) and is reported to the host within the assessment (`schema:output:assessment`).

#### Platt calibration · `sec:platt:calibration`

The division carries no material of its own. It collects the purpose of the calibration parameter, the two regimes and their soft transition, the fitting objective and the buffer it reads, the search that minimises it, the sample floor below which it does not run, the cadence at which it does, the initial value, the drift coupling, and what a regime transition does during convergence.

**Motivation (What the calibration parameter is for)** · `mot:platt:purpose`

The raw estimate is directionally correct and arbitrarily scaled. A higher estimate means higher risk, but nothing in the construction fixes what estimate corresponds to what probability, because the target the models train on is a sign and the feature scale is set by standardisation rather than by any outcome. A sigmoid applied to an arbitrarily scaled score produces an arbitrarily scaled probability.

The calibration parameter fixes the scale against outcomes. It is one number per regime, fitted so that among all requests receiving a stated probability, about that fraction actually turn out positive. That is the whole of the claim: calibration makes the probability mean what it says, and does not make the ranking better. A model that discriminates poorly and is perfectly calibrated still discriminates poorly, which is why calibration health and discrimination health are monitored separately.

**Definition (The two regimes)** · `def:platt:regimes`

The sister and the anchor relate scores to probabilities differently, because one reads fine-grained per-Sentinel features and the other coarse aggregates. Two parameters are fitted rather than one, and the effective parameter interpolates between them by blend weight:

$$\kappa_\text{eff}(w) = (1 - g(w)) \cdot \kappa_\text{sister} + g(w) \cdot \kappa_\text{anchor}, \qquad g(w) = \sigma(s_\kappa (w - 0.3))$$

The regime boundary sits at a blend weight of $0.3$ — sister-dominated below, anchor-dominated at or above — and the transition slope is $s_\kappa = 20$.

The transition is soft for a reason that a hard threshold would violate immediately. The blend weight moves continuously with every assessment, and a step change in the calibration parameter at the boundary would produce a discontinuity in the reported probability for two assessments whose underlying estimates differ imperceptibly. At a slope of twenty the interpolation is effectively complete within about a tenth of the weight either side of the boundary, which is narrow enough to keep the regimes distinct and wide enough to keep the probability continuous.

**Definition (The fitting objective)** · `def:platt:objective`

Each regime's parameter minimises a weighted cross-entropy over the calibration buffer, against the binary target $y = \mathbb{1}[v > 0]$:

$$\kappa_R^* = \arg\min_{\kappa > 0}\; -\sum_{i \in R} w_{\text{imp},i} \cdot \gamma_\text{cal}^{|R| - \text{rank}(i)} \cdot \bigl[y_i \log \sigma(\hat{\rho}_i / \kappa) + (1 - y_i) \log(1 - \sigma(\hat{\rho}_i / \kappa))\bigr]$$

Two weights multiply each record. The importance weight is the same one the model update used (`def:weighting:balancing-weights`), so the calibration is fitted against the class balance the model was trained under rather than against the raw one. The recency factor discounts older records geometrically by rank, so the fit tracks the current score distribution rather than the whole history the buffer happens to hold.

Cross-entropy rather than a squared error because the quantity being fitted is a probability, and cross-entropy is the proper scoring rule that a calibrated probability minimises in expectation. Fitting a squared error would produce a parameter that is optimal for a different question than the one the probability answers.

**Construction (The calibration buffer)** · `constr:platt:buffer`

The calibration reads a fixed-capacity circular buffer of $N_\text{cal,buf}$ records, two thousand by default. Each record is written at label time and carries what both regime fits need.

| Field | Carries |
| --- | --- |
| Effective estimate | The blended estimate at assessment time |
| Effective uncertainty | The blended raw uncertainty at assessment time |
| Blend weight | The anchor weight at assessment time |
| Binary outcome | Whether valence was positive, at label time |
| Importance weight | The weight the update applied, at label time |
| Label index | The global label counter |
| Axis predictions | One prediction per active axis, at assessment time |

Every record contributes to *both* regime fits, weighted by its blend contribution: a record at weight $w$ enters the sister fit at $(1 - g(w)) \cdot w_\text{imp}$ and the anchor fit at $g(w) \cdot w_\text{imp}$. A record deep in one regime contributes almost nothing to the other; a record near the boundary contributes substantially to both. A regime's effective sample count is the sum of its weights, and it is that weighted count the sample floor is read against, not a count of records.

The stored uncertainty is not used by the fit. It is there so that the realised coverage of the reported intervals can be measured after the fact (`alg:monitoring:empirical-coverage`), at a cost of eight bytes per record.

**Algorithm (Fitting the calibration parameter)** · `alg:platt:fitting`

The objective is minimised by golden-section search over the logarithm of the parameter.

1. **Bracket** the search on $\log \kappa$ over $[\log \kappa_\text{min}, \log \kappa_\text{max}]$, with $\kappa_\text{min} = 0.01$ and $\kappa_\text{max} = 100$ by default.
2. **Contract** the bracket by golden section until it is narrower than $10^{-4}$ in $\log \kappa$, or until fifty iterations have run.
3. **Publish** the resulting parameter for that regime.

Searching in the logarithm rather than the parameter is what makes a single tolerance meaningful across two orders of magnitude: the parameter is a scale, and a fixed absolute tolerance would be far too coarse at the bottom of the range and far too fine at the top. Golden section rather than a gradient method because the objective is one-dimensional and cheap, and a derivative-free bracket that cannot overshoot is worth more here than asymptotic speed. The cost is fifty passes over the buffer — a hundred thousand operations at default capacity, comfortably under a millisecond, and small enough that the refit cadence can be driven by correctness rather than by budget.

**Requirement (The minimum sample floor)** · `req:platt:minimum-samples`

A regime's parameter is refitted only when its weighted partition of the buffer holds at least $N_\text{cal,min}$ records — thirty by default — with at least $n_\text{cal,pos}$ positive outcomes and $n_\text{cal,neg}$ negative outcomes, three of each by default. Below any of the three floors the regime keeps the parameter it has.

Both floors are needed and neither substitutes for the other. A fit against thirty records that are all negative has no information about where the sigmoid should steepen and would drive the parameter to a bound; a fit against three records of each class has the classes but not the resolution. The floors are low deliberately: they are there to prevent a degenerate fit, not to certify a good one, and a host that treats the floor as a maturity signal has misread it. The two regime counts travel in the risk basis (`schema:risk:basis`) precisely so that the distinction between "fitted" and "fitted well" stays visible to whoever is consuming the probability.

**Table (Refitting cadence)** · `tab:platt:refit-cadence`

Five triggers refit a regime whose sample floor is met.

| Trigger | Condition |
| --- | --- |
| Periodic | Every $N_\text{refit}$ labels, two hundred by default |
| Lifecycle | After any Sentinel or outcome axis registration or deregistration |
| Drift | After any prediction drift accumulator reset |
| Anchor reactivation | The anchor regime meets its floor and more than $N_\text{refit}$ labels have passed since its last fit |
| Manual | A host-initiated recalibration request |

The five are not interchangeable. The periodic trigger tracks slow change; the lifecycle trigger exists because a dimension change alters the score distribution immediately and without warning; the drift trigger couples calibration to the monitor that noticed the scores moved; the reactivation trigger exists because a frozen regime is stale in a way no other trigger would notice (`disc:platt:regime-transition`); and the manual trigger is the host's escape hatch when it knows something the Core cannot see. A deployment that fires only the periodic trigger is calibrated on average and miscalibrated exactly when something changed.

**Definition (The initial calibration parameter)** · `def:platt:initial-value`

Both regimes start at $\kappa_0 = 1.0$.

The value matters less than it appears to. Before labels exist the estimate is near zero, so the probability is near one half whatever the parameter is, and the first refit at around two hundred labels replaces the initial value with one fitted against actual outcomes. What the choice fixes is the behaviour in the window between the first non-trivial estimates and the first fit, and unity is the neutral choice there: it neither compresses nor expands the raw score, so the probability reported in that window is the sigmoid of the score itself and is at least monotone in the quantity it claims to represent.

**Algorithm (Coupling the calibration to drift)** · `alg:platt:drift-integration`

A refit that moves the parameter substantially is itself evidence that the score distribution has changed, and the drift machinery is told so.

1. **Compute**, after each refit, the log-ratio $\delta_\text{cal} = |\log \kappa_\text{new} - \log \kappa_\text{old}|$.
2. **Compare** it against the threshold $\delta_\text{cal} > 0.1$, about a ten per cent change in the parameter.
3. **Reset**, when the threshold is exceeded, the prediction drift accumulators of the affected models, record the event, and report it in the health snapshot (`schema:output:health-snapshot`).

The reset is the point of the coupling. A drift accumulator measures the divergence between predictions and outcomes against a fixed mapping from score to probability; when that mapping is refitted, the accumulated divergence was computed under a mapping that no longer applies, and carrying it forward would attribute the calibration's own correction to the model's drift. Resetting discards genuine evidence along with the stale evidence, which is the accepted cost: a false drift alarm caused by the system's own recalibration is worse than a delayed true one.

**Discussion (Regime transition during convergence)** · `disc:platt:regime-transition`

Convergence runs the blend weight from anchor-dominated to sister-dominated, and that passage produces four calibration effects that are expected and self-correcting rather than faults.

The first fit is unstable. The leverage bound fires on a third to two thirds of early labels (`data:gaussian:binding-frequency`), so the score distribution at two hundred labels is not the steady-state one; the second fit may therefore move the parameter enough to trip the drift threshold. That is a calibration transient, and a threshold crossing at the second refit during convergence is not grounds for investigation (`cav:limitation:pre-calibration`).

The anchor regime then starves. Once the sister converges, most records sit at a weight far below the boundary, the anchor's weighted count falls under its floor, and its parameter freezes. A frozen parameter is harmless while the sister dominates and stale the moment the anchor reactivates — after a Sentinel registration, say — which is why reactivation is a refit trigger of its own and why the frozen state and the labels since the last anchor fit are both reported.

Boundary records cross-contaminate. During the transition many records sit near the boundary and contribute substantially to both fits, pulling each regime's parameter toward the other's. The periodic refit corrects this iteratively, and the first few refits during the transition may oscillate before settling. No mechanism change is warranted: the oscillation is bounded by the same soft transition that causes it, and it ends when the weight distribution stops moving.

#### Training eligibility · `sec:eligibility:training`

**Table (Training eligibility)** · `tab:eligibility:training`

Which observation trains which model is the specification's causal-inference contract, and it is a table rather than a predicate because every row is a separate judgement about confounding.

| Action taken | Outcome | Sister and anchor train | Operational trains | Why |
| --- | --- | --- | --- | --- |
| Allow | Any | Yes | Yes | Unconfounded |
| Challenge | Passed, then any outcome | Yes | Yes | The request proceeded |
| Challenge | Failed | Configurable, included by default | Yes | Policy-governed (`req:host:eligibility-policy`) |
| Slow | Any | No | Yes | Causally confounded |
| Block | Any | No | Yes | Causally confounded |
| Any | Host-investigated | Yes | Yes | Ground truth (`conv:eligibility:ground-truth`) |

The sister and anchor train only on unconfounded outcomes; the operational model trains on every labelled outcome. Slow is confounded and must not train the sister or the anchor: the host slowed the request, the outcome that followed is the outcome of a slowed request, and admitting it teaches the inherent-risk estimate about the host's own behaviour. A failed Challenge does train the sister by default, because the default policy asserts that failing a challenge is a property of the request rather than an effect of having been challenged; a deployment that disputes that sets the policy false and submits investigated challenge outcomes as ground truth instead.

The action recorded here is observed host behaviour, not a recommendation. The same four names appear in the decision landscape as candidate actions, and the two readings must not be conflated: this table asks whether an observation was confounded, and a landscape asks what to do next. The predicate follows the table: Allow is eligible, Slow and Block require ground truth, and Challenge follows the declared policy, with ground truth overriding every action.

**Convention (The ground-truth flag)** · `conv:eligibility:ground-truth`

A label may assert that its outcome was determined by investigation rather than inferred from what the host did. The flag is the host's assertion of exactly that, and it makes the label eligible whatever action was taken.

The flag is the only mechanism that supplies unconfounded coverage in regions the host always restricts, and those regions are precisely where the sister model would otherwise never learn anything. Without it the censored bandit problem is closed: the host restricts where risk is estimated high, the sister sees nothing there, and the estimate is never corrected by evidence.

The Core trusts the flag unconditionally and has no mechanism to verify it. An investigation pipeline that systematically mislabels outcomes corrupts the sister model directly and silently, and no Core-side diagnostic distinguishes a corrupted pipeline from a genuine change in the environment (`cav:limitation:investigation-pipeline`). The flag is a contract discharged by the host, and its integrity is the host's to maintain.

#### Importance weighting · `sec:weighting:importance`

The division carries no material of its own. It collects the two class-rate trackers, the update they run, their initial value, why they carry no time-indexed decay, the weights derived from them, and how those weights interact with the leverage bound.

**Table (The class-rate trackers)** · `tab:weighting:trackers`

The risk target is binary and positive-valence events are typically much rarer than negative ones, so an unweighted posterior gradient is dominated by the majority class. Two trackers estimate the positive-valence rate, and the weights are derived from them.

| Tracker | Updated on | Feeds | Decay rate |
| --- | --- | --- | --- |
| Global | Every label | The operational model | $\gamma_\text{opr}$ |
| Eligible | Eligible labels only (`tab:eligibility:training`) | Sister, anchor, axis models, and the Ledger's neutral reference | $\gamma_\text{inh}$ |

Each tracker decays at the forgetting rate of the models it serves, so the class balance it reports covers the same effective horizon as the evidence those models retain. The matching is the whole design. A tracker faster than its model over-reacts to transient fluctuations in class balance and applies a correction the model's own memory does not warrant; a tracker slower than its model applies stale balance corrections to recent gradients. No new parameter is introduced: both rates are already fixed by the forgetting table (`tab:risk:forgetting-rates`).

An axis with a custom forgetting rate still draws from the eligible tracker, and the resulting mismatch is bounded per label by the rate difference over the complement of the eligible rate — negligible at the default and small anywhere in the supported range. The global and eligible trackers take the operational and inherent rates respectively, matching the model horizons this table assigns them.

**Algorithm (The tracker update)** · `alg:weighting:tracker-update`

Each tracker is an exponentially weighted average of the positive-valence indicator, updated on the labels that qualify for it.

1. **Determine** eligibility from the recorded action and the ground-truth flag — a table lookup available before any model is touched (`tab:eligibility:training`).
2. **Update** the global tracker on every label, and the eligible tracker only on eligible labels, by

   $$P_+ \leftarrow \gamma_P \, P_+ + (1 - \gamma_P) \, \mathbb{1}[v > 0]$$

3. **Derive** the weights from the updated rate before the model update reads them (`alg:runtime:update-path`).

The ordering is what makes the procedure well defined. Eligibility is resolved first because it decides which tracker moves and which models train; the trackers move before the weights are derived so that a label contributes to the balance estimate it is itself weighted against, which keeps the estimator consistent as the rate drifts.

**Definition (The trackers' initial value)** · `def:weighting:initial-value`

Both trackers start at $P_{+,0} = 0.5$, where the two weights are equal at unity. The first labels are therefore weighted evenly, committing to no assumed base rate, and the average converges toward the empirical rate over roughly $3/(1 - \gamma_P)$ labels.

| Tracker | Rate | Labels to converge | At two hundred labels a day |
| --- | --- | --- | --- |
| Global | $0.9995$ | About six thousand | About thirty days |
| Eligible | $0.9998$ | About fifteen thousand eligible | About a hundred and twenty-five days at sixty per cent eligibility |

Through convergence the weights under-correct for imbalance relative to the true rate, so the gradient is biased toward the majority class, and the bias decreases monotonically as the estimate settles. The horizon is long enough that it is worth avoiding rather than waiting out: the initial value is configurable, and a host that knows its base rate can seed it directly, or can pre-seed with historical labels (`alg:host:pre-seeding`) and eliminate the transient entirely.

**Decision (No time-indexed decay on the trackers)** · `dec:weighting:no-time-decay`

The trackers do not decay with elapsed time, only with labels. The decision is recorded because every other stateful quantity in the system does decay with time, and the exception looks like an oversight until the reasoning is stated.

The class balance among labels has not changed because time passed without labels. What has changed is the system's certainty about it, and that certainty is already represented elsewhere: the models' own time-indexed precision decay widens the posterior and reduces the influence of historical labels regardless of what weights they carried. Decaying the tracker as well would represent the same loss of certainty twice, and would do it in a place where the representation is wrong — a decayed rate estimate does not become more uncertain, it drifts toward whatever its decay target is.

After a drought the tracker therefore retains its pre-drought estimate, and the first labels back are weighted on the last known balance. If the balance really did change, the tracker adapts over its own horizon, which is the same horizon the model's forgetting factor uses to discount the evidence trained under the old balance. The two stay in step, which is the property the whole design is built around.

**Definition (The balancing weights)** · `def:weighting:balancing-weights`

The weights are the reciprocal of twice the relevant class rate, capped at a ceiling:

$$w_+ = \min\!\left(\frac{1}{2P_+ + \varepsilon},\; w_\text{ceiling}\right), \qquad w_- = \min\!\left(\frac{1}{2(1 - P_+) + \varepsilon},\; w_\text{ceiling}\right)$$

The ceiling defaults to one hundred for every model. The defining property of this form is that each class contributes exactly half the total gradient weight, at any class rate, until the ceiling binds.

| Positive rate | $w_+$ | $w_-$ | Ratio | Gradient balance |
| --- | --- | --- | --- | --- |
| $0.50$ | $1.0$ | $1.0$ | $1.0$ | 50% |
| $0.10$ | $5.0$ | $0.56$ | $8.9$ | 50% |
| $0.05$ | $10.0$ | $0.53$ | $18.9$ | 50% |
| $0.01$ | $50.0$ | $0.51$ | $98.0$ | 50% |
| $0.005$ | $100.0$ | $0.50$ | $200.0$ | 50% |
| $0.002$ | $100.0$ | $0.50$ | $200.0$ | 28.6% |
| $0.001$ | $100.0$ | $0.50$ | $200.0$ | 16.7% |

The ceiling binds below a rate of $1/(2 w_\text{ceiling})$ — half a per cent at the default — and from there the balance falls linearly with the rate, so the model under-learns from exactly the observations it has fewest of (`cav:limitation:ceiling`). A deployment below that rate can raise the ceiling and accept the added single-observation volatility, pre-seed with historical positive labels, or monitor discrimination directly (`def:monitoring:top-decile-lift`). The balancing form includes the ceiling, so every row of this table describes the weights.

**Discussion (Weighting against the leverage bound)** · `disc:weighting:leverage-interaction`

The importance weight and the leverage bound both cap what one observation does, and which of them binds changes over the life of the deployment.

Early, the bound dominates. A positive-valence label carries a large importance weight and arrives against a near-prior covariance, so its leverage is high and the cap $c/(h + \varepsilon)$ overrides the importance weight entirely; the bound fires on a third to two thirds of positive labels during cold start. That is the layered protection working: the importance ceiling cannot prevent gradient concentration during cold start, and the bound can, because it is computed from the posterior rather than from the class rate.

Later, the weight dominates. As the posterior concentrates, leverage falls for typical feature directions, the bound relaxes, and the importance weight becomes the operative constraint — firing on a few per cent of positive labels and almost never on negative ones in steady state.

The interaction is deliberate rather than incidental. A heavily weighted observation is exactly the observation whose potential to distort the posterior is greatest, and it receives proportionally more scrutiny for that reason. The two mechanisms are not redundant: one bounds the influence of a class, the other bounds the influence of a direction, and a deployment needs both.

**Definition (The risk target)** · `def:risk:target`

The target every risk model trains on is the sign of the valence:

$$r_\rho = 2 \cdot \mathbb{1}[v > 0] - 1 \in \{-1, +1\}$$

Positive valence maps to $+1$ and everything else to $-1$, under the sign convention the host declares (`conv:valence:sign`).

The binary encoding does three things. It aligns with the zero-mean prior, since under balanced weighting the expected target is zero, which is what the prior asserts. It eliminates the conflation of severity with probability: a severity-weighted target would train the model to predict expected loss, the product of probability and magnitude, where the risk estimate is meant to represent probability alone. And it makes the gradient uniform, so every label contributes the same magnitude before weighting and no low-severity event arrives as a near-zero gradient.

Severity is not discarded; it enters through features rather than through the target (`cav:limitation:severity-path`). Identity and Ledger valence averages, compressed and raw, carry severity history into the feature vector, and the model learns whatever relationship holds between that history and the binary outcome. A deployment needing severity predicted in its own right registers a severity outcome axis (`def:registry:outcome-axis`).

**Definition (The feature compression scale)** · `def:risk:compression-scale`

A two-sided compression scale governs the compressed valence features — not the risk target, which is already bounded:

$$r_v = \tanh(v / \kappa_v), \qquad \kappa_v \leftarrow \gamma_\kappa\,\kappa_v + (1 - \gamma_\kappa)\,|v| \quad \text{when } v \neq 0$$

The scale adapts to the typical magnitude of the valence values the deployment actually reports, and the compression maps arbitrary magnitudes into the open unit interval.

Compression is needed because the features are averages and an unbounded input lets one extreme observation dominate an average for a long time; adaptation is needed because the alternative is a configured scale, which asks the host to know its own valence distribution before it has seen one. The hyperbolic tangent preserves ordinal distinctions throughout its range while flattening the tail, so a large loss still reads as larger than a moderate one without reading as a hundred times larger. Per-axis scales adapt at the same rate on the labels reporting their axis (`def:axis:adaptive-compression`).

**Algorithm (The leverage-bounded update)** · `alg:update:sherman-morrison`

Every Bayesian linear model in the document is updated by this procedure, given a prior precision $B$, its inverse $\Sigma$, a forgetting factor $\gamma$, a standardised observation $\hat{\phi}$, a target $r$, a target weight $w_\text{target}$, the ceiling, and a leverage safety factor $c$.

1. **Compute the leverage** from the prior covariance: $v = \Sigma \hat{\phi}$ and $h = \hat{\phi}^\top v$.
2. **Cap the weight:** $w_\text{eff} = \min(w_\text{target},\, w_\text{ceiling},\, c/(h + \varepsilon))$.
3. **Apply forgetting:** $B \leftarrow \gamma B$.
4. **Apply the replenishment floor:** $B_{jj} \leftarrow \max(B_{jj}, \lambda_\text{floor})$ for every $j$ (`req:gaussian:prior-replenishment-floor`).
5. **Update the precision:** $B \leftarrow B + w_\text{eff} \hat{\phi}\hat{\phi}^\top$.
6. **Update the covariance:** $\Sigma \leftarrow \frac{1}{\gamma}\bigl(\Sigma - \frac{w_\text{eff} v v^\top}{\gamma + w_\text{eff} h}\bigr)$.
7. **Update the mean** using the prior covariance-vector product $v$ of step 1, not the covariance step 6 has just replaced:

   $$\mu \leftarrow \mu + \frac{w_\text{eff}\,(r - \hat{\phi}^\top \mu)}{\gamma + w_\text{eff} h}\; v$$

Step 7 is stated against $v$ deliberately. The mean update is the posterior covariance times the weighted residual, and substituting the Sherman–Morrison identity turns that into the prior product divided by the same denominator step 6 uses — algebraically identical, and unambiguous about which covariance is meant, where an ordered procedure whose sixth step has already overwritten $\Sigma$ is not. The covariance is recomputed from the precision by Cholesky factorisation every thousand labels, or sooner when the condition number crosses its threshold, because steps 5 and 6 maintain the two matrices independently and accumulate disagreement between them (`alg:gaussian:condition-adaptive-recompute`).

**Proposition (What the leverage bound guarantees)** · `prop:update:leverage-bound`

The leverage $h = \hat{\phi}^\top \Sigma \hat{\phi}$ measures how far an observation stretches the model along a direction of high posterior uncertainty, and the cap $c/(h + \varepsilon)$ bounds the ratio of posterior to prior precision along any direction to at most $1 + c$. At the default safety factor of five, no single observation can increase the precision along its own direction by more than a factor of six.

The bound fires when $h > c/w_\text{target}$, which makes it a function of the class as much as of the geometry: at a negative label's typical weight it requires a leverage of ten and is rare, while at a positive label's weight in a sparse deployment it requires a tenth and is common.

The guarantee is bought at a cost stated plainly. The cap makes the effective weight depend on the current posterior, so the observation model for the $n$-th label depends on the first $n-1$ — the composite posterior is not the posterior of any fixed generative model (`cav:gaussian:fixed-model-departure`). The departure is one-directional: the result is wider than the fixed-model posterior, concentrated in the directions where the bound fires most (`prop:gaussian:conservatism`). A conservative posterior is the right failure mode for a system whose uncertainty is consumed by a decision layer, and the calibration absorbs the effect on point estimates (`def:platt:regimes`).

**Table (Forgetting rates)** · `tab:risk:forgetting-rates`

Every model forgets along two independent axes: by labels processed, and by time elapsed.

| Model | Label-indexed rate | Half-life in labels | Time-indexed rate, hourly | Time half-life |
| --- | --- | --- | --- | --- |
| Operational | $0.9995$ | About 1,400 | $0.9999$ | About 290 days |
| Sister | $0.9998$ | About 3,500 | $0.9999$ | About 290 days |
| Anchor | $0.9998$ | About 3,500 | $0.9999$ | About 290 days |
| Outcome axes, by default | $0.9998$ | About 3,500 | $0.9999$ | About 290 days |

The operational model forgets two and a half times faster than the others, and the asymmetry follows from what each estimates. Operational risk includes the host's intervention posture, which a host may change deliberately and overnight; inherent and coarse risk reflect the underlying environment, which changes on its own slower schedule. A single shared rate would either make the sister chase the host's policy changes or make the operational model lag them.

The time-indexed rate is uniform and deliberately slow. Its purpose is to guarantee that a model left without labels eventually widens rather than holding a stale posterior forever; it is a floor under forgetting, not a mechanism meant to drive forgetting under normal operation, where the label-indexed rate is faster by orders of magnitude.

### Chapter (Outcome Axis Prediction Models) · `chap:spec:axis-prediction`

The chapter is the shortest normative one in the document, and short for a structural reason rather than an editorial one. An outcome axis model is the same Bayesian linear model as the risk models, over the same feature vector, under the same update, differing only in what it is trained to predict — and what it is trained to predict is the host's business, not the Core's. The chapter therefore states the model, its training target, its outputs, what emerges between axes without being modelled, and what registration and deregistration do; everything else it inherits.

**Definition (The per-axis model)** · `def:axis:per-axis-model`

Each registered outcome axis carries an independent Bayesian linear model predicting that axis's value from the feature vector:

$$\hat{o}_a(\phi) = \phi^\top \mu_a, \qquad \sigma^2_{o,a}(\phi) = \phi^\top \Sigma_a \, \phi$$

The structure is identical to the core risk models — a Gaussian posterior over linear weights, updated by the leverage-bounded procedure (`alg:update:sherman-morrison`) — and the axis models are extended and marginalised by the same lifecycle algebra as the sister and operational models (`thm:gaussian:extension`).

The Core interprets no axis. It imposes no assumption about what a predicted value means, no assumption about which direction of it is desirable, and no assumption about how the prediction ought to influence a decision. An axis is a number the host declared, reported, and asked to have predicted; the Core learns the mapping and returns it. That neutrality is what lets a deployment register financial loss, a compliance score, and a latency budget on the same mechanism without the Core needing a theory of any of them (`def:registry:outcome-axis`).

**Definition (The training target)** · `def:axis:training-target`

An axis model trains on the labels its eligibility mode admits, against a compressed target. The mode is fixed at registration (`schema:registry:axis-record`): the eligible-only mode uses the same criterion as the sister model (`tab:eligibility:training`), and the all-labels mode uses every labelled outcome, as the operational model does. The compressed target is

$$r_a = \tanh\bigl(o_a / (\kappa_a + \varepsilon)\bigr) \in (-1, +1)$$

The choice of mode is a causal judgement the host makes about its own axis, and it turns on one question: is this axis's value affected by the action the host took? Financial loss usually is — an intervention that blocks a fraudulent request reduces the loss it would have caused — so an eligible-only model learns inherent expected loss rather than realised loss. A compliance score usually is not, so all labels are admissible and the larger training population is free. Choosing the wrong mode does not produce an error; it produces a model that answers a different question than the host thinks it asked (`cav:limitation:axis-confounding`).

Forgetting is per axis, defaulting to the sister's rate, with time-indexed forgetting at the same rate as the core models (`tab:risk:forgetting-rates`).

**Definition (The adaptive compression scale)** · `def:axis:adaptive-compression`

Each axis carries its own compression scale, adapting on the labels that report that axis:

$$\kappa_a \leftarrow \gamma_\kappa\,\kappa_a + (1 - \gamma_\kappa)\,|o_a| \qquad \text{when } o_a \neq 0$$

The scale does two things at once: it maps arbitrary-range axis values into the open unit interval, where the Gaussian linear model is well behaved, and it learns the typical magnitude of those values so that the host is not asked to declare a scale for a quantity it has not yet observed.

Adaptation has a cost, and it is worth stating rather than discovering. Because the scale moves, historical labels have their effective targets retroactively shifted: a label trained under one scale is interpreted under a later one. Under a sustained shift in axis-value magnitude the inverse-transformed prediction therefore carries a transient multiplicative bias in the ratio of the new scale to the old, decaying as the forgetting factor discards labels trained under the old scale. The reported uncertainty absorbs part of this, since the non-stationarity presents to the posterior as additional unpredictability, but only part (`cav:limitation:kappa-nonstationary`). The current scale is reported per axis so that a host can watch for the jumps that signal a regime change in magnitudes.

**Table (Inference outputs)** · `tab:axis:inference-outputs`

Three quantities are reported per active axis, all of them in the host's own units rather than in the compressed ones the model works in.

| Output | Computation |
| --- | --- |
| Raw prediction | $\kappa_a \cdot \operatorname{atanh}\bigl(\operatorname{clip}(\hat{o}_a, -1+\varepsilon, 1-\varepsilon)\bigr)$ |
| Raw uncertainty | $\kappa_a \cdot \sigma_{o,a} / (1 - \hat{o}_a^2 + \varepsilon)$ |
| Prediction interval | Raw prediction $\pm\, z_\alpha \cdot$ raw uncertainty |

The prediction inverts the compression to recover the original units, and the clip is what keeps the inversion finite when the model predicts at or beyond the boundary. The uncertainty propagates the posterior through the same inverse by its derivative, which is why it grows without bound as the compressed prediction approaches the boundary — correctly so: near saturation the compression has discarded the information that would distinguish a large value from a very large one, and the widening interval says exactly that. The interval uses the standard normal quantile, defaulting to the ninety-five per cent two-sided value.

These outputs are reported to the host within the assessment (`schema:output:assessment`) and never enter the derivation (`inv:guarantee:outcome-neutrality`).

**Definition (The prior-only axis prediction)** · `def:axis:prior-only-prediction`

When the axis evaluation cannot be made — a feature vector whose length disagrees with the model's, or a non-finite value out of the computation — the reported prediction is the axis prior carried through the same transform a successful evaluation uses. The raw prediction is zero, the raw uncertainty is

$$\sigma_{\text{raw},0} = \kappa_a \cdot \sqrt{\frac{1}{\gamma_t^{\Delta t_\text{snap}} \cdot \lambda_\text{prior}}}$$

and the interval is the standard quantile around them, exactly as in the inference outputs (`tab:axis:inference-outputs`). The point estimate is zero because the prior mean is zero, and at the origin of the compression the inverse transform's derivative is one — so the prior standard deviation $\sqrt{T_\text{corr} / \lambda_\text{prior}}$ crosses into the host's units scaled by the compression scale alone.

This is the axis form of the risk basis's own degraded fallback at the same checkpoint, which derives the prior standard deviation from the prior precision and the snapshot's age. The two fallbacks are one checkpoint and answer to one notion of a prior: a degraded axis prediction widens with staleness and carries the axis's own scale exactly as a successful one does, so a consumer cannot mistake it for a confident answer of moderate width, and the degradation is flagged beside it (`dec:degradation:retain-and-flag`).

**Proposition (Cross-axis prediction emerges)** · `prop:axis:cross-axis-prediction`

Where two axes are registered together, each axis model's feature vector already contains the other axis's historical averages — through the identity cells (`tab:keyspace:outcome-state`) and, for spatially enabled axes, through the Ledger features (`tab:extraction:ledger-features`). Cross-axis prediction therefore emerges from the ordinary Bayesian update, with no cross-axis coupling anywhere in the construction: an axis model learns whatever relationship holds between the other axis's history and its own future values, exactly as it learns any other feature's relationship.

What this buys is that cross-axis structure costs nothing to enable and needs no declaration. What it costs is that the structure's quality is not controllable. Convergence depends on co-occurrence: axes frequently reported together update each other's features often and cross-predict well, while axes seldom reported together carry stale cross-axis features and degrade toward the unconditional prediction (`cav:limitation:cross-axis`). Nothing reports that degradation as such, because from the model's point of view a stale feature and an uninformative one are indistinguishable.

**Algorithm (Axis lifecycle)** · `alg:axis:lifecycle`

Registration extends and deregistration marginalises; the anchor model is touched by neither, its projection being fixed.

1. **Extend**, on registering an axis, every non-anchor model — operational, sister, and every existing axis model — by the new axis's feature count (`alg:registry:axis-registration`).
2. **Create** the new axis's own prediction model at the extended dimension.
3. **Destroy**, on deregistering an axis, that axis's prediction model.
4. **Marginalise** every surviving non-anchor model by the removed dimensions (`alg:registry:axis-deregistration`).

The marginalisation is what makes deregistration cheap and correct at once. What an axis's features taught the surviving dimensions is not discarded with them: the Schur complement folds the cross-block precision into the kept block, so a model that learned risk partly through a severity axis's spatial averages keeps that learning after the axis is gone (`thm:gaussian:marginalisation`). Identity features, which are present for every axis, transfer by the same mechanism. The ideal Schur identity is exact; the regularised correction is the declared approximation, and its kept-block fallback transfers none of the correction. That declared operation lets a deployment reshape its axis set without a retraining cycle (`inv:guarantee:structural-exactness`).

### Chapter (The Outcome Ledger) · `chap:spec:outcome-ledger`

The chapter divides into two halves that read very differently. The first is normative: what the Ledger holds, how it is updated and decayed, and what happens when its entries are created, deleted, and collected. The second is analytic, and it exists to establish that the Ledger is structurally uninformative for most cells in most deployments — a conclusion the specification states about its own mechanism rather than leaving to be discovered. Both halves are needed. A reader who takes only the first will over-rely on a memory that is mostly empty; a reader who takes only the second will remove a mechanism that is decisive where it does converge.

**Definition (The Outcome Ledger)** · `def:ledger:purpose`

The Outcome Ledger maintains, for each Sentinel, a layered map of outcome history over that Sentinel's coordinate domain. Each layer corresponds to a depth in the dyadic hierarchy. The root layer, covering the whole domain, always exists; finer layers are created when the Sentinel's batch report carries cells at greater depth. Several layers may cover one coordinate, and feature extraction resolves to the deepest available.

Every layer independently tracks the outcome history of its entire range. A label updates every layer containing its coordinate, from the deepest to the root (`alg:ledger:all-layers-update`), mirroring the multi-scale delivery the spectral Sentinel publishes as its algorithm S-9.3, where an observation reaching a cell also reaches every ancestor on the path above it. Each layer's averages are therefore always current with respect to every label in its range, and no layer is ever stale.

Two properties follow, and between them they eliminate the whole question of state inheritance. Deletion requires no merge, because the next coarser layer already reflects every label the finer layer saw — the only thing lost is spatial discrimination, and a transfer would double-count. Creation requires no inheritance, because the coarser layer continues to exist and be updated underneath; it is the best available prior for the new layer's rate, and it is expressed through the model's learned weights on the coarser layer's features rather than through copied state.

**Summary (When the Ledger is worth its cost)** · `summ:ledger:value`

The Ledger supplies persistent spatial reputation: the ability to remember that a Sentinel cell has historically been associated with adverse outcomes, even after the Sentinel's own measurement baselines have adapted and stopped reporting anything unusual about it. No other feature pathway can do this, because every other pathway is computed from the current report.

The capability requires two conditions at once: the cell's true adverse rate must substantially exceed the population average, and the cell must receive enough eligible labels for its running average to outrun time-indexed decay (`thm:ledger:materiality`). For the majority of cells in typical deployments one or both fail, and the Ledger features carry no information (`cav:limitation:ledger-low-traffic`). Detection in those cells runs through the immediate-convergence pathways instead (`tab:ledger:low-maturity-detection`).

The Ledger is therefore a high-traffic bonus and not a baseline dependency, and the distinction is load-bearing: a deployment that sizes its detection expectations on the Ledger converging will be disappointed everywhere except its busiest cells, and a deployment that removes the Ledger loses precisely the memory its busiest cells depend on.

**Theorem (Ledger materiality)** · `thm:ledger:materiality`

A Ledger bad-rate feature is material — it produces a standardised departure of more than half a standard deviation from the standardisation mean — only when the cell's true adverse rate substantially exceeds the population average *and* its eligible label rate is high enough for the attenuation factor to preserve that excess through time-indexed decay. Neither condition suffices alone.

The two conditions are independent constraints and both bind. A cell at three times the population rate, against a standardisation mean of about $0.03$ and a standard deviation of about $0.04$, needs a steady-state average above $0.05$ to be material — which the attenuation table reaches only above about a hundred eligible labels a day (`data:ledger:attenuation`). A cell at six times the population rate reaches materiality at around ten labels a day, and still fails at three and a half.

The result is the chapter's central negative claim, and it is a theorem rather than an observation because it follows from the two decay constants and the standardisation statistics by arithmetic, not from any deployment's experience. Its practical form is a maturity predicate on each cell and a fleet-wide count of immature cells (`def:monitoring:immature-cells`), so that a deployment can see how much of its Ledger is actually carrying information rather than assuming all of it is.

**Table (Per-entry outcome state)** · `tab:ledger:entry-state`

Every entry, at every depth, carries the same eleven fields.

| Field | Update |
| --- | --- |
| Total assessments | Incremented per assessment routed to this cell |
| Per-action count | Incremented by the recorded action at label time |
| Bad-rate average | Averaged at rate $\lambda_L$ on the positive-valence indicator |
| Compressed valence average | Averaged at rate $\lambda_L$ on the compressed valence |
| Raw valence average | Averaged at rate $\lambda_L$ on the raw valence |
| Per-axis compressed average | One per spatially enabled active axis |
| Per-axis raw average | One per spatially enabled active axis |
| Last updated | The timestamp time-indexed decay is measured from |
| Recent eligible label count | Cumulative, saturating against $N_\text{ledger} = 200$ at its consumers |
| Adverse eligible label count | Cumulative and undecayed, over exactly the population the row above counts |
| Eligible arrival window | Exponentially time-weighted eligible arrivals: one added per eligible label, decayed at $\gamma_{t,\text{ledger}}$ |

The averaging rate is $\lambda_L = 0.999$ by default, a label-indexed half-life of about six hundred and ninety-three labels *per cell* — which is the number that matters, and the reason the analytic half of this chapter exists. In low-traffic regions the cell does not receive labels fast enough for the label-indexed half-life to be the operative one, and the effective decay is dominated by elapsed time instead (`def:ledger:time-decay`).

The saturating count is held cumulative and clipped at its consumers rather than clipped in storage, so that one threshold can be raised later without having discarded the evidence that would justify it.

The last two rows are the materiality evidence, and they exist because the averages above them cannot supply it. Every average in this table has already paid elapsed-time decay by the time anything reads it, so a low average is two situations at once — an ordinary cell, and a high-rate cell whose labels arrive too sparsely to hold the excess. The theorem's two conditions are exactly those two situations told apart (`thm:ledger:materiality`), and telling them apart takes the true rate and the arrival rate as evidence in their own right rather than one attenuated average standing in for both.

The pair is undecayed on both sides and conditioned on eligibility on both sides, because the convergence and attenuation figures below are indexed in eligible labels: a rate measured over a wider population would then be attenuated by a factor computed for a narrower one. The window is decayed on the Ledger's own hourly rate rather than on a horizon of its own, which is what makes it readable as an inter-arrival decay — a cell receiving one eligible label every $\Delta h$ hours settles at $L = 1 / (1 - \gamma_{t,\text{ledger}}^{\Delta h})$, so the per-interval decay the attenuation needs is $1 - 1/L$ and no arrival interval has to be estimated separately. Neither field is a second maintenance phase: both are written by the same all-layers update that writes the averages (`alg:ledger:all-layers-update`) and read by the same read-time decay that reads them (`def:ledger:time-decay`).

**Data (Convergence and steady-state attenuation)** · `data:ledger:attenuation`

The Ledger averages face two independent constraints: how fast a running average tracks a true rate, and how much elapsed-time decay erodes it between labels. From a zero start, the bad-rate average after $n$ eligible labels at a cell of true rate $p$ is $\bar{b}_n = p(1 - \lambda_L^n)$.

| Eligible labels | Fraction of true rate | At $p = 0.05$ |
| --- | --- | --- |
| 10 | 1.0% | 0.0005 |
| 50 | 4.9% | 0.0024 |
| 100 | 9.5% | 0.0048 |
| 200 | 18.1% | 0.0091 |
| 500 | 39.4% | 0.0197 |
| 693 | 50.0% | 0.0250 |
| 1,000 | 63.2% | 0.0316 |

Against that, time-indexed decay at $\gamma_{t,L} = 0.999$ per hour erodes the average between labels, so a cell receiving $r$ eligible labels a day settles at a permanently attenuated value:

$$\bar{b}_\infty(p, r) = \frac{(1 - \lambda_L) \cdot p}{1 - \lambda_L \cdot \gamma_{t,L}^{24/r}}$$

| Label rate, per day | Attenuation | At $p = 0.05$ | At $p = 0.15$ |
| --- | --- | --- | --- |
| 1 | 4.0% | 0.0020 | 0.0061 |
| 3.4 | 12.4% | 0.0062 | 0.019 |
| 10 | 29.4% | 0.015 | 0.044 |
| 100 | 80.6% | 0.040 | 0.121 |

Below about three and a half eligible labels a day the elapsed-time decay dominates outright and the average cannot exceed an eighth of the true rate however many labels eventually accumulate. The same attenuation applies to every per-cell average, compressed, raw and per-axis alike, since all of them share both rates.

The attenuation is evaluated from the stored arrival window rather than from an estimated rate, and the two are the same figure written differently. A window holding $L$ time-weighted eligible arrivals has settled at $\gamma_{t,L}^{24/r} = 1 - 1/L$, and substituting that into the expression above gives

$$A(L) = \frac{1 - \lambda_L}{(1 - \lambda_L) + \lambda_L / L}$$

which reproduces the four tabulated rows to the precision they are stated at. Writing it this way removes the step in which a rate is recovered from the evidence and then exponentiated back, so nothing about the arrival pattern has to be assumed beyond what the window already holds. The two limits read correctly on their own: a window decayed to a single arrival gives $1 - \lambda_L$, which is what one isolated label leaves in an average, and a window filling faster than the decay empties it approaches one.

**Algorithm (The all-layers update)** · `alg:ledger:all-layers-update`

At label time, for each Sentinel that was reporting when the assessment was made:

1. **Find** every entry whose interval contains the request's coordinate for this Sentinel, from deepest to shallowest. The root always exists, so at least one is always found.
2. **Decay** each entry at write time for the elapsed interval since its last update, before applying anything (`def:ledger:time-decay`). The eligible arrival window pays the same factor as the averages; the raw counts pay nothing, being undecayed by construction.
3. **Update the bad-rate average** toward the positive-valence indicator at rate $\lambda_L$.
4. **Update the compressed valence average** toward $\tanh(v / \kappa_v)$.
5. **Update the raw valence average** toward the raw valence.
6. **Update the per-axis averages**, compressed and raw, for each reported axis with spatial features enabled.
7. **Increment** the per-action count by the recorded action.
8. **Increment** the recent eligible label count if the label is eligible (`tab:eligibility:training`), add one to the eligible arrival window, and increment the adverse eligible label count when that eligible label is adverse. Then set the last-updated timestamp.

Every entry containing the coordinate is updated, whether or not a finer entry also contains it; each independently tracks the complete history of its own range. The cost is one pass of averages per containing depth per Sentinel per label, and in practice entries exist at only a few depths — typically three to eight — so the cost is bounded by the hierarchy's realised shape rather than by its possible one.

**Definition (Time-indexed Ledger decay)** · `def:ledger:time-decay`

The Ledger decays with elapsed time far faster than the core models do:

$$\gamma_{t,\text{ledger}} = 0.999 \text{ per hour}, \qquad \text{a half-life of about 29 days}$$

against the models' hourly rate with its half-life near two hundred and ninety days, and the identity layer's faster fourteen (`tab:keyspace:decay-rates`).

The decay is applied in two modes, and the pair is what makes the Ledger readable without locking. At write time, during label processing, the elapsed interval is measured, every average is scaled by the decay over it, the label is applied, and the timestamp advances. At read time, during assessment, the decayed value is computed as a pure function of the stored value and the elapsed interval, without modifying the entry at all. The two agree exactly: read-time decay produces the value that persisting the decay would have produced, which is why an assessment can touch Ledger entries without taking a write lock (`inv:guarantee:non-blocking`).

The fast rate is the primary mitigation for the contamination loop (`alg:valence:contamination-loop`). However few eligible labels a cell receives, its stale reputation dissolves on a twenty-nine day timescale; under label-indexed decay alone a starved cell's reputation would persist far longer, and the elapsed-time rate imposes a hard bound the loop cannot escape (`inv:guarantee:ledger-floor`).

#### Entry lifecycle · `sec:ledger:entry-lifecycle`

The division carries no material of its own. It collects the three events that alter the set of entries — creation, deletion, and collection — and the justification they share, all of them consequences of all-layers routing rather than mechanisms in their own right.

**Algorithm (Entry creation)** · `alg:ledger:entry-creation`

When a cell appears in a Sentinel's batch report and no entry exists at that exact interval, an entry is created with neutral state.

1. **Zero** every average: bad rate, compressed valence, raw valence, and every per-axis average.
2. **Zero** the total assessments, every per-action count, and the recent eligible label count.
3. **Zero** the materiality evidence: the adverse eligible label count and the eligible arrival window. A cell with no arrivals names no arrival rate and a cell with no eligible labels names no true rate, so both read as absent rather than as zero until evidence arrives (`thm:ledger:materiality`).
4. **Stamp** the last-updated timestamp at the current time.

Nothing is inherited from any ancestor. The ancestors continue to exist beneath the new entry and continue to be updated by every label in their range (`alg:ledger:all-layers-update`), so for coordinates inside the new entry's range the new entry is what extraction reads while its ancestors keep accumulating the same labels.

The cost is stated rather than hidden: the features extracted from this entry are zero until labels accumulate, and at the convergence rates of the attenuation analysis that is a long time (`cav:limitation:fresh-entry`). Detection during the interval runs through the immediate-convergence pathways (`tab:ledger:low-maturity-detection`). This is the designed operating mode for a new entry, not a gap in it.

**Algorithm (Entry deletion)** · `alg:ledger:entry-deletion`

When an entry below the root has been absent from the batch report for $N_\text{absent}$ consecutive report cycles — three by default — it is deleted.

1. **Count** consecutive report cycles in which the entry's interval does not appear.
2. **Delete** the entry outright once the count reaches the threshold.
3. **Serve** subsequent coordinates in its former range from the deepest surviving entry, which is already current.

No merge, and no state transfer. The nearest ancestor already reflects every label that passed through the deleted entry's range, having been updated continuously throughout its existence, so a merge would count every one of those labels a second time.

The threshold exists to absorb transience. A cell that drops out of one report — zero observations this cycle, or a passing internal dynamic of the Sentinel — and returns before the threshold finds its entry intact, with no deletion, no recreation, and no loss. The behaviour the threshold governs is deletion, and the entry does not persist in any dormant form after it: an implementation that names this threshold for a suspension it does not perform should correct the name to match what happens.

**Algorithm (Garbage collection)** · `alg:ledger:garbage-collection`

An entry whose averages have all decayed below $10^{-6}$, and whose last label arrived longer ago than the collection horizon — sixty days by default — may be removed. The root is never collected. No merge on collection either; the ancestor is already current.

Two strategies are admissible, and the choice is the deployment's.

1. **Sweep periodically**, the default: a maintenance task scans each Sentinel's Ledger on a fixed cadence, hourly by default, removing entries that meet both predicates. This is the correct strategy for hash-keyed or flat spatial indices, where an expired neighbour has no cheap meaning, and it keeps variable collection cost off the assessment path.
2. **Collect lazily on access**, admissible only where an expired neighbour is cheaply defined, as in a trie: reading or writing an entry reclaims expired neighbours in the same index within a bounded step count.

Either way the per-Sentinel write budget binds for any operation holding the Ledger write lock (`inv:guarantee:staleness`). The floor and horizon are deliberately conservative: collection is a storage economy and never a correctness mechanism, and an entry collected while it still carried information would be indistinguishable, to every consumer, from a cell that had never been seen.

**Justification (Why no state transfers)** · `just:ledger:no-state-transfer`

Neither creation nor deletion moves state, and the two cases fail for opposite reasons.

Inheritance on creation would transfer outcome history from a broader range to a narrower one, which assumes the broader range's adverse rate applies uniformly across its sub-ranges. That assumption is precisely the one the Sentinel refuted by creating a finer cell at all: the cell exists because the Sentinel found structure the coarser range was hiding. Starting from zero assumes nothing, and the ancestor's rate remains available through the model's learned weights on the ancestor's own features — a better path, because the model conditions on every other feature simultaneously where a copied average would condition on nothing.

Merging on deletion would double-count. Under all-layers routing the ancestor's average already includes every label that reached the deleted entry, so folding the child's average in again would inflate the ancestor by the child's entire history. Simple deletion is not the convenient choice among several; it is the only correct one.

#### Contamination and starvation · `sec:ledger:contamination`

The division carries no material of its own. It collects the loop's timescales and its binding constraint, the starvation-relief score, and the opposite-direction exposure that the same symmetric decay creates.

**Table (The loop's timescales)** · `tab:ledger:loop-timescales`

The contamination loop runs through four timescales, and which one binds decides how long a contaminated reputation lasts.

| Timescale | Typical value | What it governs |
| --- | --- | --- |
| Sentinel baseline recovery | Hours to about a day | Measurement features normalising after an adverse event |
| Label-indexed Ledger decay | About 693 labels per cell | The averaging half-life in labels |
| Effective Ledger decay when starved | Bounded by elapsed time | The half-life when eligible labels are rare |
| Time-indexed Ledger decay | About 29 days | The hard bound, whatever the label flow |

Below about three and a half eligible labels a day the elapsed-time decay dominates the effective decay, that being the rate under which steady-state attenuation cannot clear its ceiling (`data:ledger:attenuation`). For most cells in most deployments this is the binding constraint, and the consequence is the reassuring half of the analysis: the loop's persistence is bounded at twenty-nine days however starved the cell becomes. A transient average driven to unity decays to about $0.84$ after a week, about half after twenty-nine days, and to an eighth after eighty-seven — by which point the model's learned weight on the feature produces negligible elevation. The measurement-only anchor feature runs beneath all of it, unaffected (`prop:risk:anchor-measurement-only`).

**Definition (The starvation-relief score)** · `def:ledger:starvation-score`

Label guidance identifies cells whose Ledger state is starved of eligible feedback by scoring the two conditions together:

$$\text{starvation}(s, \text{cell}) = \bigl(\bar{b}_{s,\text{cell}} - P_+^\text{eligible}\bigr)^+ \cdot \left(1 - \frac{n_{\text{elig},s,\text{cell}}}{N_\text{ledger}}\right)^+$$

The first factor measures how far the cell's remembered adverse rate diverges above the system-wide eligible base rate; the second measures how far the cell falls short of the label count at which its average would be trusted. Both are clipped below at zero, so a cell that is either unremarkable or well fed scores nothing at all.

The product is the point. A cell with a high remembered rate and plenty of eligible labels needs no relief — its average is earned. A cell with few labels and an unremarkable rate is merely quiet. It is the conjunction that identifies the cell where the loop is closed: a reputation the system is acting on, held up by evidence too thin to have tested it, and not being tested because the system is acting on it. Those are the cells where an investigated label buys the most, which is what the guidance interface is for (`def:guidance:starvation`).

**Discussion (Reputation laundering and symmetric decay)** · `disc:ledger:laundering`

Everything above concerns stale adverse reputation lasting too long. The decay architecture is symmetric, and an adversary can work the forgiving half of it.

An adversary controlling a key range can generate genuinely benign traffic through it — real, unobjectionable activity earning eligible negative labels — and each such label drives the cell's average and the learned competitive weight downward. After enough benign volume the range reads as clean, and an attack launched from inside it starts from a low-suspicion baseline. This is much cheaper than evading the measurement layer, which demands being statistically indistinguishable from normal at every scale at once; laundering the memory layer demands only sending traffic that is genuinely normal, which by construction it is. Alongside it runs scheduled forgiveness: because elapsed-time decay dissolves reputation on a fixed schedule regardless of label flow, even a detected and confirmed attack's reputational cost expires on that schedule, and the decay cannot distinguish a range that reformed from one that waited out the timer.

Mitigation is partial and structural. A laundered range that turns adverse still trips the measurement-only anchor feature, the aggregate cross-Sentinel features, and the competitive interactions where the range is competitive (`def:feature:template-competitive`) — the Ledger being one pathway of seven (`tab:ledger:low-maturity-detection`). But no mechanism distinguishes an earned benign label from a manufactured one, because the Core trusts every label it is given (`inv:guarantee:honest-uncertainty`). The symmetric-decay trade was resolved deliberately toward curing staleness, false-positive persistence being the more common operational pathology, and the forgiveness exposure is the accepted cost of that choice (`cav:limitation:laundering`).

**Definition (Root entry semantics)** · `def:ledger:root-semantics`

The root entry, at depth zero, carries three distinguished roles under all-layers routing.

It is the universal fallback: any coordinate without a finer entry reads from the root during extraction, which covers every region the Sentinel has never refined deeply enough to report cells for. It is the domain-wide baseline: its average is the running adverse rate across every labelled request routed through this Sentinel, and since every label updates it, the root is the richest, most stable and most current entry in the Ledger. And it is the contamination anchor: being the most heavily smoothed entry, it is the most resistant to any single transient, where finer entries are volatile precisely because they see fewer labels.

The root is created at Sentinel registration and destroyed only at Sentinel deregistration. It is exempt from collection unconditionally (`alg:ledger:garbage-collection`), and the exemption is not an optimisation detail: a Ledger whose root had been collected would have no fallback for unrefined coordinates and no baseline to read finer entries against.

**Table (Ledger against identity outcome state)** · `tab:ledger:versus-identity`

The identity layer's per-cell outcome state is structurally parallel to the Ledger and answers a different question.

| | Sentinel Outcome Ledger | Identity competitive cell state |
| --- | --- | --- |
| Indexed by | Sentinel cell, a spatial coordinate | Identity cell, an entity key range |
| Exists for | Every cell ever seen | Competitive cells only |
| Updated at | Label time | Label time |
| Read at | Assessment time, per Sentinel | Assessment time, through per-dimension aggregates |
| Primary purpose | Per-Sentinel spatial risk features | Per-axis identity context and warm start |
| Contamination mitigation | Elapsed-time decay over 29 days | Elapsed-time decay over 14 days |

The Ledger answers what happens to traffic *through* a network, geographic or categorical region; the identity layer answers what happens to traffic *from* a key range. Both enter the feature vector and neither is redundant with the other: a request can come from a well-behaved entity through a dangerous region or the reverse, and the two states disagree in exactly the cases that matter most. The faster identity decay reflects the difference in what is being remembered — an entity's behaviour can change on its own initiative, where a region's character changes more slowly (`tab:keyspace:outcome-state`).

**Table (Detection at low Ledger maturity)** · `tab:ledger:low-maturity-detection`

The Ledger features are one of seven pathways from a Sentinel to the risk model. When they are immaterial — as they are for most cells (`thm:ledger:materiality`) — detection runs through the other six.

| Pathway | Features | Convergence | Needs the Ledger |
| --- | --- | --- | --- |
| Chain z-scores (`tab:extraction:chain-z-scores`) | 24 | Immediate, from Sentinel baselines | No |
| Chain cumulative sums (`tab:extraction:chain-cusums`) | 12 | Immediate | No |
| Chain structure (`tab:extraction:chain-structure`) | 8 | Immediate | No |
| Coordination (`tab:extraction:coordination`) | 12 | Immediate | No |
| Batch context (`tab:extraction:batch-context`) | 1 | Immediate | No |
| Ledger features (`tab:extraction:ledger-features`) | $3 + 2m_s$ | Weeks to months | Yes |
| Aggregate cross-Sentinel features (`tab:feature:aggregate-block`) | 15 | Immediate | No |

Fifty-seven of the per-Sentinel slot's features carry information from the first batch report, before any label has been processed; the Ledger features, five to ten per cent of the slot, are the only ones needing per-cell outcome history. Outside the slot, competitive indicators populate from the first assessment, per-dimension measurement state tracks alarm profiles independently of any label, and the measurement-only anchor feature bypasses the Ledger entirely (`prop:risk:anchor-measurement-only`).

The architecture is designed so the Ledger is a bonus at high traffic rather than a dependency at any. Where both materiality conditions hold it supplies what no other pathway can — memory that a neighbourhood was dangerous after the measurement baselines have adapted — and where they do not, detection falls back to the immediate pathways as a designed mode rather than a degraded one.

**Decision (Why the Ledger rate is fixed)** · `dec:ledger:no-adaptive-rate`

A traffic-adaptive averaging rate — faster where labels are plentiful, slower where they are scarce — would improve the attenuation factor for exactly the low-traffic cells the analysis shows are uninformative, and it is the obvious repair. It is not adopted, and the reasoning is recorded because the repair will keep suggesting itself.

The elapsed-time decay's primary purpose is contamination mitigation: it must dissolve stale reputation on a fixed schedule whatever the traffic (`def:ledger:time-decay`). An adaptive label-indexed rate that slowed decay at starved cells would extend the contamination loop's persistence in precisely the cells most vulnerable to starvation — the cells where a reputation is least tested and most likely to be wrong. The repair would buy sensitivity in the low-traffic regime at the cost of the one guarantee that makes the low-traffic regime survivable.

The fixed rate is therefore the correct choice, and the attenuation cost is accepted as the price of contamination resistance. What the deployment gets in exchange is a bound it can reason about: no cell's remembered reputation outlives the horizon, whoever stops sending labels and for whatever reason.

## Part (The Decision Landscape) · `part:spec:decision-landscape`

Part III ended at the risk basis. Part IV specifies what a host does with one: a stateless transform from a risk basis, a declared channel policy and a challenge posterior to a posture-indexed decision landscape — the postures at which the optimal action changes, each with its two-source uncertainty and its dominance probability — together with the exploration signal that says whether resolving one of those boundaries is worth an observation. Nothing in this Part learns. The transform holds no state, reads none, and returns the same landscape for the same three inputs however many times it is called.

The Part specifies what the derivation is: its inputs, its presentation-free output, the exploration quantity read from that output, and the optional rendering layered over it.

### Chapter (The Derivation Function Interface) · `chap:spec:derivation-interface`

The chapter fixes Part IV's shape. It states the derivation's signature and the three inputs that cross into it, the landscape those inputs produce, the purity that makes the transform replayable, the two utilities that read a landscape at a posture, and the recorded decision that keeps presentation outside all of it. The three inputs are specified separately because they stand at different distances from the Core: the risk basis is the Core's own output and crosses unchanged, the channel policy is the host's declaration and is developed in the next chapter, and the challenge posterior arrives from a component the Core does not know about — which is why the boundary it crosses is specified here, where the crossing happens, rather than with the component that maintains it.

**Signature (The derivation function)** · `sig:landscape:derivation-function`

The derivation is a function of exactly three arguments — a risk assessment, a channel policy, and a challenge posterior — returning one decision landscape.

| Argument | Supplied by | Carrying |
| --- | --- | --- |
| Risk assessment | The Core | The risk basis, the axis predictions, the per-Sentinel alarm summaries, and a health snapshot |
| Channel policy | The host | The named action set, the reward parameters, the sensitivity exponents, and the neutral zone |
| Challenge posterior | The Companion Tracker | The challenge-effectiveness distribution, conjugate or moment-matched |

The derivation reads the risk basis (`schema:risk:basis`) and ignores the rest of the assessment; the remaining fields travel in the argument because a host holds one assessment and derives from it, not because the transform consumes them. It is deterministic, stateless and closed form: no iteration, no search, no convergence criterion, and no dependence on the Core's parameter state or on the Companion's pseudo-counts beyond the posterior it is handed. The same three inputs produce the same landscape on every machine and at every time (`pf:landscape:purity`).

Nothing else crosses. A fourth argument carrying display parameters would make the returned object a function of presentation, which is exactly what the landscape is specified not to be (`inv:landscape:presentation-free`). Rendering is a separate call over the resulting decision landscape.

**Signature (The challenge posterior)** · `sig:companion:posterior`

The challenge posterior is the distribution over $q_c$, the probability that a challenge correctly identifies an adverse source. It crosses the derivation boundary in one of two variants.

| Variant | Carrying | Supports |
| --- | --- | --- |
| Conjugate | The Beta pseudo-count pair $(\alpha, \beta)$ | Point estimate, variance, and exact quantiles |
| Moment-matched | The point estimate $\hat{q}_c$ and its variance $\sigma^2_{\hat{q}_c}$ | Point estimate and variance; quantiles only through a moment-matched Beta |

Both variants yield $\hat{q}_c$ and $\sigma^2_{\hat{q}_c}$, which is all the crossover locations and their variances require. The conjugate variant carries strictly more: its quantile function is what credible intervals push through (`def:landscape:credible-intervals`) and what makes the dominance probability exact rather than approximated (`thm:landscape:dominance`). The moment-matched variant exists for hosts whose challenge estimate comes from somewhere else — an external analytics pipeline, a fixed operational override — and it degrades those two consumers to a moment-matched surrogate rather than removing them.

The variant distinction is therefore not a convenience: it is the boundary at which the Companion's conjugate structure either survives into the derivation or does not. A host with no Companion at all supplies the uniform prior (`def:companion:prior`), whose effect is conservative — wide intervals and non-trivial dominance probabilities until evidence accumulates — rather than absent. The posterior itself is maintained and decayed elsewhere (`alg:companion:inference`); what this signature fixes is only what arrives here. The conjugate pair survives in the landscape, while an external mean-and-variance pair enters through the moment-matched variant.

**Signature (The decision landscape)** · `sig:landscape:output`

The landscape is the complete decision content of one assessment on one channel: where the optimal action changes, how certain each boundary is, which evidence source that uncertainty comes from, and how likely each action is to be dominated outright.

| Group | Field | Carrying |
| --- | --- | --- |
| Evidence | Shared term $u$ | The risk-driven crossover location, common to every transition |
| Evidence | Shared uncertainty $\sigma_u$ | The single scalar carrying all risk-driven uncertainty |
| Evidence | Challenge posterior | The posterior as it crossed the boundary |
| Transitions | Crossover vector | One entry per adjacent pair of declared actions |
| Regimes | Regime vector | One entry per declared action |

The crossover vector has fixed shape: exactly $J-1$ entries for $J$ declared actions, whatever the evidence says about dominance. Shape stability is what makes a landscape safe to store, to replay, and to difference across a policy comparison (`ex:architecture:compositional-dividends`) — a Companion update that carries $\hat{q}_c$ across a dominance threshold changes values and never structure. Each crossover reports its transition pair, its location, its offset at the point estimate, its sensitivity to the challenge estimate, its variance and its credible interval (`def:landscape:action-crossover`). Each regime reports its action, its logit width where the action is interior, the width's variance, and its domination probability.

Absent from the landscape: no tags, no posture, no classification probabilities, no display constants. Those belong to the rendering layer (`dec:landscape:rendering-optional`). The presentation-free landscape carries the declared exponent sum alongside those groups so the optimal-action utility can reconstruct the cost curves; the rendered tag profile is a separate object.

**Invariant (The landscape is presentation-free)** · `inv:landscape:presentation-free`

Every field of the landscape is a function of the risk basis, the declared policy and the challenge posterior alone. No kernel bandwidth, compression factor, display floor, spectrum parameter or other presentation constant reaches any of them.

The invariant is this chapter's own claim rather than a general property recorded elsewhere, because it is a claim about this signature: it holds exactly when the derivation's argument list is the three of (`sig:landscape:derivation-function`) and fails the moment a fourth argument carrying display parameters is admitted, whether or not any field happens to read it. Two of the composition dividends rest on it directly (`ex:architecture:compositional-dividends`). An external service deriving under a different policy isolates the cost structure exactly only if the compared outputs share no display parameter; a replayed assessment reproduces its landscape from three stored inputs only if three inputs are all there were.

The landscape call takes the three specified inputs, while the separate rendering call is the only one that takes display configuration (`dec:landscape:rendering-optional`).

**Proof (The purity of the derivation)** · `pf:landscape:purity`

The derivation reads no global state, writes no state, allocates no persistent resource, has no side effect, and returns the same landscape for the same three inputs.

The five clauses are demonstrated by construction rather than by testing, because the signature admits no counterexample. The argument list carries no mutable reference, so nothing reachable through it can be written. It carries no handle to the Core, the Companion, a registry, a clock or a random source, so there is no external state to read and no source of nondeterminism to read it from. The return value is the only channel out, so there is no side effect to observe. Determinism then follows from the first four: a function whose output depends only on its arguments, computed by closed-form arithmetic over them, returns one value per argument triple.

This is what makes replay meaningful (`inv:guarantee:replay`): a host that stored the assessment, the policy and the posterior stored the complete determinant of the landscape, and needs no model state to reproduce it (`inv:guarantee:derivation-purity`).

**Signature (The landscape utilities)** · `sig:landscape:utilities`

Two utilities read a landscape at a posture. Neither requires model state and neither is part of the derivation itself.

| Utility | Given | Returning |
| --- | --- | --- |
| Optimal action | A landscape and a posture | The action whose regime contains the posture's logit |
| Fragility | A landscape and a posture | The decision fragility at that posture (`def:fragility:definition`) |

The split is deliberate. The landscape is computed once per assessment per channel and describes every posture at once; the utilities are read repeatedly, at whatever postures the host cares about, without recomputing anything. A host running one assessment against several postures — a staged response, a comparison across operating points — pays for the landscape once and for each reading almost nothing.

Both utilities are total: every posture in the open unit interval has an optimal action and a fragility, including postures inside a dominated action's inverted region, where the naive reading fails and the specified one does not (`alg:landscape:optimal-action`). Both read the landscape without rendering it.

**Algorithm (The optimal action)** · `alg:landscape:optimal-action`

At posture $\pi$ the optimal action is the one minimising expected cost at $\ell(\pi)$. It is computed from the upper envelope of the per-action cost curves, not by locating the bracketing crossover entries.

1. **Reconstruct** each declared action's expected-cost curve as a function of the logit, from the shared term, the per-transition offsets and the posture-sensitivity functions (`def:landscape:posture-sensitivity`).
2. **Evaluate** each curve at $\ell(\pi)$.
3. **Select** the action of least expected cost, ties broken toward the more permissive action so that the answer is left-continuous in the posture.

The bracketing shortcut is wrong and the reason is structural. When an interior action is dominated at the point estimate its two bounding crossovers are inverted — the lower one lies above the upper — so they bracket nothing, and a reader walking the crossover vector in order would return the dominated action for an interval where it is never optimal. The envelope has no such failure mode: a dominated action simply never attains the minimum, and no special case is needed to exclude it (`thm:landscape:dominance`). For the four declared actions of a typical channel two passes suffice, so correctness here costs nothing worth measuring.

**Decision (Rendering is a layer over the landscape)** · `dec:landscape:rendering-optional`

The resonance rendering is a function of a landscape and a risk basis, taken by hosts that want a smooth visual spectrum over classifications and actions. It is optional, and the landscape is complete without it.

The decision is recorded because the alternative is available and was rejected. A derivation that rendered directly would spare hosts one call and one type, and it would cost the two properties the rest of this chapter is built on. Rendering interposes display constants — kernel bandwidths, compression factors, a magnitude floor — between the evidence and the output (`tab:config:rendering`), so a rendered output is a function of presentation and the presentation-free invariant cannot be stated of it (`inv:landscape:presentation-free`). And rendered tag probabilities are normalised kernel evaluations, not posterior probabilities of any event, so a host reading decision semantics off them is reading a display artefact. All decision semantics are therefore defined on the landscape (`sig:landscape:output`) and all exploration semantics on fragility (`def:fragility:definition`); the rendering carries no decision content the landscape does not already hold (`sig:rendering:contract`).

The primary derivation produces the decision landscape without display configuration, and the optional rendering call takes that landscape, the risk basis and the display configuration.

### Chapter (Channel Policy) · `chap:spec:channel-policy`

The channel policy is everything the host declares about one channel: which actions it can actually take, what each mistake costs it, how fast those costs grow with caution, and where on the resulting axis it currently wants to stand. The Core has no concept of any of it. A policy is handed to the derivation on each call and never registered, which is what lets one assessment serve several channels with different action sets and different costs (`ex:architecture:compositional-dividends`).

The chapter is mostly numbers, and the numbers are consequential. Two of them decide what a posture means; five decide where the boundaries between actions fall; one is accepted and read by nothing. The chapter states each, states how sensitive the landscape is to it, and says plainly which of the surprises a host may meet are the declared costs speaking rather than the derivation misbehaving.

**Definition (Posture as the host's cursor)** · `def:channel:posture`

The posture $\pi \in (0, 1)$ is the host's declared level of caution on this channel. It is not an input to the derivation. The landscape is computed without reference to it and describes the crossover structure at every level of caution at once; the posture selects a read-point on that structure.

Given $\pi$, the host locates $\ell(\pi) = \ln\bigl(\pi / (1 - \pi)\bigr)$ among the crossover locations and recovers the optimal action there (`alg:landscape:optimal-action`) and its fragility (`def:fragility:definition`). Posture is the navigator's heading; the landscape is the chart at every heading; the host chooses a heading and reads the chart at that bearing.

The separation is what makes the landscape reusable. A host comparing two operating points, or staging a response across several, computes one landscape and reads it twice, and the two readings are guaranteed consistent because they came from one object rather than from two derivations that might have seen different evidence (`inv:guarantee:posture-independence`). The derivation takes no posture, returns the whole landscape, and exposes the optimal-action and fragility utilities as separate posture-indexed reads.

**Table (The named posture constants)** · `tab:channel:posture-constants`

Four named constants are provided as conveniences.

| Name | $\pi$ | Reading |
| --- | --- | --- |
| Permissive | $0.1$ | Minimise friction; accept risk |
| Normal | $0.3$ | Balance friction and risk |
| Elevated | $0.6$ | Lean toward caution |
| Emergency | $0.9$ | Minimise risk; accept friction |

The host may use any value in the open unit interval. The four are conveniences and not privileged operating points: nothing in the derivation tests for them, no threshold is defined at them, and a landscape read at $0.31$ is neither more nor less well-defined than one read at Normal. They exist so that operational documentation and incident procedure have names to use, and so that a host changing posture under pressure is choosing among prepared positions rather than inventing a number.

Their meanings do not transfer between channels (`prin:channel:relativity`).

**Principle (Posture is channel-relative)** · `prin:channel:relativity`

A posture means nothing on its own. Its operational content is the pair of cost multipliers it induces (`def:landscape:posture-sensitivity`), and those depend on the channel's own sensitivity exponents (`def:channel:sensitivity-exponents`).

Elevated under $\beta_b = 3$ expresses a materially more cautious risk attitude than Elevated under $\beta_b = 1.5$: the same cursor position, the same name, and a different multiplier on the cost of missing an adverse outcome. Postures therefore do not transfer between channels whose exponents differ, and neither do the named constants. A host operating one posture dial across several channels should hold the exponents constant across them, or treat each channel's posture as its own quantity and resist the arithmetic that would average them.

There is a sharper way to say it. The landscape depends on the risk estimate only through a rigid translation of the whole crossover set along the logit axis (`thm:landscape:rigid-translation`), so on a single channel posture and prior log-odds of risk are interchangeable currencies: the posture axis is the risk axis viewed in a mirror. Two channels with different exponents put different scales on that mirror, which is exactly why a posture read on one says nothing about the other.

**Definition (The declared action set)** · `def:channel:actions`

The action set is the ordered list of actions the host can take on this channel, from most permissive to most restrictive. The standard set is Allow, Challenge, Slow, Block; any subset of at least two, in that order, is valid.

An action means one thing to the derivation: a position in the order, with a benign-class cost and an adverse-class cost attached (`tab:landscape:action-costs`). The derivation knows nothing about how the host executes it. Challenge tests the entity; Slow restricts throughput; Block denies. Those readings are the host's, and the derivation's arithmetic is identical for any four actions carrying those costs under any other names.

The order is load-bearing rather than decorative. Crossovers are computed between adjacent declared actions (`def:landscape:action-crossover`), and the reward differentials that locate them are differences between neighbours in the declared order (`def:landscape:reward-differentials`), so declaring the set out of order does not produce a differently-ordered landscape — it produces a wrong one. Any ordered subset of two or more is accepted.

**Caveat (What narrowing the action space costs)** · `cav:channel:action-space-sizing`

The declared actions should correspond to operationally distinct capabilities. Declaring an action the host cannot genuinely execute differently from its neighbour fractures the neighbour's regime without improving any decision.

| Operational capability | Recommended action set | Challenge character |
| --- | --- | --- |
| Allow or deny only | Allow, Block | No Challenge regime |
| Allow, test, or deny | Allow, Challenge, Block | Wide — about a fifth of the posture axis at defaults |
| Allow, test, throttle, or deny | Allow, Challenge, Slow, Block | Narrow — about a twentieth of the posture axis at defaults |
| Allow, throttle, or deny | Allow, Slow, Block | No Challenge regime |

The exposure runs in both directions and neither is a defect of the derivation. Declaring Slow beside Challenge when the two are the same operational response costs Challenge most of its regime, and the arithmetic is quantified rather than asserted (`ex:landscape:three-action`). Narrowing the set instead costs evidence: an action never taken produces no outcomes, and a channel that removes Challenge stops generating the challenge results its own effectiveness estimate is built from (`cav:limitation:challenge-narrow`). A host who finds the four-action Challenge regime surprisingly narrow should first check that Slow is a meaningfully different response, and adopt the three-action set if it is not.

**Table (The reward parameters)** · `tab:channel:reward-parameters`

Seven declared parameters drive every crossover position on the channel.

| Parameter | Symbol | Default | Carrying |
| --- | --- | --- | --- |
| Pass | $R_p$ | $1.0$ | Opportunity cost of full denial |
| Friction | $R_f$ | $0.3$ | Cost of challenging a benign case |
| Missed | $R_m$ | $3.0$ | Cost when an adverse outcome is not prevented |
| Caught | $R_c$ | $2.0$ | Value of identifying an adverse source |
| Blocked | $R_b$ | $1.5$ | Cost of denial to a benign case |
| Slow severity | $\alpha_s$ | $0.2$ | Throttle severity relative to full restriction |
| Block catches | $\beta_c$ | $1.0$ | Fraction of challenge catching retained under Block |

The last two are extended parameters: they appear only inside the derivation's cost model and their per-action semantics are stated there (`def:landscape:extended-rewards`) rather than restated here. Only ratios matter, since the crossover locations depend on the parameters through the logarithm of a differential ratio (`thm:landscape:rigid-translation`) — a channel that doubles every reward gets the identical landscape.


**Data (Crossover position against the reward ratio)** · `data:channel:reward-sensitivity`

The Allow-to-Challenge transition is the most consequential boundary on most channels. Its position is governed by the ratio of the benign-class cost of challenging, $\Delta_\text{good} = R_f$, to the adverse-class benefit of challenging, $\Delta_\text{bad} = \hat{q}_c(R_m + R_c)$.

| $R_m$ | $R_f$ | Ratio | $\pi^*$ at $\hat{p} = 0.1$ | at $\hat{p} = 0.3$ | at $\hat{p} = 0.5$ |
| --- | --- | --- | --- | --- | --- |
| $3$ (default) | $0.3$ | $1{:}8$ | $0.508$ | $0.375$ | $0.300$ |
| $10$ | $0.3$ | $1{:}20$ | $0.421$ | $0.297$ | $0.232$ |
| $30$ | $0.3$ | $1{:}53$ | $0.329$ | $0.222$ | $0.169$ |
| $100$ | $0.3$ | $1{:}170$ | $0.236$ | $0.152$ | $0.114$ |

Computed at $\hat{q}_c = 0.5$, $\beta_b = 1.5$, $\beta_g = 1.0$, $R_c = 2$.

As the ratio falls — missing an adverse outcome growing more expensive relative to friction — the crossover moves toward Permissive and the Allow regime shrinks. That is the correct decision-theoretic behaviour and not a saturation artefact: if the host has declared that a miss costs a hundred and seventy times what a challenge does, almost everything should be challenged.

**Data (How the challenge estimate moves the crossover)** · `data:channel:challenge-interaction`

Challenge effectiveness scales the adverse-class benefit of challenging linearly, so the Allow-to-Challenge crossover moves with it.

At the default rewards and $\hat{q}_c = 0.5$, the crossover sits at $\pi^* = 0.508$ for a risk estimate of $0.1$. At $\hat{q}_c = 0.1$ — challenges rarely identifying an adverse source — the same crossover moves to $0.663$: much less of the posture axis favours Challenge, because challenging buys much less. The reward-sensitivity table is therefore read together with the Companion's current posterior (`alg:companion:inference`) and not on its own.

The posterior's spread matters as well as its centre. Higher challenge uncertainty widens the credible interval of every crossover that depends on $\hat{q}_c$, through that crossover's sensitivity term (`thm:landscape:crossover-covariance`), and it does so without moving the point estimate at all. A host seeing wide boundaries and a stable modal action is looking at challenge uncertainty rather than risk uncertainty, and the decomposition says which (`def:fragility:definition`). The derivation reads the point estimate into the policy constants and the variance into each crossover's uncertainty.

**Remark (Verify the declared costs before reading the landscape)** · `rem:channel:verify-costs`

A host who finds the crossover positions surprising should suspect the declaration before the derivation.

The landscape is a faithful rendering of the declared reward structure. If it says challenge nearly everything, the declared cost of a miss says that missing an adverse outcome is very expensive relative to friction; if the Challenge regime is a sliver, the declared action set says Slow is a distinct response and the declared costs say it is nearly as good as challenging for adverse traffic. In both cases the remedy is the declaration, not the landscape (`cav:limitation:reward-sensitivity`).

The practical form is a short pre-flight. Are all declared actions genuinely distinct operational responses (`cav:channel:action-space-sizing`)? Are the five costs on one scale, and do their ratios match what the business would actually trade? Are the exponents the ones this channel's risk attitude wants, rather than the ones another channel used (`def:channel:sensitivity-exponents`)? A landscape derived from unexamined defaults is a decision-theoretic statement the host has not actually made.

**Proposition (The challenge regime width)** · `prop:channel:challenge-width`

In the four-action set the Challenge regime's width in logit space is fixed by the reward structure and the challenge estimate alone:

$$w_C = \frac{1}{\beta_b + \beta_g}\left[\ell(\hat{q}_c) + \ln\!\left(1 + \frac{R_c}{R_m}\right)\right]$$

At the default $\hat{q}_c = 0.5$ and $R_c / R_m = 2/3$ this is $0.204$ logit units, about a twentieth of the posture axis. The width is independent of $\alpha_s$, $R_f$, $R_b$, $R_p$ and $\beta_c$; the slow-severity parameter enters both differentials at the Challenge-to-Slow transition as a common factor and cancels, so it governs the Slow regime's width and not this one. It is independent of the risk estimate for the same reason every regime width is (`thm:landscape:rigid-translation`).

Three levers widen it, and only three. Raise the catching value relative to the cost of a miss; supply evidence of high challenge effectiveness through the Companion; or remove Slow from the action set, which is by far the largest of the three and takes Challenge to about a fifth of the axis (`ex:landscape:three-action`). Each interior landscape regime reports the width and its variance directly, and the rendering expresses the width through its magnitudes, so a host can observe any of the three levers acting.

**Definition (The sensitivity exponents)** · `def:channel:sensitivity-exponents`

Two exponents govern how fast the cost of each kind of error grows with posture: $\beta_b$ for missing adverse observations, $\beta_g$ for restricting benign ones. Defaults: $\beta_b = 1.5$, $\beta_g = 1.0$.

They are policy fields of the channel and not derivation constants, because they are a statement about the host's risk attitude rather than about the arithmetic: a channel whose adverse outcomes are catastrophic and whose friction is cheap declares a high $\beta_b$, and one protecting a latency-sensitive path declares a low one. Their sum appears as the divisor of every crossover location and of the shared uncertainty (`thm:landscape:rigid-translation`), so raising both together compresses the whole landscape toward the centre of the posture axis without reordering anything, while raising one alone shifts it.

Because they fix what a posture means, they are the mechanism behind posture's channel-relativity (`prin:channel:relativity`), and a host that changes them has changed the meaning of every posture previously recorded on the channel.

**Definition (The neutral zone)** · `def:channel:neutral-zone`

The neutral zone is a declared policy field, default $0.1$, which affects neither the landscape nor any decision read from it.

It is accepted and inert by design. The quantity it names — the band around neutrality within which a classification is reported as unremarkable — is a property of the display spectrum and is defined by the rendering layer (`def:rendering:magnitudes`), where it can be changed without changing any decision. Admitting it in the channel policy keeps one declaration site for everything the host says about a channel; reading it in the derivation would make a decision boundary a function of a display parameter, which the presentation-free invariant forbids (`inv:landscape:presentation-free`).

The field is therefore specified as carried and unread, and an implementation that begins reading it in the derivation has broken an invariant rather than fixed an oversight.

### Chapter (The Crossover Landscape) · `chap:spec:crossover-landscape`

The chapter derives the landscape. It fixes how costs scale with posture, the per-action cost model and the differentials between adjacent actions, the decomposition that separates the two evidence sources exactly, and then the crossovers, the dominance probabilities, the covariance and the intervals that follow from it in closed form.

Nothing here iterates and nothing here searches. Every quantity is an elementary function of the risk basis, the declared rewards and the challenge posterior, and the chapter's central result is why: the risk estimate enters every crossover through one shared term, so the entire structure translates rigidly along the logit axis as risk moves, and everything about the structure's *shape* is independent of risk altogether. That is what makes a regime table reusable across assessments, a width exactly known, and an uncertainty attributable to one evidence source rather than to a mixture.

**Definition (The posture-sensitivity functions)** · `def:landscape:posture-sensitivity`

Two multipliers scale the two classes of cost with posture. The cost of missing an adverse observation grows with caution; the cost of restricting a benign one falls with it.

$$s_\text{bad}(\pi) = e^{\beta_b \cdot \ell(\pi)}, \qquad s_\text{good}(\pi) = e^{-\beta_g \cdot \ell(\pi)}$$

At Permissive the pair is about $0.04$ and $9.0$; at Emergency, about $27.0$ and $0.11$. The exponential-on-logit form is chosen for three properties at once: the multipliers are smooth and strictly monotone across the whole posture axis, they are unbounded in both directions so no declared cost ratio is unreachable, and their logarithms are linear in $\ell(\pi)$ — which is what makes every crossover condition solvable in closed form rather than numerically (`thm:landscape:rigid-translation`).

Because these multipliers are the entire operational content of a posture, they are the mechanism behind posture's channel-relativity (`prin:channel:relativity`): two channels declaring different exponents are using the same cursor to mean different things.

#### The per-action cost model · `sec:landscape:cost-model`

The division carries no material of its own. It collects the two extended reward parameters, the per-action cost tables they enter, the differentials between adjacent actions, and the one threshold at which an action's differential can vanish.

**Definition (The extended reward parameters)** · `def:landscape:extended-rewards`

Two of the seven declared rewards (`tab:channel:reward-parameters`) appear only in the cost model, and both are dimensionless fractions rather than costs.

**Slow severity** $\alpha_s \in (0, 1)$, default $0.2$, places Slow on the restriction spectrum between Challenge and Block. At $\alpha_s = 0$ Slow is Challenge; at $\alpha_s = 1$ it is Block. It scales both the friction Slow adds for benign traffic and the share of adverse outcomes Slow prevents, which is why it governs the Slow regime's width and cancels out of the Challenge regime's (`prop:channel:challenge-width`).

**Block catches** $\beta_c \in [0, 1]$, default $1.0$, is the fraction of the challenge mechanism's catching value retained under Block. At $\beta_c = 1$ the channel challenges and then blocks, so the full catching opportunity survives denial; at $\beta_c = 0$ Block preempts the challenge entirely and no intelligence is gathered from denied requests. It is the one parameter that can make an action's adverse-class differential vanish (`thm:landscape:catching-forfeiture`).

**Table (The per-action costs)** · `tab:landscape:action-costs`

Each action carries a benign-class cost and an adverse-class cost. Positive is a cost to the host; negative is a benefit.

| Action | Benign-class cost | At defaults |
| --- | --- | --- |
| Allow | $0$ | $0$ |
| Challenge | $R_f$ | $0.30$ |
| Slow | $R_f(1 + \alpha_s)$ | $0.36$ |
| Block | $\beta_c R_f + R_b + R_p$ | $2.80$ |

| Action | Adverse-class cost | At defaults |
| --- | --- | --- |
| Allow | $R_m$ | $3.00$ |
| Challenge | $(1 - \hat{q}_c)R_m - \hat{q}_c R_c$ | $0.50$ |
| Slow | $(1 - \hat{q}_c)(1 - \alpha_s)R_m - \hat{q}_c R_c$ | $0.20$ |
| Block | $-\beta_c \hat{q}_c R_c$ | $-1.00$ |

Two readings are worth making explicit. The adverse-class cost of Block is negative at the defaults because blocking an adverse source is a benefit and the challenge-then-block sequence still collects the catching value; it is exactly zero at $\beta_c = 0$, where denial buys prevention and no intelligence. And Challenge's adverse-class cost is a mixture rather than a reduction: the share $\hat{q}_c$ is caught and the remaining share is missed, which is why the challenge estimate scales the benefit of every transition into or through Challenge.

**Definition (The reward differentials)** · `def:landscape:reward-differentials`

For an adjacent pair in the declared order, the benign-class differential is the additional cost the more restrictive action imposes on benign traffic, and the adverse-class differential is the benefit it buys on adverse traffic:

$$\Delta_\text{good}^{(j \to j+1)} = C_\text{good}(a_{j+1}) - C_\text{good}(a_j), \qquad \Delta_\text{bad}^{(j \to j+1)} = C_\text{bad}(a_j) - C_\text{bad}(a_{j+1})$$

For the four-action set:

| Transition | $\Delta_\text{good}$ | $\Delta_\text{bad}$ | At defaults |
| --- | --- | --- | --- |
| Allow to Challenge | $R_f$ | $\hat{q}_c(R_m + R_c)$ | $0.30$ / $2.50$ |
| Challenge to Slow | $\alpha_s R_f$ | $(1 - \hat{q}_c)\alpha_s R_m$ | $0.06$ / $0.30$ |
| Slow to Block | $R_b + R_p + (\beta_c - 1 - \alpha_s)R_f$ | $(1 - \hat{q}_c)(1 - \alpha_s)R_m - \hat{q}_c R_c(1 - \beta_c)$ | $2.44$ / $1.20$ |

For the three-action set the Challenge-to-Block differentials are $(\beta_c - 1)R_f + R_b + R_p$ and $(1 - \hat{q}_c)R_m + (\beta_c - 1)\hat{q}_c R_c$, which at the defaults are $2.5$ and $1.5$. They are computed from the per-action costs directly and are *not* the sum of the four-action set's Challenge-to-Slow and Slow-to-Block differentials: removing an action changes which pair is adjacent, not merely which crossovers are reported.

**Theorem (The catching-forfeiture threshold)** · `thm:landscape:catching-forfeiture`

When $\beta_c < 1$ the Slow-to-Block adverse-class differential decreases and can reach zero. Block is dominated exactly when

$$\hat{q}_c > \frac{(1 - \alpha_s)R_m}{(1 - \alpha_s)R_m + R_c(1 - \beta_c)}$$

| $\beta_c$ | Threshold | Reading |
| --- | --- | --- |
| $1.0$ (default) | $1.0$, never reached | Challenge-then-block; Block always viable |
| $0.5$ | $0.706$ | Partial forensic value from denied requests |
| $0.0$ | $0.545$ | Pure block; Block dominated at moderate effectiveness |

The result is a theorem rather than an observation because it follows from the cost tables by algebra: the numerator is the prevention Block adds over Slow and the denominator adds the catching value Block forfeits, so the threshold is the challenge effectiveness at which forfeited intelligence exactly cancels added prevention. Above it, blocking is worse than throttling for adverse traffic and strictly worse for benign traffic, so no posture makes Block optimal.

The crossing is continuous, not a switch: the differential approaches zero, the crossover location grows without bound in the logit, and the regime inverts by a depth that measures how far past the threshold the estimate sits (`thm:landscape:dominance`). A non-positive differential produces an infinite crossover, from which the inversion depth — the quantity that says *how far* — is not recoverable.

**Theorem (The rigid-translation decomposition)** · `thm:landscape:rigid-translation`

Every crossover separates exactly into one term driven by the risk estimate and shared by all of them, and one offset per transition driven by the reward structure and the challenge posterior.

At the crossover posture between adjacent actions the two expected costs are equal,

$$\hat{p} \cdot \Delta_\text{bad}^{(j)} \cdot s_\text{bad}(\pi^*) = (1 - \hat{p}) \cdot \Delta_\text{good}^{(j)} \cdot s_\text{good}(\pi^*)$$

and solving for the crossover's logit, using the identity relating the risk probability to the effective raw estimate and the calibration parameter (`def:risk:probability`), gives the decomposition (`eq:landscape:rigid-decomposition`).

Three consequences follow and the rest of the chapter rests on them. The shared term is common to every crossover, so as the risk estimate varies the entire crossover set translates rigidly along the logit axis without changing spacing or order (`inv:guarantee:rigid-translation`). All risk-driven uncertainty in the landscape is therefore the single scalar $\sigma_u = \sigma_\text{eff} / ((\beta_b + \beta_g)\kappa_\text{eff})$, applied once rather than propagated per transition. And the offsets depend on the rewards and the challenge estimate alone, so every regime width is independent of the risk estimate — which is why a regime table computed for one assessment is the regime table for every assessment on that channel at that posterior. The shared term is carried once on the landscape, and each crossover record carries its offset and challenge sensitivity.

**Equation (The rigid decomposition)** · `eq:landscape:rigid-decomposition`

$$\ell^*_{j \to j+1} = \underbrace{\frac{-\hat{\rho}_\text{eff}}{(\beta_b + \beta_g)\,\kappa_\text{eff}}}_{u} \;+\; \underbrace{\frac{1}{\beta_b + \beta_g}\ln\frac{\Delta_\text{good}^{(j)}}{\Delta_\text{bad}^{(j)}(\hat{q}_c)}}_{b_j(\hat{q}_c)}$$

The offsets and their sensitivities to the challenge estimate, at the default rewards and $\hat{q}_c = 0.5$:

| Transition | $b_j$ | $s_j = \partial \ell^*_j / \partial \hat{q}_c$ | At defaults |
| --- | --- | --- | --- |
| Allow to Challenge | $-0.848$ | $-1/[(\beta_b + \beta_g)\,\hat{q}_c]$ | $-0.8$ |
| Challenge to Slow | $-0.644$ | $+1/[(\beta_b + \beta_g)(1 - \hat{q}_c)]$ | $+0.8$ |
| Slow to Block | $+0.284$ | $[(1 - \alpha_s)R_m + R_c(1 - \beta_c)] \,/\, [(\beta_b + \beta_g)\,\Delta_\text{bad}^{(\text{S}\to\text{B})}]$ | $+0.8$ |
| Challenge to Block, three-action | $+0.204$ | $+1/[(\beta_b + \beta_g)(1 - \hat{q}_c)]$ | $+0.8$ |

Interior regime widths are differences of adjacent offsets and are therefore constants of the reward structure and the challenge posterior: $w_C = 0.204$ and $w_S = 0.928$ in the four-action set, and $w_C = 1.052$ in the three-action set. The Slow width carries no challenge term at all at $\beta_c = 1$, so it is exactly known however uncertain the Companion is.

**Definition (The action crossover)** · `def:landscape:action-crossover`

A crossover is the boundary between two adjacent declared actions, reported as one record per adjacent pair.

| Field | Carrying |
| --- | --- |
| Transition | The adjacent-in-declared-order pair the boundary lies between |
| Location | The crossover's logit, the shared term plus this transition's offset |
| Offset | This transition's offset at the point estimate |
| Challenge sensitivity | The derivative of the location in the challenge estimate |
| Variance | The two-source variance at this crossover (`thm:landscape:crossover-covariance`) |
| Interval | The ninety-five per cent credible interval on the logit scale (`def:landscape:credible-intervals`) |

Under dominance a crossover may be inverted relative to its neighbours — the lower boundary of an interior action lying above its upper boundary — and the record is reported unchanged. That is the landscape's honest statement that the intervening action has no viable regime at the point estimate, and the magnitude of the inversion is how far from viability it sits. Consumers recover the optimal action from the cost-curve envelope rather than from the record's order (`alg:landscape:optimal-action`). The crossovers form a public vector on the landscape with one record per adjacent pair. Each record carries the fields above and also retains the adjacent cost differentials used by the optimal-action utility.

**Equation (The crossover)** · `eq:landscape:crossover`

$$\pi^*_{j \to j+1} = \sigma\bigl(\ell^*_{j \to j+1}\bigr), \qquad \ell^*_{j \to j+1} = u + b_j(\hat{q}_c)$$

The posture-scale crossover is the logistic image of the logit-scale one, and the landscape reports both because they answer different questions: the logit scale is where the arithmetic is linear and the uncertainty is Gaussian, and the posture scale is where the host's cursor lives. Every derived quantity — the variance, the interval, the width, the covariance — is computed on the logit scale and mapped afterward, never the reverse, since the logistic map is nonlinear and an interval transformed after computation is exact where a variance transformed after computation is not.

**Theorem (Dominance as a probability)** · `thm:landscape:dominance`

Because every regime width is independent of the risk estimate (`thm:landscape:rigid-translation`), every domination condition reduces to a threshold on the challenge effectiveness alone. Dominance is therefore reported as a probability under the challenge posterior, exact and closed form for the conjugate variant:

$$P(\text{action } a \text{ dominated}) = I_{\theta_a}(\alpha, \beta) \quad \text{or} \quad 1 - I_{\theta_a}(\alpha, \beta)$$

with $I$ the regularised incomplete beta function and $\theta_a$ the action's threshold, the direction taken per row:

| Action | Dominated when | $\theta_a$ at defaults | $P$ at the uniform prior |
| --- | --- | --- | --- |
| Challenge, four-action | $q_c \leq R_m/(2R_m + R_c)$ | $0.375$ | $0.375$ |
| Challenge, three-action | $q_c \leq R_m R_f/[(R_b + R_p)(R_m + R_c) + R_m R_f]$ | $0.067$ | $0.067$ |
| Block, at $\beta_c < 1$ | $q_c$ above the forfeiture threshold (`thm:landscape:catching-forfeiture`) | $1.0$ at $\beta_c = 1$ | $0$ |
| Slow, at defaults | never; its width is positive unconditionally | — | $0$ |

Dominance is reported and never structurally enforced (`inv:guarantee:dominance`): the crossover vector keeps one entry per adjacent declared pair whatever the probability says, so the landscape's shape stays a continuous function of the evidence and a Companion update moves inversion depths and probabilities smoothly rather than adding or removing structure. An action at intermediate probability — Challenge at Companion cold start, at $0.375$, is the canonical case — is reported with both its point-estimate regime and its probability, so the host sees an open evidential question rather than a settled fact. Display suppression of dominated actions belongs to the rendering layer and has no counterpart here (`alg:rendering:dominated-treatment`). The moment-matched posterior evaluates the same expression on its moment-matched Beta. The regularised incomplete-beta mass for every regime is reported as the domination probability.

**Theorem (The crossover covariance)** · `thm:landscape:crossover-covariance`

The landscape's uncertainty has exactly two independent sources, which gives the crossover vector a covariance of rank two:

$$\operatorname{Cov}(\ell^*_j, \ell^*_k) = \sigma_u^2 + s_j s_k \sigma^2_{\hat{q}_c}$$

The diagonal entries are the per-crossover variances the records report. The off-diagonal structure is what any functional of several crossovers needs: the variance of a linear combination is $(\sum_j c_j)^2 \sigma_u^2 + (\sum_j c_j s_j)^2 \sigma^2_{\hat{q}_c}$, and treating the crossovers as independent gets that wrong in both directions depending on the signs.

Adjacent covariance is frequently negative, and the sign is informative rather than pathological. Allow-to-Challenge has sensitivity $-0.8$ and Challenge-to-Slow has $+0.8$, so their covariance is $\sigma_u^2 - 0.64 \sigma^2_{\hat{q}_c}$: challenge evidence that raises the estimate moves the two boundaries apart, widening Challenge from both sides at once. A consumer judging the joint stability of an interior action's regime must use this structure (`inv:guarantee:two-source-uncertainty`). Each crossover's variance takes the specified two-source form, and the shared risk uncertainty and each crossover's challenge sensitivity let a consumer recover every off-diagonal entry without a materialised matrix.

**Proposition (Regime widths carry no risk uncertainty)** · `prop:landscape:width-variance`

The variance of an interior regime's width is

$$\operatorname{Var}(w_j) = (s_{j+1} - s_j)^2 \,\sigma^2_{\hat{q}_c}$$

The shared term cancels identically between the two bounding crossovers, which is the covariance restatement of the rigid decomposition (`thm:landscape:rigid-translation`): widths are functions of the reward structure and the challenge posterior only, so no amount of risk uncertainty makes a width less certain.

At the defaults with the uniform prior, the Challenge width's variance is $(1.6)^2 \times 0.0833 = 0.213$ — a standard deviation of about $0.46$ against a point width of $0.204$, so at Companion cold start the width is less certain than its own value — while the Slow width's variance is exactly zero.

The operational content is a routing rule the landscape can state and no single component can. A host reading a wide width variance knows the remedy is challenge-outcome evidence and not labels, because labels cannot move a quantity that risk uncertainty does not enter (`data:companion:contributing-rate`). The landscape surfaces both the width and this variance on every interior regime; boundary regimes carry neither because their widths are unbounded on one side.

**Definition (Credible intervals by quantile push-through)** · `def:landscape:credible-intervals`

Each crossover is monotone in the challenge effectiveness on its viable domain — Allow-to-Challenge strictly decreasing, Challenge-to-Slow and Challenge-to-Block strictly increasing, Slow-to-Block monotone where Block is undominated — so an interval on the effectiveness maps to an interval on the crossover by evaluating the offset at its endpoints:

$$\text{interval}_{95}(\ell^*_j) = \Bigl[\,u + \min b_j(q_{lo}, q_{hi}) - z_{0.975}\,\sigma_u, \;\; u + \max b_j(q_{lo}, q_{hi}) + z_{0.975}\,\sigma_u \,\Bigr]$$

with the endpoints the posterior's two-and-a-half and ninety-seven-and-a-half per cent quantiles. The composition unions the two sources rather than convolving them, so it is conservative by construction; an implementation wanting tighter coverage may substitute numerical convolution.

The push-through is exact in the challenge effectiveness however nonlinear the offset is — including the steep Slow-to-Block dependence near the forfeiture threshold (`thm:landscape:catching-forfeiture`), where a Gaussian linearisation of the offset is a poor summary and a delta-method interval would understate the uncertainty badly. At the uniform prior the intervals are wide and bounded, which is how the landscape expresses Companion cold start without needing a floor parameter, and they narrow as challenge outcomes accumulate and as the Core's effective uncertainty contracts. Exact quantiles require the conjugate posterior; the moment-only variant supplies them from a moment-matched Beta, and the boundary admits nothing else (`sig:companion:posterior`).

**Algorithm (The computation order)** · `alg:landscape:computation-order`

The landscape and any rendering over it are computed in six steps, each depending only on steps before it.

1. **Crossovers.** The shared term from the risk basis, the offsets from the rewards and the challenge estimate, and their sum (`eq:landscape:rigid-decomposition`). Nothing earlier is needed and nothing later feeds back.
2. **Bandwidths.** The display bandwidths, from the crossover spacing and the two uncertainty sources (`def:rendering:bandwidths`).
3. **Magnitudes.** The tag magnitudes, from the risk basis and the bandwidths (`def:rendering:magnitudes`).
4. **Interior action locations.** The interior action tags placed by crossover matching, which reads the crossovers of step one and the bandwidths of step two (`alg:rendering:placement`).
5. **Boundary action locations.** The outermost action tags placed relative to the interior ones, which therefore cannot be computed before them.
6. **Classification tags.** Placed and scaled from the magnitudes of step three and the bandwidths of step two.

The chain is acyclic, and the specification fixes it because the alternative is a silent disagreement. Steps four and five in particular have a real dependency between them — a boundary tag is placed by reference to its interior neighbour — and computing them in the other order produces different locations without producing an error. The order is stated here, in the chapter that owns step one, so that the whole chain has one authority rather than one per layer.

**Definition (Outcome prediction neutrality)** · `def:landscape:outcome-neutrality`

The derivation reads the risk basis and nothing else of the Core. Stated concretely, what does *not* enter it:

- outcome axis predicted values and their uncertainties (`tab:axis:inference-outputs`)
- outcome axis prediction intervals
- any function of an axis model's parameters
- any Core state beyond the risk basis (`schema:risk:basis`)

What *does* influence the risk estimate, inside the Core and upstream of the derivation, are the historical outcome-axis averages carried as features: the identity layer's per-cell averages (`tab:keyspace:outcome-state`) and the Ledger's per-cell averages for spatially enabled axes (`tab:extraction:ledger-features`).

The distinction is between predictions and history. Historical averages enter the feature vector alongside everything else and influence the risk estimate through learned weights, which is intended — outcome history is informative about risk — and carries the contamination exposure that pathway always carries (`disc:valence:axis-pathway`). Predictions are Core outputs reported to the host and have no pathway into the derivation at all (`inv:guarantee:outcome-neutrality`). The derivation's input carries the assessment, channel policy and challenge posterior, but reads only the assessment's risk basis; no outcome prediction enters the landscape.

### Chapter (Worked Landscapes) · `chap:spec:worked-landscapes`

Four landscapes, worked end to end from a risk basis and a channel policy to crossovers, regimes and fragility, followed by a summary of how the design treats each of its own concerns.

The chapter is expository in force and exact in content. Every figure below is computed from the preceding chapters at stated inputs and stated to three decimals, which makes the four examples a complete numeric acceptance oracle for Chapters 13 through 15: an implementation reproducing these tables has the decomposition, the covariance, the dominance thresholds and the widths right, and one that misses a figure has a defect the figure localises. Each example was chosen to isolate a property that is easy to get wrong — the risk-independence of the regime table, the attribution of uncertainty to a source, the cost of declaring an extra action — rather than to illustrate a typical deployment.

**Setup (Assumptions common to all four landscapes)** · `setup:landscape:worked-assumptions`

All four examples use the default sensitivity exponents $(\beta_b, \beta_g) = (1.5, 1.0)$, the default rewards, $\beta_c = 1$, and the Companion at the uniform prior, giving $\hat{q}_c = 0.5$ and $\sigma^2_{\hat{q}_c} = 1/12 \approx 0.0833$. Only the risk basis and, in the fourth, the declared action set vary.

| Quantity | Value |
| --- | --- |
| Allow to Challenge differentials | $0.30$ / $2.50$ |
| Challenge to Slow differentials | $0.06$ / $0.30$ |
| Slow to Block differentials | $2.44$ / $1.20$ |
| Offsets: Allow to Challenge, Challenge to Slow, Slow to Block | $-0.848$, $-0.644$, $+0.284$ |
| Challenge sensitivity, every transition | $\pm 0.8$ |
| Widths: Challenge, Slow | $0.204$ / $0.928$ |
| Challenge-driven variance per crossover | $0.0533$ |

Every constant above is derived, not declared: the differentials from the cost tables (`def:landscape:reward-differentials`), the offsets and sensitivities from the decomposition (`eq:landscape:rigid-decomposition`), the widths as differences of adjacent offsets, and the per-crossover challenge variance as the squared sensitivity times the posterior variance. None of them depends on the risk estimate, which is why they are stated once here and not repeated in each example. Each example then reports its evidence sources, its crossovers, its regimes, and its fragility at representative postures; the same four landscapes rendered appear with the rendering layer (`ex:rendering:examples`).

**Example (A low-risk known entity)** · `ex:landscape:low-risk`

Risk basis: $\hat{p} = 0.05$, $\sigma_\text{eff} = 0.42$, $\kappa_\text{eff} = 1.0$. Evidence sources: $u = \ln(0.95/0.05)/2.5 = 2.944/2.5 = 1.178$ and $\sigma_u = 0.42/2.5 = 0.168$, so $\sigma_u^2 = 0.0282$; the challenge posterior is the uniform prior.

| Transition | Location, logit (posture) | Variance | Interval, logit | Interval, posture |
| --- | --- | --- | --- | --- |
| Allow to Challenge | $0.330$ ($0.582$) | $0.0816$ | $[-0.27,\; 1.86]$ | $[0.43,\; 0.87]$ |
| Challenge to Slow | $0.534$ ($0.630$) | $0.0816$ | $[-0.06,\; 2.06]$ | $[0.48,\; 0.89]$ |
| Slow to Block | $1.462$ ($0.812$) | $0.0816$ | $[0.87,\; 2.99]$ | $[0.70,\; 0.95]$ |

| Action | Width | Width sd | Domination probability |
| --- | --- | --- | --- |
| Allow | boundary | — | $0$ |
| Challenge | $0.204$ | $0.462$ | $0.375$ |
| Slow | $0.928$ | $0$ | $0$ |
| Block | boundary | — | $0$ |

Fragility at Normal reads modal action Allow with flip probability about zero, the nearest crossover being $4.1$ standard deviations away; at posture $0.6$, where $\ell = +0.405$, it reads modal action Challenge with flip probability about $0.72$ and a challenge share of about $0.65$.

The reading is the point of the example. The risk evidence is strong and the challenge evidence is absent, and the landscape says so structurally rather than in a caveat: the challenge term is $0.0533$ of each crossover's $0.0816$, the Challenge regime's width is less certain than its own value, and Challenge is dominated with probability $0.375$. The Slow width, by contrast, is exact. The single highest-value evidence investment on this channel is challenge-outcome observability, not labels — a conclusion no component could reach alone.

**Example (A high-risk new entity)** · `ex:landscape:high-risk`

Risk basis: $\hat{p} = 0.72$, $\sigma_\text{eff} = 0.74$, $\kappa_\text{eff} = 1.0$. Evidence sources: $u = \ln(0.28/0.72)/2.5 = -0.944/2.5 = -0.378$ and $\sigma_u = 0.74/2.5 = 0.296$, so $\sigma_u^2 = 0.0876$; the uniform prior again.

| Transition | Location, logit (posture) | Variance | Interval, logit |
| --- | --- | --- | --- |
| Allow to Challenge | $-1.226$ ($0.227$) | $0.141$ | $[-2.16,\; -0.02]$ |
| Challenge to Slow | $-1.022$ ($0.265$) | $0.141$ | $[-1.95,\; 0.19]$ |
| Slow to Block | $-0.094$ ($0.477$) | $0.141$ | $[-1.03,\; 1.12]$ |

The regime table is *identical* to the first example — Challenge at width $0.204$, standard deviation $0.462$ and domination probability $0.375$; Slow at width $0.928$ and standard deviation zero — and that identity is what the example exists to show. Only the locations moved, all three by the same $-1.556$, which is the change in the shared term and nothing else (`thm:landscape:rigid-translation`).

Fragility at Normal reads modal action Slow with flip probability about $0.31$, the two bounding crossovers sitting at $-1.022$ and $-0.094$; at Elevated it reads Block with flip probability about $0.09$. The higher effective uncertainty inflates every crossover variance to $0.141$, so the challenge contribution falls to about thirty-eight per cent and the two evidence sources now contribute comparably — the same channel, the same policy, and a different answer to what should be bought next.

**Example (Medium risk under high uncertainty)** · `ex:landscape:medium-risk`

Risk basis: $\hat{p} = 0.35$, $\sigma_\text{eff} = 0.88$, $\kappa_\text{eff} = 1.0$. Evidence sources: $u = \ln(0.65/0.35)/2.5 = 0.619/2.5 = 0.248$ and $\sigma_u = 0.88/2.5 = 0.352$, so $\sigma_u^2 = 0.124$; the uniform prior.

| Transition | Location, logit (posture) | Variance | Interval, logit |
| --- | --- | --- | --- |
| Allow to Challenge | $-0.600$ ($0.354$) | $0.177$ | $[-1.62,\; 0.52]$ |
| Challenge to Slow | $-0.396$ ($0.402$) | $0.177$ | $[-1.42,\; 0.72]$ |
| Slow to Block | $+0.531$ ($0.630$) | $0.177$ | $[-0.49,\; 1.65]$ |

The regimes are again unchanged: Challenge at $0.204$ with standard deviation $0.462$ and domination probability $0.375$, Slow at $0.928$ with standard deviation zero.

This is the genuinely uncertain assessment, and it inverts the first example's conclusion. The risk term dominates every crossover variance — $0.124$ of $0.177$, about seventy per cent — so here the binding deficit is labels and not challenge outcomes, and the fragility decomposition attributes it that way. Fragility at Normal reads Allow with flip probability about $0.20$; at posture $0.5$, where the logit is zero, it reads Slow with flip probability about $0.35$. Almost every operating point sits near a boundary. This is exactly the request the Core's own guidance would rank highest for labelling (`def:guidance:risk-informative`), and the landscape confirms it from the decision side rather than by coincidence.

**Example (A three-action channel)** · `ex:landscape:three-action`

The same risk basis as the first example — $\hat{p} = 0.05$, $\sigma_\text{eff} = 0.42$, $\kappa_\text{eff} = 1.0$, the uniform prior — with Slow removed from the declared set. The evidence sources are unchanged at $u = 1.178$ and $\sigma_u = 0.168$, both being risk-only quantities. Allow to Challenge is unchanged because its differential pair is unchanged; Challenge to Block uses the three-action differentials $2.5$ and $1.5$, giving an offset of $\ln(2.5/1.5)/2.5 = +0.204$.

| Transition | Offset | Sensitivity | Location, logit (posture) | Variance | Interval, logit |
| --- | --- | --- | --- | --- | --- |
| Allow to Challenge | $-0.848$ | $-0.8$ | $0.330$ ($0.582$) | $0.0816$ | $[-0.27,\; 1.86]$ |
| Challenge to Block | $+0.204$ | $+0.8$ | $1.382$ ($0.799$) | $0.0816$ | $[0.44,\; 2.51]$ |

| Action | Width | Width sd | Domination probability |
| --- | --- | --- | --- |
| Allow | boundary | — | $0$ |
| Challenge | $1.052$ | $0.462$ | $0.067$ |
| Block | boundary | — | $0$ |

Two further sensitivities hold at these defaults and are stated as oracle figures rather than derived again. A crossover's location moves by $1/(\beta_b + \beta_g) = 0.400$ logit units per unit change in the logarithm of its governing differential ratio, identically for every transition; and a crossover's location moves by its own challenge sensitivity, $\pm 0.800$, per unit change in the challenge estimate. Both follow from the decomposition and neither depends on the action set.

Removing Slow turns Challenge from a $0.204$-wide sliver into a $1.052$-wide regime, a factor of $5.2$, and drops its domination probability from $0.375$ to $0.067$: at the uniform prior Challenge is very likely viable in the three-action channel and close to a coin flip in the four-action one. The width uncertainty is unchanged at $0.462$, since it depends only on the difference of adjacent sensitivities and the posterior variance (`prop:landscape:width-variance`). This is the decision-theoretic content behind the sizing guidance (`cav:channel:action-space-sizing`): declare Slow only if it is a genuinely distinct operational response, because its presence costs Challenge most of its regime.

**Summary (How the landscape treats each design concern)** · `summ:landscape:design-summary`

| Aspect | Treatment |
| --- | --- |
| Inputs | Risk assessment, channel policy, challenge posterior (`sig:landscape:derivation-function`) |
| Purity | Deterministic, stateless, closed form, presentation-free (`pf:landscape:purity`) |
| Reward differentials | Per-action cost tables with two extended parameters, fully reproducible (`def:landscape:reward-differentials`) |
| Rigid translation | Exact separation into a shared term and per-transition offsets; widths risk-independent (`thm:landscape:rigid-translation`) |
| Crossover covariance | Rank two, closed form; widths carry zero risk variance (`prop:landscape:width-variance`) |
| Dominance | Exact incomplete-beta probability, reported and never enforced (`thm:landscape:dominance`) |
| Credible intervals | Quantile push-through, exact in the challenge effectiveness; honest wide-but-bounded cold start (`def:landscape:credible-intervals`) |
| Optimal action | Upper-envelope recovery, correct under inverted crossovers (`alg:landscape:optimal-action`) |
| Outcome neutrality | Axis prediction outputs never reach the derivation (`def:landscape:outcome-neutrality`) |
| Multiple channels | One assessment, one derivation call per channel; the Core's cost paid once (`ex:architecture:compositional-dividends`) |
| Replay | The three stored inputs reproduce the landscape exactly (`pf:landscape:purity`) |

Eleven aspects, and the table restates rather than decides: every row's content is fixed at the environment it cites, and a reader who finds the two disagreeing should believe the citation. It is collected here because the four examples above are the only place the whole design is exercised at once, and a summary adjacent to worked figures is checkable in a way that a summary standing alone is not.

### Chapter (Exploration and Label Guidance) · `chap:spec:exploration-and-guidance`

Two exploration signals answer two different questions, and the chapter keeps them apart because a host that conflates them will buy the wrong evidence.

Decision fragility asks whether resolving *this* decision at *this* posture would change what the host does. It is a function of the landscape and needs no model state. Core label guidance asks which pending requests would most improve the Core's own estimates, and it is a function of model state and needs no landscape. The first is a decision-quality signal, the second a learning signal; they rank differently, they are computed in different components, and the last environment of the chapter is the only place they meet.

#### Decision fragility · `sec:fragility:decision`

The division carries no material of its own. It collects the fragility record, the shares that attribute its uncertainty to a source, and the bands that say what a value of it is worth acting on.

**Definition (Decision fragility)** · `def:fragility:definition`

Fragility is read from a landscape at a posture and reports four things.

| Field | Carrying |
| --- | --- |
| Posture | The posture the reading was taken at |
| Modal action | The action optimal at the point-estimate crossovers |
| Flip probability | The probability that the true optimal action differs from the modal one |
| Risk share | The fraction of the bounding-crossover variance from the shared risk term |
| Challenge share | The fraction from the challenge posterior |

The modal action is recovered from the cost-curve envelope (`alg:landscape:optimal-action`), so a posture lying inside a dominated action's inverted region is handled correctly rather than reported as that action. The flip probability is the mass of the bounding crossover distributions lying on the far side of the posture's logit; for an interior modal action both boundaries contribute and the rank-two covariance governs the joint computation (`thm:landscape:crossover-covariance`), and inside an inverted region the crossovers bounding the *optimal* regime from the envelope are used rather than the raw adjacent records. The default computation uses the Gaussian approximation on the crossover marginals, and an implementation may substitute exact evaluation in the challenge effectiveness by quantile integration where the posterior is conjugate (`cav:limitation:fragility-gaussian`).

The two shares are the reason fragility is worth computing at all. They split the bounding variance into its risk-driven and challenge-driven parts and so answer the operational question directly: to firm up this decision at this posture, buy labels or buy challenge-outcome observability. No single component can make that call, because neither the Core nor the Companion can see the other source. The five-field fragility record is read from a landscape at the caller's posture; the separate rendered-profile utility reports the three Shannon entropies that measure display ambiguity instead (`sig:rendering:ambiguity-gauge`).

**Table (Reading a flip probability)** · `tab:fragility:interpretation`

| Flip probability | Reading | Label priority |
| --- | --- | --- |
| Above $0.35$ | The decision is effectively unresolved at this posture | Highest |
| $0.15$ to $0.35$ | The boundary is within reach of realistic evidence | High |
| $0.05$ to $0.15$ | The modal action is stable against moderate surprise | Moderate |
| Below $0.05$ | Labelling will not change the action | Low |

Fragility is a first-order proxy for value of information, and the bands are calibrated to that reading rather than to any statistical convention. An observation has decision value only when it might change the action taken, so a request deep inside a regime has near-zero fragility however uncertain its risk estimate is in absolute terms: a high probability-scale uncertainty far from any boundary changes nothing about what to do, and buying a label for it buys knowledge and no decision.

This is the essential difference from an entropy-style ambiguity gauge, which weights uncertainty by nothing at all. Fragility weights it by decision sensitivity, which is why the two rank the same population differently and why the substitution is not a rename (`inv:guarantee:exploration`). Pure model-uncertainty reduction is covered independently, and better, by the Core's own guidance (`def:guidance:risk-informative`); the two signals are complements and a host with budget for both should spend on both.

#### Core label guidance · `sec:guidance:core`

The division carries no material of its own. It collects the guidance interface, its three scoring criteria, the policy that keeps the three lists independent, and the pattern that composes them with fragility.

**Signature (The label guidance interface)** · `sig:guidance:interface`

The Core answers a budget and a small parameter set with three ranked lists of candidate requests. It reads its own state only; no landscape, no policy, and no posture reach it.

| Group | Field | Carrying |
| --- | --- | --- |
| Budget | Risk-informative, investigation, starvation-relief | How many candidates each list may return |
| Parameters | Scan limit | How many stored requests are examined |
| Parameters | Starvation threshold | The score below which a starvation candidate is not offered |
| Requests | Three lists | The candidates, one list per category |
| Candidate | Assessment identifier, entity key, timestamp, score, cross-membership | What a caller needs to act on one candidate and to see where else it appears |

Three properties are load-bearing. The budget is per category rather than global, so a host can buy learning in one direction without starving another. The scan limit bounds the work: guidance reads stored assessments and never recomputes a model, so its cost is linear in the limit and independent of the model dimension. And the candidate carries the assessment identifier rather than a copy of the assessment, so a label reported against it rejoins the stored entry by identity (`tab:config:guidance`).

**Definition (The risk-informative criterion)** · `def:guidance:risk-informative`

Risk-informative candidates are the requests the Core is least certain about. The score is the probability-scale uncertainty carried on the stored risk basis (`schema:risk:basis`):

$$\text{score}_\text{risk}(r) = \sigma_{\hat{p}, r}$$

The criterion measures exactly what a label would reduce. It needs no recomputation, since the uncertainty was computed at assessment time and stored, and it needs no reference to any decision: a high-uncertainty request is informative about the model whether or not it sits near any action boundary, which is precisely the difference from fragility (`def:fragility:definition`). A host buying only on this criterion learns efficiently and may never resolve the decisions it actually faces; a host buying only on fragility resolves its boundaries and may leave the model uncertain everywhere else.

**Definition (The investigation criterion)** · `def:guidance:investigation`

Investigation candidates are the requests where the Core suspects danger and lacks unconfounded evidence. The score is the anchor's blend weight from the stored risk basis.

The reasoning is indirect and worth stating. A high blend weight means the anchor is carrying much of the estimate, which happens when the sister model is uncertain, which happens where unconfounded outcomes have not reached — the regions the host has been restricting. Those are exactly the requests where an ordinary label says little, because the outcome was determined by the host's own action, and where an investigated outcome carrying the ground-truth flag says a great deal (`conv:eligibility:ground-truth`).

The criterion therefore ranks by the *shape* of the evidence rather than by its quantity, and it is the only guidance category whose value depends on the host having an investigation capability at all. A host without one should set its budget to zero rather than receive candidates it cannot act on (`cav:limitation:investigation-pipeline`).

**Definition (The starvation-relief criterion)** · `def:guidance:starvation`

Starvation-relief candidates are requests routed through Sentinel cells whose outcome memory is acting on a reputation that eligible feedback has not tested. The score takes the worst such cell across the reporting Sentinels:

$$\text{score}_\text{starv}(r) = \max_s \bigl(\bar{b}_{s,\text{cell}} - P_+^\text{eligible}\bigr)^+ \cdot \left(1 - \frac{n_{\text{elig},s,\text{cell}}}{N_\text{ledger}}\right)^+$$

The two factors are the divergence of the cell's remembered adverse rate above the system-wide eligible base rate, and the shortfall of its eligible label count against the count at which that rate would be trusted. Both are clipped below at zero, so a cell that is unremarkable or well fed scores nothing, and the product is what identifies the closed loop: a reputation the system is acting on, held up by evidence too thin to have tested it, and untested because the system is acting on it (`def:ledger:starvation-score`).

The criterion is the fourth mitigation layer of the contamination loop, and the only one that acts rather than waits — the others bound how long a stale reputation persists, this one buys the evidence that would settle it (`alg:valence:contamination-loop`).

**Decision (Independent lists rather than deduplication)** · `dec:guidance:no-deduplication`

Each list is sorted by its own score and truncated to its own budget, and a request appearing in two categories appears in both. It is not deduplicated; instead the candidate records which other categories it belongs to.

The decision is recorded because deduplication is the obvious alternative and is wrong in a way that is easy to miss. The three scores are not comparable — one is a probability-scale standard deviation, one a blend weight, one a product of a rate divergence and a count shortfall — so a deduplicating pass would have to rank across incomparable scales, and whichever rule it chose would silently reweight the host's declared per-category budget. Independent lists keep each budget meaning what the host set it to mean.

The cross-membership field then recovers everything deduplication would have offered, and more: a request that is uncertain *and* starved *and* worth investigating is visible as such, and a host that wants to prefer such requests can, while a host that wants exactly the budget it asked for in each category still gets it.

**Example (Composing fragility with Core guidance)** · `ex:fragility:composition`

A host wanting action-informative guidance composes the two halves itself. It assesses its pending requests, derives a landscape per assessment against its channel policy and the Companion's current posterior, reads fragility at its operating posture, and ranks by flip probability — routing each request's budget by whichever share dominates its bounding variance.

This is a composition pattern and not a Core interface, and the boundary is deliberate. The Core cannot compute fragility, because fragility depends on the host's declared costs and posture, which the Core is specified not to know (`prin:principle:measure-not-decide`). The derivation cannot compute the Core's guidance, because that depends on stored model state a pure function does not hold (`pf:landscape:purity`). Composition is where the two meet, and the host is the only place it can happen.

The routing is the part no single-component signal could produce. A high risk share sends the budget to labelling; a high challenge share sends it to challenge-outcome observability instead — more challenges executed and reported, or an override or injection where the host has evidence from elsewhere (`alg:companion:override`) and (`alg:companion:injection`). The host composes the two landscape utilities with the Core guidance lists (`sig:landscape:utilities`).

## Part (The Companion Tracker) · `part:spec:companion-tracker`

Part IV took a challenge posterior as one of the derivation's three inputs and specified the boundary it crosses without saying where it comes from. Part V specifies the component that maintains it: a Beta-Binomial conjugate model over the binary evidence stream of challenge outcomes, decaying toward its prior as that evidence ages, and answering with a point estimate, a variance, and — where the host wants exactness rather than moments — the conjugate shape itself.

The Companion is the smallest component in the architecture and the only one that can be replaced wholesale without touching anything else. It reads no Core state, holds none, and is not reached by the Core at any point; the host feeds it, reads it, and hands what it reads to the derivation. That independence is the Part's organising claim, and the last environment of the chapter is where it is stated as an obligation rather than described as a habit.

### Chapter (Challenge Effectiveness Tracking) · `chap:spec:challenge-effectiveness`

The chapter fixes what challenge effectiveness is, how it is estimated, how the estimate ages, and how a host that knows better than the estimate says so. The order is the order of the model's own life: what the quantity means, what state carries it, what the state begins as, how it is read, how it is written, how thin the evidence is, how it decays, and the two recorded decisions that fix the decay's form and its rate. The override and the injection follow, because they are the answers to the sparsity the rate environment quantifies, and the health report, the convergence bound and the boundary invariant close the chapter with what a host can see, what it must wait for, and what it may rely on.

**Definition (Challenge effectiveness)** · `def:companion:purpose`

Challenge effectiveness $q_c$ is the probability that a challenge correctly identifies an adverse source. It is a property of the challenge mechanism, not of any request, and it enters the decision landscape by two separate channels.

The point estimate $\hat{q}_c$ moves the crossover positions: a high value moves the boundary between allowing and challenging toward the permissive end, because challenges are worth deploying earlier when they work, and a low value moves it the other way (`eq:landscape:crossover`). The posterior variance $\sigma^2_{\hat{q}_c}$ widens the crossover credible intervals through the posture-sensitivity terms, expressing uncertainty about where on the posture axis each boundary belongs (`def:landscape:credible-intervals`). The posterior's *shape*, where it is available, governs the dominance probabilities exactly rather than through a moment approximation (`thm:landscape:dominance`).

Two channels, one quantity, and neither of them a Core concern: the Core's risk estimate is unchanged by any value of $q_c$ whatsoever.

**Definition (The tracker's state)** · `def:companion:state`

A tracker carries five things: two pseudo-counts, the time of its last write, its configuration, and an optional host override.

| Field | Carrying |
| --- | --- |
| $\alpha$ | The pseudo-count of challenge failures — an adverse source identified |
| $\beta$ | The pseudo-count of challenge passes — an adverse source not identified |
| $t_\text{last}$ | The time of the most recent write, from which elapsed-time decay is measured |
| Configuration | The two prior pseudo-counts, the decay rate, and the injection ceiling |
| Override | The host-supplied estimate and variance, when one is in force |

The host creates one tracker per channel, or shares one across channels whose challenge mechanisms are identical, and nothing in the state refers to the Core: no model parameters, no feature vector, no Ledger entry, no assessment identifier. The initial state is the prior (`def:companion:prior`).

**Definition (The prior)** · `def:companion:prior`

The default prior is $\text{Beta}(1, 1)$: the uniform distribution on the unit interval, expressing no directional belief about whether challenges work. Its point estimate is $0.5$ and its variance is $1/12$. A host with a view configures the two prior pseudo-counts to express it.

| Prior | $\hat{q}_{c,0}$ | $\sigma^2_0$ | The belief it expresses |
| --- | --- | --- | --- |
| $\text{Beta}(1, 1)$ | $0.50$ | $0.0833$ | No prior knowledge; the default |
| $\text{Beta}(2, 2)$ | $0.50$ | $0.0500$ | Moderate effectiveness expected |
| $\text{Beta}(5, 2)$ | $0.71$ | $0.0256$ | Challenges usually work |
| $\text{Beta}(2, 5)$ | $0.29$ | $0.0256$ | Challenges usually fail to catch |
| $\text{Beta}(5, 5)$ | $0.50$ | $0.0227$ | Confident that effectiveness is moderate |

The prior is also the point the decay returns to, which is why a host choosing one is choosing two things at once: where the estimate starts and where it ends if the evidence stops (`alg:companion:decay`).

**Algorithm (Reading the estimate)** · `alg:companion:inference`

A read returns the posterior at the moment of reading. The pseudo-counts are first decayed to the present as a pure function of elapsed time, without mutating anything, and the moments follow from the decayed pair:

$$\hat{q}_c = \frac{\alpha}{\alpha + \beta}, \qquad \sigma^2_{\hat{q}_c} = \frac{\alpha\beta}{(\alpha + \beta)^2(\alpha + \beta + 1)}$$

The effective sample size is $n_\text{eff} = (\alpha + \beta) - (\alpha_0 + \beta_0)$: the data-contributed pseudo-counts net of the prior. Two reads are specified over the same state. One returns the moments, for a host consuming a point and a variance directly. The other returns the posterior the derivation takes as its third input, carrying the conjugate shape where no override is in force and a moment-matched summary where one is (`sig:companion:posterior`). Reading decays and does not write, which is the Ledger's convention applied to a second component for the same reason: a read must not be a write (`alg:temporal:lazy-application`). Each estimate, posterior and health read computes the decayed pseudo-counts as locals, so an idle tracker regresses toward its prior without the read mutating stored state.

**Algorithm (The conjugate update)** · `alg:companion:update`

On a contributing label the stored counts are decayed to the label's timestamp and then incremented by one, a failure to $\alpha$ and a pass to $\beta$:

$$\alpha \leftarrow \alpha + \mathbb{1}[\text{fail}], \qquad \beta \leftarrow \beta + \mathbb{1}[\text{pass}]$$

This is the exact conjugate update for a Beta prior under Binomial evidence. No approximation accumulates over observations, however many arrive and in whatever order, which is why the tracker's uncertainty can be trusted at every sample size rather than only asymptotically.

A label contributes when three conditions hold jointly: the host challenged, the outcome was adverse under the sign convention (`conv:valence:sign`), and the challenge produced an observable result. A challenge of a benign request satisfies the first and third and is not evidence, because $q_c$ is defined against adverse sources specifically and a benign request passing a challenge says nothing about the mechanism's catch rate. Feeding the tracker is the host's duty; nothing in the Core routes a label here.

**Data (The contributing-label rate)** · `data:companion:contributing-rate`

The evidence base is thin, and the arithmetic that says how thin is worth carrying because it determines everything else in the chapter:

$$R_\text{contributing} = R_\text{label} \times f_\text{chall} \times f_\text{adv|chall} \times f_\text{obs}$$

the factors being the total label rate, the fraction of labelled requests challenged, the fraction of those with an adverse outcome, and the fraction with an observed result. At a one-percent challenge rate, a five-percent adverse rate and eighty-percent observability, four requests in ten thousand contribute.

| Targeting quality | $f_{\text{adv\|chall}}$ | Contributing labels per day | Days for twenty to arrive |
| --- | --- | --- | --- |
| Untargeted, at the population rate | $0.05$ | $0.08$ | $250$ |
| Weakly targeted | $0.15$ | $0.24$ | $83$ |
| Moderately targeted | $0.30$ | $0.48$ | $42$ |
| Well targeted | $0.50$ | $0.80$ | $25$ |

The last column counts arrivals and not what survives them, which is the distinction the convergence bound turns on: at the untargeted rate twenty contributing labels arrive and twenty effective samples never stand together (`bound:companion:convergence`).

The table is stated at two hundred labels a day. Targeting is the only factor a host can move by an order of magnitude, and moving it is exactly what the landscape is for, so a deployment that uses the landscape to aim its challenges buys its own evidence six to ten times faster than one that does not.

**Algorithm (Pseudo-count decay)** · `alg:companion:decay`

At each write, with $\Delta t$ the hours since the last one, the stored counts are blended toward the prior before the write is applied:

$$\alpha \leftarrow \alpha_0 + \gamma_{q,t}^{\Delta t}(\alpha - \alpha_0), \qquad \beta \leftarrow \beta_0 + \gamma_{q,t}^{\Delta t}(\beta - \beta_0)$$

Data-contributed pseudo-counts decay exponentially and the prior does not, so the point estimate regresses toward the prior mean and the variance rises toward the prior variance. Both are correct when evidence goes stale: an estimate held up by year-old observations should say so in its uncertainty and should not go on asserting a mean it no longer has grounds for.

| Time since the last contributing label | Evidence retained |
| --- | --- |
| One week | $96.6\%$ |
| One month | $86.7\%$ |
| Three months | $65.2\%$ |
| Six months | $42.5\%$ |
| One year | $17.3\%$ |

At a constant arrival rate $\lambda_c$ per day the steady-state effective sample size is $n_\text{eff,ss} = \lambda_c / (-24 \ln \gamma_{q,t})$: about $167$ at $0.8$ contributing labels a day, and about $17$ at $0.08$.

**Decision (Blending toward the prior rather than scaling the counts)** · `dec:companion:decay-form`

The alternative form multiplies both counts by the decay factor and floors them at the prior, preserving their ratio exactly. It answers a narrower question — how certain am I about what I last measured — and leaves the point estimate where it was. It is the right form when the challenge mechanism is known to be stable and only precision degrades with age.

The blend toward the prior is specified as the default because it makes the weaker assumption. Stale evidence should cost the host its confidence *and* its position, and a host that wants to keep the position has to say so.

The choice is not free. Blending pulls the steady-state point estimate toward the prior mean: at $n_\text{eff,ss} = 17$ and a true $q_c$ of $0.9$ the bias is about $-0.04$, displacing the allow-to-challenge crossover by roughly one hundredth of a logit; at $n_\text{eff,ss} = 167$ it is about $-0.005$ and disappears into the noise. A host with a stable mechanism and a thin evidence stream may prefer the multiplicative form, and that is a configuration choice rather than a change of default.

**Decision (The decay rate)** · `dec:companion:decay-rate`

The rate fixes the trade between steady-state precision and responsiveness to a mechanism that has actually changed, and the two pull in opposite directions over the same parameter.

A faster rate — a twenty-nine-day half-life, say — would notice a mechanism change within weeks, at the cost of a steady-state effective sample size near three at the pessimistic arrival rate, where the prior would dominate permanently and the tracker would be an expensive way to return $0.5$. A slower rate — a five-hundred-and-seventy-eight-day half-life — would accumulate more evidence and go on asserting a catch rate the mechanism stopped having years earlier.

The default sits between them on the assumption that challenge mechanisms change on a timescale of months. A host that has just changed its own mechanism should not wait for the decay to discover it from sparse evidence; it should override or inject (`alg:companion:override`) and (`alg:companion:injection`).

**Definition (The challenge-effectiveness decay rate)** · `def:companion:challenge-decay`

The rate is $\gamma_{q,t} = 0.9998$ per hour, a half-life of about one hundred and forty-five days. It is the Companion's own parameter and belongs to the Companion: it governs how fast challenge-outcome evidence ages and nothing else, and it is independent of every Core rate, none of which it is derived from or constrained by.

The siting matters because the rate is easy to mistake for a property of the channel it is read against. It is not. Two channels sharing one challenge mechanism share this rate because they share the tracker, and two channels with different mechanisms differ in it because they have different trackers — in neither case does the channel's declared reward structure have anything to say about it. The rate therefore belongs to the Companion tracker configuration and not to the channel policy's reward parameters (`tab:channel:reward-parameters`).

**Algorithm (The host override)** · `alg:companion:override`

A host that knows the catch rate better than the evidence does may say so. An override supplies a point estimate and, optionally, a variance; while it is in force both reads return the supplied values, the second as a moment-matched posterior rather than a conjugate one. Where the variance is omitted the default is $q_c(1 - q_c)/101$ — the variance of about a hundred pseudo-observations at the overridden value, which is confident without being absolute and is why the moment-matched posterior downstream is faithful rather than degenerate.

Beneath the override the Beta model goes on accumulating and decaying as usual, so clearing the override reveals a model that has been learning throughout rather than one frozen at the moment of the override.

The override is the single highest-leverage action available at deployment. An untargeted deployment never reaches an effective sample size of twenty at all, and a weakly targeted one waits about a hundred and eight days for it (`bound:companion:convergence`); a host with a pilot study, a historical figure, or an informed guess waits for neither, and leaves the Core's own convergence as the only thing to wait for (`bound:resource:convergence-budget`). A host using the derivation should override at deployment unless it truly has no basis for an estimate.

**Algorithm (Evidence injection)** · `alg:companion:injection`

Injection adds pseudo-counts directly: so many failures, so many passes, decayed to the present first and then added. Both arguments must be non-negative and neither need be an integer, since discounted external evidence is naturally fractional. Injected counts age at the same rate as observed ones, because they are the same kind of thing.

A ceiling, one thousand by default, caps each injection; a larger value is clamped and the clamping is reported. Without it a single injection could seat itself so deep that the decay would take years to reach it, which would make the non-stationarity handling of this chapter decorative.

Injection and override are independent and compose. An override governs what the reads return; an injection changes what the model knows. A host with a precise external figure overrides, a host with external counts injects, and a host with both injects the counts, overrides with the figure, and clears the override once the model has absorbed the evidence.

**Table (The Companion health report)** · `tab:companion:health`

The tracker reports its own health, separately from the Core's, because it converges separately and fails separately.

| Field | Carrying |
| --- | --- |
| Estimate and variance | The current posterior moments, from the override where one is in force |
| Pseudo-counts | The raw $\alpha$ and $\beta$, always from the Beta model |
| Effective sample size | The data-contributed counts net of the prior |
| Sufficiency verdict | Whether that effective sample size has reached the host-owned floor, twenty by default, which an override does not bring forward |
| Prior contribution | The prior's share of the total mass, so a host can see when it dominates |
| Contributing labels, lifetime | How much evidence has ever arrived |
| Contributing rate | An exponentially weighted estimate of arrivals per day |
| Variance contribution fraction | The share of crossover variance attributable to $\hat{q}_c$ |
| Override state | Whether an override is in force, and its values |
| Days since the last contributing label | Wall-clock silence, which the decay is measured in |

The variance contribution fraction is the most actionable field. Above one half, the crossover intervals are dominated by challenge uncertainty and the host should override, inject, or challenge more; between a tenth and a half it is a material contributor worth watching; below a tenth it is negligible. Its per-request generalisation is the landscape's challenge share (`def:fragility:definition`), and this is the fleet-level summary of the same quantity. The ten rows together distinguish a thin tracker from a stale one.

The sufficiency verdict is a comparison and not a second sample-size field. Its floor belongs to the Companion beside the tracker whose health report reads it, rather than to the Core convergence configuration (`dec:challenge:sufficiency-floor-owned-here`), and what the verdict compares against that floor is the override-independent effective sample size this chapter's convergence bound fixes (`bound:companion:convergence`). Reporting it gates nothing (`dec:health:reports-never-gates`).

**Bound (Convergence)** · `bound:companion:convergence`

The Companion converges independently of the Core. Counting contributing labels as they arrive and ignoring what decays between them, the time for a target effective sample size to arrive is the target over the contributing rate:

$$T_{\hat{q}_c} = \frac{n_\text{target}}{R_\text{contributing}}$$

At twenty target observations this arrival time is two hundred and fifty days untargeted, eighty-three weakly targeted, forty-two moderately targeted, and twenty-five well targeted. An override is not a fifth entry of zero in that list. It makes the reported estimate usable at once (`alg:companion:override`), and that is the whole of what it shortens: the effective sample size the health report carries (`tab:companion:health`) is taken from the decayed pseudo-counts net of the prior whether or not an override is in force, so the evidence a channel must observe before it counts as sufficient accrues under an override at exactly the rate it would without one.

That arithmetic is the no-decay approximation and is kept here as one, because the correction it needs is not a rounding. Pseudo-counts decay between arrivals (`alg:companion:decay`), so a channel fed at a regular rate does not accumulate without limit. With one contributing label every $h = 24 / R_\text{contributing}$ hours the effective sample size approaches

$$C_+ = \frac{1}{1 - \gamma_{q,t}^{\,h}}$$

immediately after a contribution, and $C_+ - 1$ immediately before the next. A target above $C_+$ is never reached at that rate however long the host waits, and a target between the two is reached but not held between contributions:

| Targeting quality | Contributing labels per day | Ceiling $C_+$ | Time to $n_\text{eff} = 20$ |
| --- | --- | --- | --- |
| Untargeted, at the population rate | $0.08$ | $17.17$ | Never |
| Weakly targeted | $0.24$ | $50.50$ | $108$ days |
| Moderately targeted | $0.48$ | $100.49$ | $48$ days |
| Well targeted | $0.80$ | $167.15$ | $28$ days |

So twenty effective observations are reached and held by weakly, moderately and well targeted channels, and by no untargeted one. The two hundred and fifty days above is the time twenty contributing labels take to *arrive* at the population rate, not a time at which twenty of them are still standing: at that rate the decay carries evidence away faster than arrivals replace it well before twenty accumulates, and the ceiling is a little over seventeen. The three tiers that do converge take longer than the no-decay row says — a hundred and eight days rather than eighty-three, forty-eight rather than forty-two, and twenty-eight rather than twenty-five.

Full system convergence is the larger of the two components' times, so the Companion is usually what binds: the Core reaches useful estimates in about ten days at the reference configuration, and the Companion does not (`bound:resource:convergence-budget`). A host using assessments without the derivation is unaffected, because it never reads $q_c$ at all.

The bound also carries an exposure it cannot remove. The quantity is really the catch rate over the population the host chooses to challenge, and better targeting changes that population — plausibly toward more capable adversaries — which the tracker reads as drift and re-converges over its long decay horizon while the crossovers move under it (`cav:limitation:challenge-policy-loop`). An override inherits the same relativity rather than escaping it, being calibrated on some earlier population of its own. Both the sparsity and the policy loop are disclosed among the document's limitations (`cav:limitation:challenge-thin`) and (`cav:limitation:challenge-convergence`).

**Invariant (The Companion's boundary)** · `inv:companion:boundary`

The Companion has three interfaces and they are of three different kinds.

To the Core: none. It reads no model parameter, no feature vector, no Ledger entry and no Core state of any description, it writes nothing there, and it influences no risk estimate, no importance weight, no eligibility determination and no learned quantity. The Core, symmetrically, holds no Companion state (`inv:guarantee:companion-independence`).

To the derivation: one, read-only. At each derivation the host supplies the Companion's posterior as an argument; the derivation evaluates quantiles and tail probabilities from it and does not modify it, feed back into it, or retain anything derived from it (`sig:companion:posterior`).

To the host: one, bidirectional. The host feeds contributing labels, reads the estimate, the posterior and the health report, and may override or inject.

Three interfaces, one of them empty, is the whole boundary, and the emptiness is the load-bearing one: it is what lets the Companion be replaced, or omitted entirely, without any consequence for what the Core learns. The boundary appendix lists it alongside the document's other crossings (`app:spec:interface-map`).

**Requirement (The replacement surface)** · `req:companion:replacement-trait`

The Beta-Binomial tracker is one estimator of challenge effectiveness and the specification requires no more of a replacement than an interface: a method returning the scalar estimate, and a posterior method defaulting to a moment-matched Beta built from that estimate.

The defaulting is what makes the surface worth having. A provider of a scalar and a variance — an external analytics pipeline, a lookup table over historical trials, a constant a host is confident in — participates fully in the landscape's interval and dominance machinery at first-order fidelity without implementing anything further, while a provider holding a genuine conjugate posterior overrides the second method and gets exact quantiles. The only constraints are that the estimate lie strictly inside the unit interval and the variance be strictly positive.

Neither the Core nor the derivation is affected by which provider is in place, which is the point: this is the one extension point in the architecture where a host can substitute its own science. The provider trait admits both the exact decayed posterior and a scalar-only provider through the specified moment-matched default.

## Part (The Operational Cycle) · `part:spec:operational-cycle`

Parts II through V specified the three components one at a time: what the Core sees, what it learns, what the derivation computes, and what the Companion estimates. Part VI specifies them running. Observations enter, assessments leave, labels return and models move; two of those flows share state and must not wait on each other; and every decaying quantity in the system ages under one of two mechanisms whose rates are inventoried in one place.

The Part is written from the host's side of each interface. What the host declares, what it must promise, what it receives per request, what it reports back, what may block what, and what forgets at which rate — these are the six chapters, and their common subject is the contract rather than the arithmetic. Every environment that closes against the operational surface records its current position there. Label-time reconstruction now reproduces the raw vector its prediction used; the remaining departures are named at their own sites.

### Chapter (Runtime Declarations and the Host Contract) · `chap:spec:host-contract`

The chapter fixes what a host says before the system runs and what it promises while it does. Three categories of construction configuration, two lifecycle declarations the host initiates, the label-reporting contract every later mechanism reads, the pre-seeding path that shortens the cold start, and one piece of deployment guidance about severity. The sign convention every one of these depends on is fixed at the front of the document and is cited here rather than restated (`conv:valence:sign`).

**Requirement (What the host declares at construction)** · `req:host:construction`

The Core is constructed from three categories of configuration and no others: the model parameters, the eligibility policy, and the signal schema.

No channel policy. No reward parameters. No posture. No action space. The Core has no concept of a decision to be made, and the configuration surface is where that claim is either kept or quietly broken, which is why the absence is stated as a requirement rather than left as an observation. A host wanting decisions declares its channel policy to the derivation, separately and per channel, and may declare several against one Core (`sig:landscape:derivation-function`).

The separation is the construction-time expression of the principle that the Core measures and does not decide (`prin:principle:measure-not-decide`). It is what makes one Core serve many channels, and what makes a Core replaceable without renegotiating a single reward.

**Requirement (The eligibility policy)** · `req:host:eligibility-policy`

The eligibility policy is one declaration, made at construction, governing whether a challenge label without an investigated outcome counts as unconfounded evidence. It defaults to counting.

The declaration is needed because the Core's label interface does not carry the challenge result (`tab:host:label-reporting`), so the eligibility table cannot distinguish a challenge that caught something from one that did not (`tab:eligibility:training`). The policy therefore governs all non-ground-truth challenge labels uniformly: under the default they reach the sister and anchor models, and under its negation they are treated as confounded and do not. A host needing the finer distinction has a better instrument than this switch — it marks investigated challenge outcomes with the ground-truth flag, which bypasses the criterion for exactly the labels whose outcome was established rather than inferred (`conv:eligibility:ground-truth`).

The parameter is tabulated with the rest of the configuration surface (`tab:config:eligibility`). Challenge follows the declaration, while the other actions and ground-truth override follow the eligibility table.

**Requirement (Outcome axis registration)** · `req:host:axis-registration`

An outcome axis is registered by the host, at any time, before or after traffic has begun. Registration is a lifecycle event and not a configuration setting: it extends every non-anchor model in the system, and the extension is exact rather than approximate (`alg:registry:axis-registration`).

The registration record carries what the axis is, how its outcomes are compressed, how fast it forgets, whether it participates in spatial features, and which eligibility mode governs its training (`schema:registry:axis-record`). The host supplies all of it; the Core infers none of it and cannot, since an axis is a statement about what the host cares to predict.

Because registration may happen at any time, a deployment need not know its full set of axes in advance, which is the property that makes axes worth having at all: the alternative is a schema fixed before the first useful question has been asked. The protocol, its scaling limits and its lifecycle mechanics are specified where the registries are (`sec:registry:outcome-axis`).

**Requirement (The signal schema declaration)** · `req:host:schema-declaration`

The signal schema is declared once, at construction, and fixed thereafter. Each declaration names a signal, its shape, and whether it persists with the entity or lives for a request (`schema:signal:shapes`).

The schema is what sets the width of the signal block in the feature vector, so it is not a piece of configuration that can be revised while models hold parameters indexed against it: a signal added later would move every index after it, and the exactness the lifecycle machinery provides is defined over registrations, not over redefinitions of a declared block (`req:signal:schema-fixed`).

A host uncertain about its signals should declare the superset it might use rather than plan to extend, and accept the cost of a few dimensions that carry nothing. The shapes and persistence classes are defined in the signal chapter (`chap:spec:host-signals`).

**Table (The label-reporting contract)** · `tab:host:label-reporting`

A label is reported against a stored assessment and carries five fields.

| Field | Required | Carrying |
| --- | --- | --- |
| Assessment identifier | Yes | The stored assessment the outcome belongs to |
| Action taken | Yes | What the host actually did, as an observed fact |
| Valence | Yes | The outcome's magnitude, whose sign fixes the risk target |
| Outcomes | No | Per-axis values; an axis not reported is not updated |
| Ground truth | Yes | The host's assertion that the outcome was investigated |

The action taken need not match any action any landscape indicated. The host is free to do anything, including something no derivation suggested, and the Core learns from what was done rather than from what was advised — which is what makes the eligibility table a statement about evidence rather than about compliance (`tab:eligibility:training`).

The ground-truth flag is trusted unconditionally, because the Core has no means of checking it. A corrupted investigation pipeline that systematically mislabels outcomes corrupts the sister model directly and is not detectable by the document's monitoring, which sees labels and not their provenance (`cav:limitation:investigation-pipeline`) and (`cav:monitoring:integrity-scope`).

Two fields of the composed system's label flow are deliberately not here. The challenge result goes to the Companion, because challenge effectiveness is not a Core quantity (`alg:companion:update`). Host posture metadata goes nowhere, because no Core model reads it.

**Algorithm (Pre-seeding)** · `alg:host:pre-seeding`

A host holding historical outcomes may preload them, and they travel the ordinary label path rather than a shortcut: both class-rate trackers move, per-axis compression scales move, and every eligible model updates, under the same eligibility rules live labels obey (`alg:runtime:update-path`). Standardisation statistics move on that path only once the full-vector phase is in service (`alg:standardisation:batch-initialisation`). Before then the cold observation ramp owns the coordinate transition, so a pre-seeded label cannot race it and cannot be overwritten by it.

Pre-seeding is worth most at cold start, where it can carry a deployment through the first warm-up stages before a single live request arrives, so the first assessment is directional rather than a restatement of the prior (`tab:warmup:stages`). It also seats the importance-weight trackers at a real base rate instead of an assumed one, and can converge a severity axis ahead of traffic. Where the publication interval permits, a pre-seed batch publishes once at its end rather than once per historical label (`req:publication:interval`).

The constraints are the interesting part. A pre-seeded label needs a stored assessment to attach to, so either the buffer is pre-seeded in step (`def:runtime:pending-entry`) or a simplified path synthesises a feature vector from the historical record — which in turn requires the signal schema and the Sentinel registry to be declared first. Either arrangement uses the specified pending entry without adding a channel field to the Core.

**Remark (Combining risk and severity)** · `rem:host:severity-discrimination`

A deployment where a small loss and a large one demand different responses should register a severity axis before live operation, because the risk model will not supply the difference. Its target is binary and answers whether an outcome will be adverse, never how adverse (`def:risk:target`). Valence exponentially weighted averages carry severity into the features, but slowly: roughly twenty labels per entity and a thousand per cell before they mean anything (`data:ledger:attenuation`). The axis model gives a calibrated severity prediction from its first eligible labels instead (`def:axis:per-axis-model`).

Without pre-seeding the axis converges in about twice the model dimension in eligible labels — some thirteen days at the reference configuration — and until then severity discrimination rests on those near-zero averages (`bound:resource:convergence-budget`) and (`cav:limitation:severity-cold`). Pre-seeding with enough historical labels carrying the axis value closes the gap entirely (`alg:host:pre-seeding`).

The combination is the host's, deliberately. The Core keeps probability and severity apart because their weighting is a loss function and only the host holds one (`def:landscape:outcome-neutrality`). The simplest composition ranks by the risk probability times a host utility over the predicted severity. A host using the derivation has a more principled option: modulate the posture by predicted severity, reading the landscape higher for severe predictions and lower for mild ones. Because posture and prior log-odds are interchangeable on a channel, that is a shift of the read-point and not an override of the model (`prin:channel:relativity`). Neither alone suffices, and the separation is what keeps the choice visible (`cav:limitation:severity-path`).

### Chapter (The Assessment Interface) · `chap:spec:assessment-interface`

The chapter carries the Core's principal output and the path that produces it. Reports arrive and are published; requests arrive and are answered; between the two sit two routing indices, a pipeline that reads almost everything and writes almost nothing, and an output structure that is a determination of position rather than a recommendation of course. The chapter ends with the composition pattern, which is the only place in the document where all three components appear in one worked sequence.

**Algorithm (Sentinel report reception)** · `alg:runtime:report-reception`

A batch report arrives for one Sentinel and atomically replaces that Sentinel's cached report. Publication is a pointer swap: assessments already in flight continue reading the previous report, assessments beginning afterwards read the new one, and no assessment sees a half-replaced report or waits for the replacement to finish. Calls for one Sentinel are serialised; calls for different Sentinels proceed independently.

The report consumed here is the post-lift form, its coordinates already carried into the Core's internal width (`alg:encoding:coordinate-lift`). The Core reads the per-cell analyses, the scoring and maturity records, the scoring geometry, the coordination reports, the contour snapshot and the analysis-set summary, and does not read the per-sample scores, the inline health report or the wavelet portrait; what it does not read is enumerated where the boundary is verified (`tab:boundary:not-consumed`). Non-consumption is not a judgment of value: the portrait explains how a Sentinel's partition came to be, which is a direct Sentinel-to-host diagnostic and not a thing the Core could estimate.

Two further effects run inside the same call, before the swap: the report index is rebuilt (`def:runtime:report-index`) and the Ledger's cell set is maintained against the new report (`alg:runtime:cell-set-maintenance`).

**Requirement (The assessment interface)** · `req:runtime:assessment-interface`

The interface takes a batch of request contexts and returns one assessment per request. A context carries the entity key, a timestamp, the declared signals, and one coordinate per Sentinel.

Concurrent calls are the normal case and are not a degradation of one. Each request in a batch is processed independently: no cross-request state, no ordering dependency, and no consistency guarantee within the batch beyond each request seeing whatever state was current when it ran. A batch is an amortisation of call overhead, not a transaction.

There is no channel field, because the Core has no channels. One assessment is channel-independent and may be carried into any number of landscapes by successive derivations against different policies, which is what lets a single quadratic-cost assessment serve a dozen channels (`sig:landscape:derivation-function`).

**Algorithm (The assessment pipeline)** · `alg:runtime:assessment-pipeline`

The pipeline reads published model state acquired once at the start of the call, the current Sentinel reports, and the Ledger — the last through a decay computed at read time as a pure function rather than persisted (`def:ledger:time-decay`). For each request, in order:

1. **Route** the request's coordinates to receiving cells in each online Sentinel's cached report (`def:runtime:report-index`).
2. **Extract** the per-Sentinel features and compute the alarm summaries (`def:runtime:alarm-summary`).
3. **Observe identity.** Encode the entity key on each registered dimension, record the observation for deferred processing (`alg:publication:identity-draining`), read the current competitive set from published state, populate the competitive indicators, update the per-range measurement state, and populate the per-dimension and cross-dimension features (`def:keyspace:competitive-set`).
4. **Aggregate** across Sentinels and accumulate the sub-scores into the concordance window (`def:feature:template-aggregate`).
5. **Assemble, offer and standardise** the feature vector: fixed blocks directly, dynamic blocks from the extractions and the identity state, interactions by iterating the compiled triples (`alg:dimension:compilation-pipeline`). While cold initialisation is still active, offer the assembled raw vector to the standardisation ramp (`alg:standardisation:batch-initialisation`). Then standardise the whole against the means and variances of the snapshot acquired at the start of the call; acceptance of the offer may reach only a later snapshot.
6. **Estimate risk**: the two regressions, the blend weight, the blended raw estimate and its probability-scale image with uncertainty (`def:risk:subspace-blend`) and (`def:risk:probability`).
7. **Predict each active axis** (`tab:axis:inference-outputs`).
8. **Record** the request in the pending buffer (`def:runtime:pending-entry`).
9. **Return** the assessment (`schema:output:assessment`).

There is no derivation step. The pipeline ends at an assessment, and a host wanting a landscape asks for one separately. Two properties hold over the whole sequence: per-request independence, so that a batch of $n$ produces what $n$ single calls would, and the enumerated write set below.

**Invariant (The enumerated assessment-path writes)** · `inv:runtime:enumerated-writes`

The assessment path directly writes exactly the three pieces of shared state in this table and nothing else.

| Write | Where it happens | What it is |
| --- | --- | --- |
| Per-dimension measurement state | Identity observation | A bounded per-dimension update in the active competitive cells |
| The concordance window | Aggregation | A sharded accumulator taking one sub-score set |
| The pending entry | Recording | One insertion into a concurrent map |

No model parameter, no dimension map, no published standardisation statistic, no identity graph structure — importance, topology or competitive set — and no Ledger entry is written by an assessment, ever. The Ledger in particular is read through a pure decay computation rather than a decay-and-store, which is precisely what allows an assessment to consult outcome memory without taking a write lock (`def:ledger:time-decay`).

Beside those three an assessment may enqueue work to the thread that owns it: the identity observation, the signal-cache insertion, the cold standardisation observation and the per-Sentinel bootstrap observation. The two bootstrap accumulators may also advance while they are active. None of these changes the immutable snapshot the assessment acquired, and none of them is a write to a model.

What separates them from the label path is evidence authority rather than the absence of a trace. An assessment never advances an operational, sister, anchor or outcome-axis posterior, a Platt parameter or a calibration diagnostic, an outcome-memory value, or the continuing standardisation average: every one of those is taught by an outcome, and every one of them moves on the label path alone. An accepted cold or per-Sentinel observation may move observational geometry, and moves it only into a snapshot later than the one its own request was answered from (`alg:standardisation:batch-initialisation`). What this invariant promises is therefore not that repeated assessment leaves no residue — the ramp exists in order to leave it — but that nothing an assessment leaves behind was taught by an outcome.

The invariant is what makes the concurrency chapter's non-blocking claim checkable rather than aspirational: a reader can enumerate three writes and confirm each is behind a bounded or structural budget (`tab:publication:tiers`), and the enqueues are enqueues rather than mutations of the structures they feed, drained on the maintenance thread's cycle (`alg:publication:identity-draining`).

**Table (The numeric checkpoints)** · `tab:runtime:numeric-checkpoints`

The system takes six numbered numeric readings between its boundary and its models, placed from the boundary inward. The placement is definition rather than decision — where each reading is taken and what it does on a non-finite finding — and it lives here because the degradation record declines to own it (`cav:degradation:checkpoint-placement`). The first four are taken on every assessment, and what each sanitises is counted into that assessment's degradation context; the last two guard model state at the end of the label update path (`alg:runtime:update-path`), and each reverts the label's whole effect to the last published snapshot and counts the revert on the published health summary.

| Checkpoint | Where it is taken | Reading, and the response |
| --- | --- | --- |
| CP1 | The signal boundary, before the cache merge | A non-finite host signal value is counted and sanitised to zero |
| CP2 | Each Sentinel's extraction | An extraction carrying any non-finite value is zeroed whole, occupancy included, and the Sentinel recorded |
| CP3 | The assembled feature vector, after standardisation | A non-finite position is sanitised to zero and counted |
| CP4 | The model estimates, after the blend | A non-finite point estimate or uncertainty falls back to the prior, per model |
| CP5 | The working copy, after a label's updates | Any model with a non-finite mean entry, or a precision matrix a factorisation refuses, reverts the label; so does an update any model refused, the covariance it read having yielded no non-negative leverage |
| CP6 | The snapshot about to publish | A non-finite mean or covariance entry, or a non-positive covariance-diagonal entry, reverts likewise and a clean snapshot publishes |

All six are performed as placed. The two halves differ in posture on purpose: an assessment must answer, so its four checkpoints substitute and disclose; a label may be refused whole, so its two checkpoints revert rather than repair, and the reverted state is the last state known good — where the rebuild that produces it succeeds.

CP5's precision reading is a verdict rather than a diagonal scan, and it costs nothing extra to take. A strictly positive precision diagonal is necessary for definiteness and is not sufficient for it (`req:gaussian:positive-definiteness`), so a diagonal scan admits exactly the matrices this checkpoint stands between and the published snapshot; a spectral reading, or even a factorisation, taken per model per label would be cubic work on a path that is otherwise linear in the width. It does not have to be taken here, because it has already been taken: the recomputation check runs earlier on the same label (`alg:runtime:update-path`), so a model that rebuilt this label carries this label's verdict, and a model that did not carries its last rebuild's.

That stored verdict is current rather than stale, because every mutation between two rebuilds preserves definiteness. The bounded update adds a positive-semidefinite rank-one term, its leverage being non-negative or the update having been refused — and a refused update reverts the label through this checkpoint's other reading rather than through this one. The forgetting scales the matrix by a positive factor, which preserves the definite cone exactly. The replenishment clamp only raises diagonal entries (`req:gaussian:prior-replenishment-floor`). So a matrix a factorisation accepted at the last rebuild is accepted still. A model that has never rebuilt carries no verdict and is not read: its precision matrix is the prior, definite by construction. The non-finite scans stay beside the verdict and are not folded into it, because a not-a-number entry is not a definiteness question and no earlier factorisation's verdict speaks to it.

The revert is a rebuild of the working copy from the published snapshot, and the rebuild can itself be refused, because the snapshot's covariance may not be positive definite in the working arithmetic. That verdict is the factorisation's own, and there is one factorisation to take it from (`dec:posterior:repair-cascade`). Where the rebuild is refused, the checkpoint has judged the working copy corrupt and cannot replace it, and there is no last state known good left to revert to.

The label path then stops, which is what the two checkpoints do when the disposition they exist to take is unavailable (`dec:surface:async-label`). No further label is applied; labels already queued are drained and refused rather than applied to the state the checkpoint has just disowned; the published snapshot stays as the read surface, so assessments go on answering; and the cause is recorded once — the model whose reconstruction was refused, the pivot, and how far into the label stream it happened — reaching the host at its next submission and on both tiers of the health surface. The command path is untouched, so a checkpoint or a shutdown requested after the stop still completes. The rebuild itself is at the deployment's declared prior rather than at the library default, so a revert does not retune the deployment on its way past; there is no second numerical parameter for it to take, because the factorisation it performs carries none (`dec:posterior:one-inversion-utility`).

One of CP5's two readings is taken inside the update rather than after it, and for a reason particular to what that reading is. The leverage is a quadratic form in the covariance, so it is non-negative for every feature vector exactly while the covariance is positive semi-definite, and a negative one is the report that the matrix is no longer a covariance. The update cannot consume that reading and disclose it afterwards, because the weighting divides by the leverage: a negative denominator turns the effective weight negative, a negative weight subtracts information the model never received, and the subtraction carries the precision matrix further from the property whose loss was being reported. So the update stops at the reading, mutating nothing, and the checkpoint then disposes of the label the way it disposes of any other corrupt finding. The other five readings are of state already written, and are taken where the table places them.

**Definition (Time-corrected uncertainty)** · `def:runtime:time-correction`

Model parameters decay in elapsed time as well as in labels, but the decay is applied when the working copy is available for mutation, which is at label time. Between labels the published snapshot's covariance stands still while the clock does not, so an uncertainty read from it would be too narrow by exactly the decay that has not yet been applied.

The correction is a scalar on the quadratic form, applied at read time to each model before blending:

$$\sigma^2(\phi) = \frac{1}{\gamma_t^{\Delta t_\text{snap}}} \cdot \hat{\phi}^\top \Sigma_\text{snapshot} \hat{\phi}$$

where the elapsed interval is measured from the snapshot's publication. The point estimate is untouched, because time-indexed decay scales the precision and the covariance reciprocally and leaves the mean where it was (`alg:temporal:lazy-application`).

The correction is exact rather than a safety margin, and it is the reason an assessment taken from a snapshot published hours ago reports honest uncertainty rather than the uncertainty of the moment the snapshot was cut. It applies on every model's quadratic form.

**Definition (The current report index)** · `def:runtime:report-index`

The current report index is a sorted interval map over one Sentinel's coordinate space, holding the cells present in that Sentinel's most recent batch report. Each entry maps a coordinate range to the cell's report data: its scores at every ancestor level, its coordination features, and its batch statistics.

The index is rebuilt in full on each reception and published by the same atomic swap the report itself is (`alg:runtime:report-reception`). Rebuilding rather than patching is the simpler discipline and the correct one: a batch report is a complete statement of what a Sentinel currently sees, so an index that merged it with an older statement would be describing a partition that never existed.

Where several reported cells cover a coordinate at different depths, lookup takes the deepest, because the deepest reported cell is the finest statement the Sentinel has made about that coordinate and the coarser ones are its ancestors. Report staleness — how far a Sentinel's baselines have moved since it reported — is carried as a feature rather than corrected for here (`def:extraction:report-staleness`). The index is published behind a lock-free swap.

**Definition (The Ledger spatial index)** · `def:runtime:ledger-index`

The Ledger index is a layered depth-priority map over each Sentinel's domain. Entries may overlap: several can cover one coordinate at different depths, and that overlap is the structure rather than a defect of it.

The two routings are deliberately different. A read finds the *deepest* entry containing the coordinate and reads only that one, so extraction sees the finest outcome memory available and never averages it with a coarser statement. A write finds *all* entries containing the coordinate, deepest to root, and updates each independently, so a label at a leaf informs its ancestors as well (`alg:ledger:all-layers-update`). Read routing always succeeds because the root entry always exists (`def:ledger:root-semantics`).

The index is populated incrementally from batch reports: a reported cell without an entry gets one (`alg:ledger:entry-creation`). Entries then persist across reports, accumulating outcome history that no single report contains, and leave only by the disappearance threshold or by collection (`alg:ledger:entry-deletion`) and (`alg:ledger:garbage-collection`). Reads and writes use their distinct routings.

**Algorithm (Extraction routing)** · `alg:runtime:extraction-routing`

For each Sentinel and each request's coordinate in that Sentinel's space, three lookups run and their failures are handled differently.

Batch features come from the deepest reported cell covering the coordinate; if no reported cell covers it, the Sentinel's batch features are zero for this request, which is the honest encoding of "this Sentinel currently says nothing here" (`tab:extraction:chain-z-scores`). Coordination features are read from the same cell's coordination summary and are zero on the same condition (`tab:extraction:coordination`).

Ledger features come from the deepest Ledger entry containing the coordinate, decayed to the present as a pure function and read without mutation (`tab:extraction:ledger-features`). This lookup cannot fail. The asymmetry is the point: a Sentinel may fall silent over a region and the Core loses its current measurement there, but outcome memory is the Core's own and remains available at whatever depth it has accumulated.

All three lookups follow the stated routing, the Ledger one taking the Ledger's own read routing so that the depth-walk the memory record specifies is the walk the assessment path performs (`dec:memory:depth-walk`), and the read leaving the stored entry untouched. Assessments in split regions therefore read the deeper, more specific outcome history, and the feature distributions the models see shift once as online adaptation absorbs the new reading. Nothing compensates for the shift, because a shim holding shallower distributions steady would preserve a dilution, and the adaptation the models already perform is the mechanism that absorbs a one-off shift of exactly this kind.

**Algorithm (Reported cell-set maintenance)** · `alg:runtime:cell-set-maintenance`

On each reception the Ledger index is reconciled against the reporting Sentinel's new report in three steps. Every cell in the report that lacks an entry at its exact interval gets one, created neutral. Every entry below the root that was previously associated with a reported cell has its consecutive-absence counter incremented if its interval is absent from this report and reset to zero if present. Every entry whose counter reaches the threshold — three, by default — is deleted.

No merge is needed on deletion, because all-layers write routing has kept the ancestor entry current all along (`alg:ledger:all-layers-update`).

The Core classifies nothing. It does not decide whether a vanished cell was split, evicted, restored or subdivided; it observes appearance and persistent disappearance and acts on those two facts alone (`conv:terminology:abstraction-boundary`). The Sentinel-internal cause is not merely unavailable but irrelevant, and treating it as relevant is the error this discipline exists to prevent (`cav:limitation:abstraction`). The contour snapshot's structural metadata is relayed to the host as diagnostic context and drives no Ledger operation.

**Bound (The cost of routing)** · `bound:runtime:routing-cost`

| Operation | Cost | When |
| --- | --- | --- |
| Rebuilding the report index | $O(\lvert R\rvert \log \lvert R\rvert)$ over the report's cells | Per reception |
| Ledger cell-set maintenance | $O(\lvert R\rvert \log \lvert L\rvert + \lvert T\rvert + \lvert D\rvert)$ over report, Ledger, tracked entries and deletions | Per reception |
| Read routing, all Sentinels | $O(n_s(\log \lvert R\rvert + \log \lvert L\rvert))$ | Per assessment |
| Write routing, all Sentinels | $O(n_s \cdot d_\text{max})$ | Per label |

At eight Sentinels and a maximum depth of eight, a label costs at most sixty-four constant-time updates to the Ledger. Against the quadratic model updates on the same path this is not a cost worth optimising, and the table is carried to say so rather than to warn.

**Bound (Ledger memory)** · `bound:runtime:ledger-memory`

The Ledger's memory is bounded in practice by the number of distinct cells reported within the decay horizon, not by the number ever reported: the time-indexed decay carries an unvisited entry toward zero and collection removes it once it arrives (`def:ledger:time-decay`) and (`alg:ledger:garbage-collection`).

At a thousand cells per Sentinel per day with thirty percent overlap between days, steady state is around twenty thousand entries per Sentinel; at roughly two hundred bytes each and eight Sentinels, some thirty-two megabytes. This is small enough to be uninteresting beside the pending buffer, which is where the system's memory actually goes (`tab:resource:memory`).

**Schema (The risk assessment)** · `schema:output:assessment`

An assessment is a determination of position. It carries five things.

| Field | Carrying |
| --- | --- |
| Identifier | The handle a label is later reported against |
| Risk basis | The full risk estimate and its provenance (`schema:risk:basis`) |
| Outcome predictions | One prediction per active axis, with uncertainty (`tab:axis:inference-outputs`) |
| Per-Sentinel summaries | One alarm summary per reporting Sentinel (`def:runtime:alarm-summary`) |
| Health snapshot | The compact validity flags a per-request consumer needs (`schema:output:health-snapshot`) |

The basis is transparent by design: the blend weight shows how much of the estimate rests on the coarse anchor rather than the fine sister model, the two regime record counts show whether the calibration is mature enough to be believed (`req:platt:minimum-samples`), and the borrowed share shows how much of the precision behind the reported uncertainty the models' own floors were holding up along this request's features rather than the evidence (`dec:risk:evidence-only-uncertainty`). The uncertainty beside it has already been widened by that share, so the share is what a host reads to undo the widening or to apply a correction of its own. Probability-scale values serve the host; raw-scale values serve the derivation; both are always present, because deciding which a consumer needs is not the Core's business.

What is absent is as specified as what is present. No tags, no posture, no channel reference, no action probabilities, no fragility. Every one of those is a derivation output, computed from an assessment by a host that has declared what its actions cost (`prin:principle:measure-not-decide`).

**Schema (The embedded health snapshot)** · `schema:output:health-snapshot`

Every assessment carries a compact, fixed-size health snapshot of eleven specified fields, so that a per-request consumer can tell a trustworthy assessment from a provisional one without fetching anything.

| Field | Carrying |
| --- | --- |
| Snapshot version | The join key into the full health report stream |
| Snapshot age | The elapsed interval the uncertainty correction was computed at |
| Warm-up stage | Where the deployment stands against the warm-up sequence |
| Sister calibrated | Whether the sister regime has enough records to be believed |
| Anchor regime frozen | Whether the anchor's calibration has been fixed |
| Drift flag | Whether any model's drift accumulator is near its threshold |
| Coverage fraction | Reporting Sentinels over registered ones, for this assessment |
| Uncertainty inflation | The most recent empirical inflation factor |
| Zero Sentinels | Whether the assessment was computed with none reporting |
| Standardisation phase | `WaitingForInit`, `Transitioning` or `InService`, for the coordinate snapshot this assessment acquired |
| Standardisation observations | Accepted cold-ramp observations represented by that snapshot, from zero through $N_\text{init}$ |

The version is the field the design turns on. Everything per-Sentinel or per-dimension stays in the full report and is fetched by version when forensics need it (`chap:spec:output-structures`), so an assessment log stays small and is still fully auditable after the fact — the snapshot says which report to go and read. It is also the coordinate version, because every advance of the ramp publishes a model snapshot (`alg:standardisation:batch-initialisation`): the version says which coordinate system a result was computed in, and the phase and the count say how far through its acquisition that system was. A second version namespace would be a second authority over one fact. Ramp maturity is the count divided by $N_\text{init}$ and is not carried beside it, for the same reason: a stored fraction and a stored count are two descriptions of one position, and two descriptions of one thing maintained apart eventually disagree.

The last two rows describe the snapshot the assessment used and not the ramp's position at the moment of reading, which is the whole of their forensic value. A consumer comparing two results is comparing two coordinate systems whenever the versions differ, and these fields say which.

Two of the fields are computed elsewhere on the same path and are merely carried here, the age by the uncertainty correction (`def:runtime:time-correction`) and the inflation factor by the coverage diagnostic (`alg:monitoring:empirical-coverage`); the stage and the drift flag likewise restate what their own environments own (`tab:warmup:stages`) and (`alg:monitoring:drift-cusums`). The snapshot carries all eleven fields, including the version, phase, accepted count and anchor-regime frozen flag, so the forensic join and the ramp position this environment establishes can both be read directly.

**Definition (The per-Sentinel alarm summary)** · `def:runtime:alarm-summary`

The alarm summary is a deterministic compression of one Sentinel's extraction into eight diagnostic fields: the peak z-score, the peak accumulator, the peak coordination z-score, a composite, the chain depth, the maturity, whether the Sentinel's coordinates are hierarchical, and whether the covering Ledger entry is immature.

It does not enter any model. It exists so a host can see which Sentinel is alarming and how confidently, which is a question the risk probability cannot answer because it has already aggregated the answer away.

The composite is the maximum of the cell-level peak z-score and the cell-level peak accumulator after normalisation, giving one scalar per Sentinel. The hierarchical flag reflects the coordinate semantics declared at registration. The immaturity flag is a maturity predicate: it holds when the deepest Ledger entry covering the request has fewer eligible labels than the materiality threshold, or when its estimated eligible label rate implies a steady-state attenuation below the floor — the two conditions under which the entry's outcome features are present but not yet worth weighting (`thm:ledger:materiality`) and (`data:ledger:attenuation`). The predicate is taken against the entry the extraction routing actually reached rather than against the mere existence of an entry (`alg:runtime:extraction-routing`).

Each input is read where it is stored. The eligible-label count and the arrival window that names the steady-state attenuation are fields of every entry (`tab:ledger:entry-state`); the count arm compares against (`def:config:ledger-materiality-threshold`) and the rate arm against (`def:config:attenuation-materiality-floor`). The health surface reads the same two fields over the whole Ledger, but it does not report this flag aggregated: it counts cells against the threshold alone (`def:monitoring:immature-cells`) and reports the attenuation condition separately, as a share of traffic rather than a count of cells (`def:monitoring:ledger-value`). The evidence is shared and the readings are three, so neither health figure is the fleet-wide total of this flag and none of the three may be substituted for another.

Two boundary readings are fixed rather than left to a caller. An entry whose arrival window has emptied names no attenuation, and the rate arm takes the limit the attenuation formula gives there — zero, which lies below every floor the configuration admits (`def:config:attenuation-materiality-floor`) — so a cell that has stopped receiving evidence cannot pass as mature on a count it stopped adding to. A Sentinel with no Ledger entry at all is immature for want of any evidence whatever; with the root in place that state does not arise, because the root contains every coordinate (`def:ledger:root-semantics`).

**Example (Composing the three components)** · `ex:runtime:composition`

The host is the only place the three components meet, and the pattern is short. It assesses a request; it reads the Companion's posterior; it derives a landscape from the two against a channel policy; it reads the landscape at its posture for an action and for that action's fragility; it decides and acts. Nothing in that sequence is a call the Core makes.

Multiple channels cost almost nothing. One assessment is derived against each channel policy in turn — the quadratic-cost work is paid once and each derivation is linear in the declared actions plus a handful of quantile evaluations.

Replay needs three stored values and no component state: the assessment, the channel policy, and the challenge posterior. Those reproduce the landscape exactly, without access to model parameters, pseudo-counts, Sentinel reports or the pending buffer, which is what makes an audit of a past decision possible at all (`inv:guarantee:replay`).

Feedback is two independent reports, not one. The Core is labelled with the outcome; the Companion is updated with the challenge result, and only when a challenge was performed on an adverse request (`alg:companion:update`). They touch different components and may go in either order. The Core never sees the challenge result and the Companion never sees the valence magnitude or the feature vector, so the boundary the specification draws holds at runtime and not only on paper (`inv:companion:boundary`). A host wanting neither landscapes nor challenges runs the first and last steps alone, and needs no Companion at all.

### Chapter (The Label Interface and Learning Pipeline) · `chap:spec:label-pipeline`

The chapter is where the system learns. A host reports an outcome against a stored assessment; the stored assessment is found, its feature vector rebuilt in the current coordinate system and restandardised against the current statistics, and seventeen ordered steps then move every quantity a label is evidence about. The buffer that holds assessments between the two halves is specified here too, because its contents are exactly what reconstruction has to work from, and because what it does not store is what reconstruction cannot rebuild.

**Requirement (The label interface)** · `req:runtime:label-interface`

A label is reported against a stored assessment and carries the five fields the host contract fixes (`tab:host:label-reporting`). The call returns an acknowledgement, or an error where the assessment is not in the buffer — expired, evicted, or already labelled.

Two properties bind. Label processing is serialised internally on the path that owns the models, because the updates are not commutative in their conditioning and a second label applied concurrently to the first would produce a posterior neither of them justifies. And label processing never blocks assessment: a label arriving under load is queued rather than executed inline, and the queue is bounded (`req:publication:label-queue`) and (`inv:guarantee:non-blocking`).

The sign convention governing valence is the document's, fixed once and read here (`conv:valence:sign`). The five fields form one payload, and the call returns a typed acknowledgement.

**Algorithm (The label update path)** · `alg:runtime:update-path`

When a host labels assessment $r$, seventeen steps run in this order.

1. Retrieve the pending entry (`def:runtime:pending-entry`).
2. Reconstruct the feature vector in the current coordinate system (`alg:runtime:reconstruction`).
3. Restandardise against the current statistics (`bound:standardisation:restandardisation`).
4. Compute the risk target from the sign of the valence (`def:risk:target`).
5. Update the valence compression scale, where the valence is non-zero (`def:risk:compression-scale`).
6. Determine eligibility from the action taken and the ground-truth flag (`tab:eligibility:training`).
7. Update the class-rate trackers: the global tracker always, the eligible tracker only where the label is eligible (`alg:weighting:tracker-update`).
8. Compute the importance weight from the appropriate tracker (`def:weighting:balancing-weights`).
9. Update the operational model (`alg:update:sherman-morrison`).
10. Where the label is eligible, update the sister and anchor models.
11. For each active axis with a reported value: compress it, update its compression scale, determine its eligibility, and update its model where eligible (`def:axis:adaptive-compression`) and (`def:axis:training-target`).
12. Update the identity dimension outcome state (`alg:runtime:identity-outcome-update`).
13. Update the Ledger, per Sentinel and per reported axis (`alg:ledger:all-layers-update`).
14. Update the calibration buffer, refitting where the cadence triggers (`constr:platt:buffer`) and (`tab:platt:refit-cadence`).
15. Update the standardisation statistics (`alg:standardisation:label-time-procedure`).
16. Update the prediction drift accumulators (`alg:monitoring:drift-cusums`).
17. Publish the working copy as the new snapshot (`def:publication:model-snapshot`).

No challenge effectiveness update appears, because the Companion is the host's to feed and the Core never sees a challenge result (`inv:companion:boundary`). Steps two and three are where the coordinate system is reconciled: the buffer holds raw features, and the statistics they are standardised against have moved since the assessment, by a bounded amount. Step nine runs unconditionally and step ten does not, which is the whole of the eligibility distinction in operational terms. Steps nine, ten and each axis update in step eleven are mutually independent — each reads the shared standardised vector and the target and writes only its own model — so they may run in parallel, reducing the wall-clock cost from a multiple of the quadratic update to roughly one of them.

**Algorithm (The identity outcome update)** · `alg:runtime:identity-outcome-update`

At step twelve, for each registered identity dimension: the stored entity key is re-encoded, and the competitive cells recorded at assessment time are walked. For each such cell still competitive, lazy decay is applied and then its adverse rate, its compressed and raw valence averages, and its per-axis outcome averages are updated (`tab:keyspace:decay-rates`). A cell no longer competitive is skipped: its outcome state no longer exists, its information having been transferred into the surviving dimensions when it left (`alg:keyspace:cell-exit`).

The update reaches every competitive cell containing the coordinate and not only the deepest. An outcome from one host address updates the address's cell, its subnet's cell and its block's cell alike, wherever all three are competitive, because each is a distinct claim about a distinct population.

The asymmetry with reconstruction is deliberate and worth stating. Outcome averages are written to the *stored* active set, filtered to cells still competitive; reconstruction populates indicator features from the *current* set (`alg:runtime:reconstruction`). A cell promoted in the interval therefore gets a reconstructed indicator and no outcome update this cycle, and a demoted one gets neither. The outcome average belongs to the entity's history and must not be written into a cell that does not hold that history; the indicator describes the model's present parameterisation and must match it. The divergence is small, because competitive restructuring is rare, and it self-corrects on the next label (`cav:limitation:competitive-churn`).

**Bound (Per-label cost)** · `bound:runtime:label-cost`

The per-label cost is tabulated once, with the other resource bounds, and is cited here rather than duplicated (`tab:resource:label-cost`). At the reference configuration it is roughly $1.22$ million operations, on the order of a millisecond (`tab:resource:reference-configuration`).

The shape of that figure is what matters operationally: it is dominated by the quadratic model updates of steps nine through eleven, so it grows with the square of the feature dimension and linearly in the number of active axes, and is essentially independent of the number of Sentinels, the Ledger's size and the identity layer's depth. A deployment whose label cost is a problem has too many dimensions, not too much traffic.

**Definition (The pending entry)** · `def:runtime:pending-entry`

An assessment is recorded in the pending buffer so that a label arriving later can be matched to what was actually computed. The entry carries twelve things.

| Field | Carrying |
| --- | --- |
| Identifier and timestamp | The handle the label is reported against, and when |
| Entity key | The key, for re-encoding on each identity dimension |
| Per-Sentinel extractions | One frozen extraction per reporting Sentinel |
| Active Sentinels | Which Sentinels were registered at assessment time |
| Reporting Sentinels | Which of those were actually reporting |
| Identity coordinates | The encoded coordinate on each dimension |
| Identity active cells | The competitive cells the coordinate fell in |
| Entity base features | The frozen per-dimension identity features |
| Entity axis features | The frozen per-dimension, per-axis identity features |
| Signal features | The frozen encoded signals |
| Risk basis | The estimate, retained for drift and calibration |

The buffer stores the per-Sentinel extractions separately rather than the assembled vector, and that is the design decision the rest of the chapter rests on. An assembled vector is indexed against the dimension map that existed when it was assembled; a lifecycle event between assessment and label invalidates those indices, and the stored vector would then be a set of numbers whose meaning had moved. Stored components can be replaced into a current map instead (`alg:runtime:reconstruction`).

No channel field and no stored landscape: the entry describes what the Core computed, not what the host decided from it. The basis is retained for the drift accumulators and the calibration buffer, which read the prediction rather than the features (`schema:risk:basis`).

**Definition (Storage precision)** · `def:runtime:storage-precision`

Pending entries store feature values at single precision by default, configurably, and upcast to double at label time.

The reasoning is a comparison of two errors rather than an appeal to frugality. Single-precision quantisation costs about one part in ten million per feature. Restandardisation — the same feature expressed against statistics that have moved between assessment and label — costs about one part in a hundred (`bound:standardisation:restandardisation`). The second is five orders of magnitude larger, so storing at double precision would halve the buffer's capacity to remove a hundred-thousandth of the error already present, which is not a trade any deployment should want.

The same reasoning is applied at the same width where extraction is stored, for the same reason (`rem:extraction:single-precision`).

**Requirement (Buffer capacity and eviction)** · `req:runtime:buffer-capacity`

Capacity is computed from the deployment's own numbers rather than guessed: twice the expected peak request rate times the expected median labelling latency, so that a burst at peak rate is still fully labelled at median delay.

| Request rate | Labelling latency | Required capacity | Memory at single precision |
| --- | --- | --- | --- |
| $100$ per second | One hour | $720{,}000$ | About $2.2$ GB |
| $10$ per second | Fifteen minutes | $18{,}000$ | About $54$ MB |
| $1$ per second | One hour | $7{,}200$ | About $22$ MB |

The table's first row is the reference configuration: the peak request rate (`def:config:reference-peak-request-rate-100-per-second`) paired with the median labelling latency (`def:config:reference-label-latency-3600-seconds`).

The buffer is the system's dominant memory consumer at any serious request rate, which is why the sizing rule is stated as a computation: a fixed default is either wasteful at low rates or silently lossy at high ones. Beyond capacity the oldest entries are evicted, and a label arriving for an evicted entry is lost — the call errors, and the outcome it carried never reaches any model (`cav:limitation:buffer-eviction`). Eviction is therefore not a graceful degradation but a measured loss of evidence, and the eviction rate belongs in the health report where a host will see it (`chap:spec:output-structures`). The parameters are tabulated with the rest of the configuration surface (`tab:config:pending-buffer`). Capacity is $2 \times R \times L$ from the reference peak request rate (`def:config:reference-peak-request-rate-100-per-second`) and the reference median labelling latency (`def:config:reference-label-latency-3600-seconds`), and health reports the lifetime share of pending insertions lost to live eviction alongside the age of the oldest live pending entry. The interval count also travels inline on the assessment whose insertion observed it (`entry:retention:eviction-counter`), (`entry:retention:eviction-visibility`).

**Algorithm (Reconstruction at label time)** · `alg:runtime:reconstruction`

At label time the feature vector is rebuilt from the stored components against the *current* published dimension map and the current model parameters, which are one self-consistent version (`inv:dimension:version-consistency`). Rebuilding against the current map is what makes a lifecycle event between assessment and label harmless: features of a deregistered entity are in the buffer and are simply not placed, because the map no longer holds indices for them, and their information has already been transferred into the surviving dimensions (`def:gaussian:schur-complement`).

Six numbered sources fill the vector, and every index has exactly one of them.

1. **Fixed blocks.** The bias, the stored signal features, and the stored identity base features.
2. **Identity dimension features.** For each currently registered dimension: re-encode the stored coordinate, look up the *current* competitive set, populate the indicators from it, and recompute the aggregates (`def:keyspace:competitive-indicators`).
3. **Axis features.** For each currently registered axis: the stored features where the axis existed at assessment time, zeros where it was registered afterwards.
4. **Sentinel slots.** For each currently registered Sentinel: the stored extraction where it was reporting, occupancy with zero features where it was active but silent, and no slot at all where it is no longer registered (`def:extraction:slot`).
5. **Aggregate features.** Recomputed from the stored extractions over the currently reporting Sentinels (`def:feature:template-aggregate`).
6. **Interactions.** Recomputed by applying the current templates to the reconstructed features (`alg:dimension:compilation-pipeline`).

The covering invariant is what guarantees the enumeration is complete: every index in the reconstructed vector is claimed by exactly one source, so there is no position whose value is a matter of which step ran last (`inv:dimension:covering`). With the working state unchanged between assessment and label the reconstruction reproduces the assessment-time raw vector exactly, block for block, which is the equality that settles it.

### Chapter (Concurrency) · `chap:spec:concurrency`

The chapter states what may wait on what. One principle governs it, and the rest of the chapter is that principle applied component by component: what is published and how, what drains off the hot path, what staleness each component may carry, and what budget each shared structure is held to. Three of the document's formal properties are stated here as the whole content of an environment and therefore mint here rather than in the register that collects them.

**Invariant (Non-blocking assessment)** · `inv:guarantee:non-blocking`

An assessment does not wait on any other operation beyond a bounded, per-component budget. No label, no lifecycle event, no report reception, no identity maintenance and no concurrent assessment may delay an assessment's return by more than the budget declared for the component it reads. Most paths declare a structural budget, meaning lock-free and therefore none; a small number declare a bounded budget with an explicit upper bound and a tail probability (`tab:publication:tiers`).

An assessment may read state that is not the most recent. One computed during a Sentinel registration may reflect the pre-registration model; one computed while a label is processing may not include that label. This is accepted, and the acceptance is the invariant's whole content: **staleness is acceptable and delay is not**. Stale state is bounded and self-correcting (`inv:guarantee:staleness`); an unbounded wait is neither, and a request-path component that can block is a component that will, under exactly the load where blocking is least affordable.

Every other operation in the system must be organised so the budget holds. The specification does not prescribe how — double buffering, sharding, deferral and atomic swaps are all admissible, and the chapter names the ones the design uses rather than requiring them. It requires the property: published state is loaded lock-free on the assessment path.

**Definition (The published model snapshot)** · `def:publication:model-snapshot`

Assessments read a published snapshot acquired once, at the start of the call. The snapshot carries a version and a publication timestamp, the three risk models, the per-axis models with their own scales and eligibility modes, the dimension map, the standardisation means and variances, the two class-rate trackers, the three calibration parameters, the calibration buffer, and the drift state per model.

The snapshot is self-consistent by construction. Model parameters, dimension map, standardisation statistics, competitive sets, calibration state and every derived quantity correspond to one version, and no assessment ever observes a mixture of two (`inv:dimension:version-consistency`). That is what makes the dimension map and the parameters indexed against it safe to read without a lock: they cannot disagree, because they are published together or not at all.

Publication is componentised. Each parameter block, the map, the standardisation vectors and the calibration state sit behind independent shared pointers, and structurally unchanged components are shared across versions rather than copied, so an ordinary label never copies the lifecycle-stable state. Because the rank-one update touches every element of a model's precision and covariance (`alg:update:sherman-morrison`), the parameter components are double-buffered: the label path mutates the working buffer, publication is a pointer swap, and resynchronising the now-stale buffer is charged to label processing rather than to any reader.

No challenge effectiveness state appears, because that is the Companion's (`def:companion:state`). Sentinel reports are published independently and may be read at a different version than the model, which is correct: a report is a measurement and not a parameter.

**Requirement (The publication interval)** · `req:publication:interval`

The publication interval is an amortisation knob, defaulting to one label. At a value of $k$ the working copy is published every $k$ labels rather than every one, amortising the resynchronisation copy across them at the cost of widening every staleness bound by up to $k$ labels (`tab:publication:staleness-bounds`).

The trade is explicit and belongs to the host. A deployment whose label rate is high enough that the per-label resynchronisation is material can buy it back with staleness it can quantify; a deployment that would rather see every label immediately keeps the default and pays. Nothing else changes with the setting: the read-time uncertainty correction goes on correcting for snapshot age whatever the interval is, so a host trading freshness for throughput still reads honest uncertainty rather than confident staleness (`def:runtime:time-correction`).

Pre-seeding is the clearest case for a value above one, publishing once at the end of a historical batch instead of once per historical label (`alg:host:pre-seeding`). The Core configuration therefore carries this interval, and the label path publishes when the interval elapses; a field of the same name on the blend-statistics configuration governs a different quantity entirely.

**Algorithm (Deferred identity observation draining)** · `alg:publication:identity-draining`

Identity dimension graphs take their observation volume from assessment traffic, and performing those observations inline would put graph maintenance — splitting, eviction, competitive restructuring — on the request path. They are deferred instead: the assessment records each observation to a per-dimension queue and returns (`alg:keyspace:observation-protocol`).

A background task drains the queues on its own cadence, batched, independent of both assessment and label processing. For each queued observation it performs the graph observation, maintains importance, and triggers whatever structural operations follow (`alg:keyspace:cell-entry`) and (`alg:keyspace:cell-exit`). Those operations publish through the same lifecycle protocol every other structural change uses (`inv:guarantee:lifecycle-publication`), and assessments read the competitive set from the most recently published state (`def:keyspace:competitive-set`).

The enqueue is the only cost the assessment path bears, and the queue is bounded and drops oldest under sustained pressure. Dropping is safe here in a way it would not be elsewhere: unit observations are exchangeable, so a dropped one slows the competitive set's adaptation and corrupts nothing.

**Invariant (Ledger concurrency)** · `inv:publication:ledger-concurrency`

Assessments read Ledger entries and never modify them. The time-indexed decay is computed at read time as a pure function of elapsed time, so a read produces the decayed value without storing it (`def:ledger:time-decay`). Stored entries move only during label processing, under the bounded per-Sentinel write budget (`alg:ledger:all-layers-update`).

The property is what makes outcome memory affordable on the request path at all. A decay-and-store read would make every assessment a writer of every entry it touched, which would put a write lock on the one structure an assessment consults most and turn read contention into write contention. Purity removes the question on both paths.

**Table (Staleness bounds)** · `tab:publication:staleness-bounds`

| Component | Staleness bound | Evolution timescale |
| --- | --- | --- |
| Model parameters | Labels processed since the last publication | Per label, per weight |
| Dimension map | Lifecycle events since the last publication | Hours to never |
| Standardisation statistics | Accepted cold observations since the last ramp publication, while transitioning; labels processed since the last publication, in service | Per accepted observation while transitioning; per label, per feature, in service |
| Identity competitive set | Restructuring events since the last publication | Hours to days |
| Calibration parameters | Refits since the last publication | Every refit interval |
| Sentinel batch report | Reports received since the last read | Seconds to minutes |
| Ledger averages | None; the read-time decay is exact | — |

Each bound is scaled by the publication interval where one is set above the default (`req:publication:interval`). The table's shape is its argument: the components whose bounds are counted in labels move slowly per label, and the one component with no bound at all is the one read through a pure function rather than from published state. Standardisation is the one row counted two ways, because it is fed by two authorities and the clock changes when the cold ramp reaches its horizon (`alg:standardisation:batch-initialisation`); a reader holding the phase knows which of its two bounds applies. Every row is a published-and-swapped component, including the transitioning bound: each accepted cold observation publishes the ramp's next coordinate snapshot.

**Invariant (Bounded staleness)** · `inv:guarantee:staleness`

Every read is stale by at most its component's declared bound, and the bounds are the table above. Nothing is unboundedly stale, and nothing is stale in a way the reader cannot see.

Two mechanisms make the property useful rather than merely true. Staleness is self-correcting: every bound is measured in events that are themselves arriving, so a component's staleness returns to zero on its next publication without any reconciliation step. And where staleness has a quantitative consequence, that consequence is computed and reported rather than absorbed — an uncertainty read from an old snapshot is inflated by exactly the decay that has not yet been applied (`def:runtime:time-correction`).

The invariant is what a host is owed in exchange for the non-blocking guarantee. An assessment never waits, and in return it may read state a few labels old; without the bound that would be an unlimited licence, and with it the trade is one a host can reason about. The read-time correction applies.

**Invariant (Lifecycle publication)** · `inv:guarantee:lifecycle-publication`

A lifecycle event — the registration or deregistration of a Sentinel, an outcome axis or an identity dimension — modifies model structure. No such modification is ever visible to an assessment already in flight. An assessment beginning before the event completes reads pre-event state throughout; one beginning after reads post-event state throughout. No assessment is drained, paused or delayed by a lifecycle event.

The two halves are separate promises and both are load-bearing. Atomicity is what makes the structural guarantee of the lifecycle operations meaningful at runtime: even a correctly partitioned operation is worth nothing if a reader can observe it half-applied and compute against a map that never existed (`inv:dimension:version-consistency`). Non-interruption is what makes lifecycle events usable in a live system rather than operations requiring a quiet period — a host may register a Sentinel at peak traffic and no request pays for it.

Structural changes therefore publish exactly as parameter changes do, through the same swap, and the standardisation entries a lifecycle event creates move under the same publication (`req:standardisation:lifecycle-entries`).

**Requirement (The pending buffer under concurrency)** · `req:publication:pending-buffer`

The pending buffer is written by assessment and read by label processing (`def:runtime:pending-entry`). Neither path may delay the other, and in particular the recording of an entry must not delay the assessment that records it.

The requirement is easy to state and easy to violate, because a buffer with a capacity bound invites a lock around the eviction decision. The buffer is therefore a concurrent map, insertion is lock-free or sharded, and the capacity discipline is enforced without serialising the writers (`req:runtime:buffer-capacity`).

**Requirement (The label queue)** · `req:publication:label-queue`

Labels arriving faster than the model-owner path can absorb are held in a bounded queue off the assessment path. Accepted labels remain first-in-first-out. On overflow the owning concurrency decision refuses the newest submission immediately rather than silently dropping an accepted one (`dec:concurrency:no-silent-drop`).

A refused label is lost evidence unless the host retries it, so every full-queue refusal is counted. The typed error remains the per-call acknowledgement; queue depth and the cumulative loss counter are the public health signal that label volume has outrun processing capacity (`chap:spec:output-structures`). The capacity is tabulated with the other concurrency parameters (`tab:config:concurrency`).

**Requirement (Report reception)** · `req:publication:report-reception`

Reception publishes a Sentinel's new report by an atomic swap, and cell-set maintenance against the Ledger runs inside the same call, before the swap (`alg:runtime:report-reception`) and (`alg:runtime:cell-set-maintenance`).

The ordering is the requirement. Maintenance creates Ledger entries for newly reported cells and deletes entries for persistently absent ones; if the report were published first, assessments in the interval would route against a report index describing cells the Ledger index does not yet hold, and would read the ancestor entry where a fresh entry was about to exist. Doing the maintenance first makes the swap the single moment at which both indices become current together. The maintenance falls under the Ledger's bounded write budget; the publication itself is structural.

**Table (The concurrency tiers)** · `tab:publication:tiers`

| Component | Tier | Budget |
| --- | --- | --- |
| Model snapshot | Structural | Lock-free load of a swapped pointer |
| Dimension map, within the snapshot | Structural | Published with the snapshot |
| Per-Sentinel report index | Structural | Lock-free load of a swapped pointer |
| Pending buffer insertion | Structural | Concurrent map, lock-free or sharded |
| Concordance window and bootstrap accumulators | Structural | Sharded lock-free accumulators |
| Deferred identity-observation enqueue | Structural | Bounded queue, drop-oldest |
| Identity graph observation | Structural | Deferred entirely off this path |
| Per-dimension measurement state | Bounded | $B = 20\,\mu\text{s}$ at $p \le 10^{-4}$ |
| Per-Sentinel Ledger | Bounded | $B = 50\,\mu\text{s}$ at $p \le 10^{-4}$ |

And four properties over the whole call.

| Property | Force |
| --- | --- |
| Assessment is lock-free on every structural component | Unconditional |
| Assessment's total wait on bounded components is within the budget | Unconditional bound; the tail is governed by $p$ |
| Assessment reads self-consistent state at each version | Unconditional |
| Assessment may read stale state | Bounded and self-correcting |

Two bounded rows out of nine is the whole of the system's blocking, and both are per-component locks over small updates rather than anything held across a model computation. Every structural row is a lock-free swap, and the two bounded rows are the two locks named.

### Chapter (Temporal Governance) · `chap:spec:temporal-governance`

Every learned quantity in the system forgets, and the chapter is the one place that says how fast. Two mechanisms, one inventory of every decaying quantity against the mechanism and rate that governs it, the discipline by which decay is applied, and the one rate whose consequence is structural rather than a matter of tuning. The inventory cites nothing and is cited by every chapter that owns a decaying quantity, which is the correct direction for a reference of this kind.

**Definition (Two independent decay mechanisms)** · `def:temporal:two-mechanisms`

Label-indexed decay is applied at each label and discounts the precision carried in existing observations. It is the primary forgetting mechanism under normal operation, and it measures age in evidence: a model that has seen a thousand labels since an observation has discounted it a thousand times, however long that took.

Time-indexed decay is applied at each access, on elapsed wall-clock time, and provides a forgetting floor. It measures age in hours, and it exists because label-indexed decay alone has a failure mode: a quantity receiving no labels never forgets anything, so a system in a quiet period, or a Ledger cell the host has stopped routing traffic to, would hold its state indefinitely and go on acting on evidence of unbounded age.

The two are multiplicatively independent:

$$\gamma_\text{eff}(n, \Delta t) = \gamma^n \cdot \gamma_t^{\Delta t}$$

Independence is what lets them be reasoned about separately. A rate can be set for evidence volume without regard to elapsed time and a floor set for elapsed time without regard to volume, and the composition needs no correction term because the two exponents are over different quantities. The label-indexed rates belong to model configuration and the time-indexed rates to their own configuration.

**Table (The decay inventory)** · `tab:temporal:decay-inventory`

| What decays | Mechanism | Rate | Purpose |
| --- | --- | --- | --- |
| Sentinel spatial importance | The Sentinel's own spatial layer | Host-chosen | Contour evolution |
| Sentinel subspace | The Sentinel's tracker | The Sentinel's own rate | Forget old correlations |
| Sentinel baselines | The Sentinel's tracker | The Sentinel's own rates | Score distribution adaptation |
| Operational model | Label-indexed | $0.9995$ | Forget old risk patterns, fast |
| Sister model | Label-indexed | $0.9998$ | Forget old inherent-risk patterns |
| Anchor model | Label-indexed | $0.9998$ | Forget old coarse-risk patterns |
| Outcome axis models | Label-indexed | $0.9998$ by default, per axis | Forget old axis patterns |
| All model parameters | Time-indexed | $0.9999$ per hour | The forgetting floor for models |
| Global class-rate tracker | Label-indexed only | $0.9995$ | Class balance for the operational model |
| Eligible class-rate tracker | Label-indexed only | $0.9998$ | Class balance for sister, anchor and axes |
| Standardisation statistics | Observation-indexed finite ramp before service, then label-indexed | $1/N_\text{init}$ of base mass per accepted observation, then $0.9998$ | Coordinate system acquisition, then continuing evolution |
| Ledger averages | Label-indexed | $0.999$ | Spatial outcome memory decay |
| Ledger averages | Time-indexed, fast | $0.999$ per hour | The contamination loop breaker |
| Identity cell outcome averages | Per label | $0.95$ | Per-range outcome history adaptation |
| Identity cell outcome averages | Time-indexed | $0.998$ per hour | Stale cell state cleanup |
| Identity spatial importance | Per observation | Host-chosen | Graph contour evolution |
| Signal cache entries | Eviction, not decay | Capacity-bounded | Entity-persistent signal memory |
| Companion pseudo-counts | Time-indexed | $0.9998$ per hour | Challenge effectiveness non-stationarity |

Three time-indexed rates are deliberately separated and the separation is the inventory's substantive claim. Models forget over about two hundred and ninety days, because learned weights should survive a quiet quarter. The Ledger forgets over about twenty-nine days, because it is the layer the contamination loop runs through and its decay is that loop's hard bound (`alg:valence:contamination-loop`). Identity cell outcome state forgets faster still, over about fourteen days, because a range's population turns over faster than a spatial region's does. The class-rate trackers carry no time-indexed rate at all, which is a recorded decision rather than an omission (`dec:weighting:no-time-decay`). The Companion's rate is independent of all of them (`def:companion:challenge-decay`). Standardisation is the one row carrying two mechanisms rather than one, and the first of them is not a decay at all: a finite ramp that retires a fixed share of the priors per accepted observation and then stops, handing the coordinate system to the label-indexed rate beside it (`alg:standardisation:batch-initialisation`). It is inventoried here because this table is where a reader comes to ask what moves a quantity and on whose clock, and the answer for standardisation is two clocks in sequence.

Every Core rate in this table governs its named component, and the ramp retires the specified equal share on every accepted cold observation. The identity per-label rate is pipeline-internal rather than host-configurable, so a host cannot set what the row presents as settable, and the parameters that are settable are tabulated with the configuration surface (`tab:config:temporal`).

**Algorithm (Lazy application)** · `alg:temporal:lazy-application`

Each decaying component carries the time it was last touched, and decay is applied on access rather than on a schedule. Nothing sweeps, nothing wakes, and a component never visited is never charged for.

For a Bayesian model the precision is scaled down and the covariance reciprocally up, which discounts the confidence and leaves the mean where it was:

$$B \leftarrow \gamma_t^{\Delta t} \cdot B, \qquad \Sigma \leftarrow \gamma_t^{-\Delta t} \cdot \Sigma$$

For a Ledger average or an identity cell average the value is scaled toward zero by the elapsed factor. The two forms differ because the quantities differ: a posterior forgets by widening, an average forgets by returning to its neutral value.

Laziness has one consequence that must be repaired rather than accepted. Model decay fires at label time, when the working copy is available for mutation, so between labels the published snapshot's covariance stands still while the clock runs. An uncertainty read from it would be too narrow by exactly the unapplied decay, and the assessment path therefore applies a read-time scalar correction — exact for the covariance, and no effect on the point estimate (`def:runtime:time-correction`). Ledger reads need no such repair because they decay purely at read time already (`def:ledger:time-decay`).

**Table (Per-component rates)** · `tab:temporal:component-rates`

| Component | Label-indexed | Time-indexed, per hour | Time half-life |
| --- | --- | --- | --- |
| Operational model | $0.9995$ | $0.9999$ | About $290$ days |
| Sister model | $0.9998$ | $0.9999$ | About $290$ days |
| Anchor model | $0.9998$ | $0.9999$ | About $290$ days |
| Outcome axis models | $0.9998$ by default | $0.9999$ | About $290$ days |
| Outcome Ledger | $0.999$ | $0.999$ | About $29$ days |
| Identity cell outcome | $0.95$ | $0.998$ | About $14$ days |
| Companion tracker | — | $0.9998$ | About $145$ days |

The table restates the inventory per component rather than per quantity, which is the view a reader tuning one component wants; the inventory is the view a reader auditing the system's forgetting as a whole wants. The two statements appear together so that a divergence between them is visible.

**Theorem (The Ledger's time-decay bound)** · `thm:temporal:ledger-decay-bound`

The Ledger's fast time-indexed rate is the primary mitigation of the contamination loop, and the mitigation is a bound rather than a tendency.

Under normal label flow the label-indexed decay dominates. At a rate of $0.999$ per label the effective half-life is about six hundred and ninety-three labels; at ten eligible labels a day that is about sixty-nine days, and the twenty-nine-day time-indexed half-life binds first.

Under complete starvation — a cell the host's own restriction has stopped sending eligible labels to, which is exactly the loop's terminal state — the label-indexed decay has no effect at all, because no labels arrive to apply it. Time-indexed decay is then the only thing acting, and it acts unconditionally: a cell's remembered adverse rate falls to half in twenty-nine days, a quarter in fifty-eight, an eighth in eighty-seven, whatever the host does or fails to do.

That is the whole of the guarantee and it is worth being precise about its limits. The bound does not prevent a cell from acquiring a reputation the evidence does not support; it bounds how long such a reputation can persist unexamined, which converts an unbounded feedback loop into a decaying one (`tab:ledger:loop-timescales`). Buying the evidence that would settle the question is a separate mechanism and the only one that acts rather than waits (`def:ledger:starvation-score`). The floor the bound establishes is stated among the guarantees (`inv:guarantee:ledger-floor`), and it rests on the default rate.

## Part (Properties and Analysis) · `part:spec:properties-and-analysis`

Parts I through VI specified what the system is and how it runs. Part VII asks what follows from it. How long a deployment waits before its output means anything, what the running system costs, what the composed design detects that no single layer of it detects alone, and how an operator sees any of it going wrong — these are the four chapters, and their common subject is consequence rather than construction.

Most of what follows computes from parameters fixed in earlier Parts rather than fixing anything of its own, and the Part is written to keep that distinction visible: an environment that binds says so, and an environment that merely restates a threshold cites the chapter that fixed it.

### Chapter (System Initialisation and Warm-Up) · `chap:spec:initialisation-and-warmup`

A deployment does not begin useful. The chapter says how it becomes so: the stages it passes through, the two quantities that must bootstrap themselves before anything downstream is trustworthy, what convergence looks like at three traffic profiles, and how the layers recover after the ground moves under them.

Two of these bind and the rest illustrate. The bootstraps are procedures the implementation must perform. The stages, the milestones, the profiles and the recovery timelines compute consequences of thresholds fixed in the calibration, Companion and feature chapters, and a restatement that cites its sources binds nothing of its own.

**Table (The six warm-up stages)** · `tab:warmup:stages`

| Stage | Extent | What has converged | What the host can expect |
| --- | --- | --- | --- |
| Cold start | First ~30 labels | Nothing; all means at $\mathbf{0}$ | $\hat{p} \approx 0.5$ everywhere, maximal uncertainty, no discrimination |
| Anchor emergence | ~30–80 labels | The anchor's fifteen parameters | First directional estimates; uncertainty still wide |
| Pre-calibration | ~80–200 labels | Direction, not magnitude | Scores ordered correctly, probabilities unreliable |
| Sister convergence | ~200–$2p$ labels | The sister's per-Sentinel features | Sentinel-specific discrimination; calibration fitted at least once |
| Interaction maturity | ~$2p$–$10p$ labels | Interaction features, in cascade order | Joint and cross-Sentinel structure, arriving unevenly |
| Steady state | Beyond | All parameters; a dynamic equilibrium | Calibrated assessments with characterised uncertainty |

The stages are approximate descriptions of a continuous process, not transitions the implementation performs, and they are given as one table rather than six identities for that reason. The anchor converges first because it is smallest (`def:risk:anchor-model`); the blend weight is high while it dominates and falls towards a small residual as the sister converges (`def:risk:subspace-blend`). Sister convergence is measured in eligible labels, so its wall-clock extent depends on the eligibility rate (`tab:eligibility:training`), and the interaction features arrive in the order the convergence cascade fixes (`tab:feature:interaction-convergence`). The importance trackers are unstable through the first two stages for the ordinary reason that they are estimated from few samples (`tab:weighting:trackers`). Forgetting never stops, so the last stage is an equilibrium rather than an end state.

The identity layer converges in parallel, on its own timescales:

| What converges | Timescale |
| --- | --- |
| Identity graph spatial structure | Minutes to hours |
| Competitive set stabilisation | Hours to days |
| Per-cell indicator weight convergence | Days to weeks |
| Per-range by alarm conjunctions | Weeks |

The Companion converges in parallel too, and on a timeline determined entirely by the rate at which contributing challenge labels arrive (`bound:companion:convergence`).

**Algorithm (The concordance threshold bootstrap)** · `alg:warmup:concordance-bootstrap`

The aggregate feature concordance thresholds require a rolling window of sub-scores before they can be calibrated at all (`alg:feature:concordance-recalibration`), and the window has to be filled before it can be read. Three phases:

Before the first calibration — the first thousand assessments — the initial value $\theta_a = 0.5$ is used unchanged.

At the thousandth assessment the accumulated sub-scores are read and their eightieth percentile replaces the initial value.

Every thousand assessments thereafter the thresholds are recalibrated from a rolling window of the ten thousand most recent sub-scores, one entry per reporting Sentinel.

The thresholds are held as separate atomic state outside the model snapshot. Changing them therefore triggers neither a calibration refit nor a drift accumulator reset, which is what keeps a threshold move from being read downstream as a change in the model's behaviour (`sec:feature:aggregate`).

The initial value is one half, the cadence is one thousand assessments, the percentile is the eightieth, and the rolling population is the ten thousand most recent sub-scores with one entry per reporting Sentinel.

**Algorithm (The standardisation bootstrap)** · `alg:warmup:standardisation-bootstrap`

Standardisation begins from feature-class priors and must replace them with measurements, in two situations that differ in what is new.

The system's own cold start is covered by the prior-mass ramp. Each accepted raw assessment vector retires an equal share of the priors' standardisation mass: the first advances the phase to `Transitioning`, and the accepted observation at $N_\text{init} = 100$ retires the last share and enters `InService` (`alg:standardisation:batch-initialisation`). What this corrects is the gross mismatch between the priors and the deployment's actual feature distributions, and it corrects it over the whole horizon rather than at one step. The phase and the accepted count are carried on every assessment throughout (`schema:output:health-snapshot`), so a host is never left reading the coordinate state off the assessment count: a refused observation and an owner still working through its backlog both leave the ramp behind the requests served, and neither is a fault.

The per-Sentinel bootstrap covers a Sentinel that registers later, into a system whose statistics have long since settled. Its features are standardised against the class priors until $N_\text{boot}$ assessments have accumulated — default one hundred — at which point bootstrapped statistics for that Sentinel are blended in (`alg:standardisation:sentinel-bootstrap`). Without it a late Sentinel would be standardised against a coordinate system that describes everything except itself (`cav:limitation:late-standardisation`).

The per-Sentinel bootstrap opens an accumulator at registration, fills it from assessments that Sentinel reports in, and blends it into the slot at the specified weight and sample count. The cold ramp starts from the specified priors (`tab:standardisation:class-priors`), retires one equal share of their mass per accepted raw vector, and publishes every advance. Its phase and accepted count travel on the compact and full health surfaces, so each result identifies its coordinate state.

**Example (Convergence at three deployment profiles)** · `ex:warmup:profiles`

Every Core figure below is the convergence budget evaluated at the profile's own rates (`bound:resource:convergence-budget`), and every Companion figure is its own bound evaluated at the profile's contributing-label rate (`bound:companion:convergence`). All three profiles assume sixty per cent eligibility and the reference dimension (`tab:resource:reference-configuration`).

| Profile | Core models | Companion, untargeted | Companion, well targeted | With override |
| --- | --- | --- | --- | --- |
| Low traffic — 10 req/s, 50 labels/day | ~42.5 days | Never | 135 days | **~42.5 days** |
| Medium traffic — 100 req/s, 200 labels/day | ~10.6 days | Never | 28 days | **~10.6 days** |
| High traffic — 1,000 req/s, 1,000 labels/day | ~2.1 days | 58 days | ~5 days | **~2.1 days** |

The Companion columns are the bound's decayed times rather than its arrival times, which is why two of them are never: at fifty and at two hundred labels a day the untargeted contributing rate puts the effective sample size ceiling below twenty, so no amount of waiting reaches it (`bound:companion:convergence`).

The Companion's own timeline varies by an order of magnitude with targeting quality alone, because the quantity that matters is not the challenge rate but the rate at which challenges produce contributing labels (`data:companion:contributing-rate`):

| Targeting quality | Contributing labels/day | Time to a usable estimate |
| --- | --- | --- |
| Untargeted, at the population rate | 0.08 | Never |
| Weakly targeted | 0.24 | 108 days |
| Moderately targeted | 0.48 | 48 days |
| Well targeted | 0.80 | 28 days |
| Host override supplied | — | 0 days |

The reading is the same at every profile: the Companion binds, unless the host supplies an override — and at untargeted rates it does not merely bind but never arrives at a usable estimate at all (`alg:companion:override`). A host that supplies one waits for the Core alone. A host that can supply neither an override nor targeted challenges will see wide crossover intervals for months (`def:landscape:credible-intervals`), and should consider the three-action space, which is markedly more robust to challenge effectiveness uncertainty (`prop:channel:challenge-width`). Hosts reading risk assessments without the derivation are unaffected by the Companion's timeline entirely.

**Table (The milestones)** · `tab:warmup:milestones`

| Milestone | Requirement |
| --- | --- |
| First risk assessment | 1 assessment, at $\hat{p} \approx 0.5$ and maximal uncertainty, in the feature-class-prior coordinate system — phase `WaitingForInit` and accepted count zero in the snapshot it acquired |
| Standardisation empirical | $N_\text{init}$ accepted cold-ramp observations; about 100 assessments absent skips or owner backlog |
| Anchor emergence | ~30 eligible labels |
| First calibration refit | ~200 labels |
| Aggregate-by-context interaction maturity | ~500 labels |
| Competitive set stabilisation | ~1–3 days at high traffic |
| Sister model convergence | ~$2p$ eligible labels |
| Per-Sentinel-by-context interaction maturity | ~2,000 labels |
| Competitive-range interaction maturity | ~2,000–5,000 labels |
| Cross-Sentinel interaction maturity | ~5,000+ labels |
| **First calibrated assessment** | max(200 labels, the Core's budget) |
| Companion: first useful estimate | ~10 contributing labels |
| Companion: practically useful variance | ~20 contributing labels |
| Companion: host override supplied | Immediate |
| **First calibrated landscape** | max(Core calibrated, Companion usable or overridden) |

The table is a reading aid and not a source. Every threshold in it is fixed somewhere else and cited from there: the refit cadence at the calibration chapter (`tab:platt:refit-cadence`), the interaction maturities at the convergence cascade (`tab:feature:interaction-convergence`), the Companion's two counts at its convergence bound (`bound:companion:convergence`), the sister's budget at the resource chapter (`bound:resource:convergence-budget`), and the standardisation count at batch initialisation (`alg:standardisation:batch-initialisation`). A restatement that cites its sources constrains nothing on its own account, and this one is offered as the single view a reader planning a deployment wants rather than as a further requirement on the implementation.

The standardisation row is the one whose figure is an approximation of a different quantity. What fixes that milestone is the accepted-observation target and the phase transition at it; the assessment count beside it is an operational approximation only, because contention and queue pressure may refuse observations and a deployment may therefore serve more than a hundred requests before the ramp reaches its horizon.

**Remark (What to trust before the first refit)** · `rem:warmup:pre-calibration`

Between the anchor's emergence and the first calibration refit the system produces scores that are ordered correctly and scaled wrongly. The calibration parameter still holds its initial value of one (`def:platt:initial-value`), which is a placeholder rather than a fit, so probabilities carry the model's ranking but not its magnitudes (`cav:limitation:pre-calibration`).

What this costs a host depends on what the host reads. A ranking, a queue order or a relative comparison survives the interval intact. An absolute probability does not, and neither does anything computed from one as though it were calibrated. Hosts using the derivation see approximate crossover locations throughout: interval widths move in the correct direction regardless of the calibration parameter's absolute value, while locations shift with the shared term and settle at the first refit (`thm:landscape:rigid-translation`).

The interval is visible rather than silent. The calibration record counts on the risk basis report how much evidence each regime's fit rests on (`schema:risk:basis`), and a count below the minimum is the explicit signal that the system is operating before calibration (`req:platt:minimum-samples`); the effects of crossing that boundary are analysed with the regimes themselves (`disc:platt:regime-transition`). A host that needs calibrated probabilities from its first day has one remedy, which is to arrive with history: pre-seeding with at least two hundred historical labels moves the deployment past this stage before it serves live traffic (`alg:host:pre-seeding`).

This section is deployment awareness rather than a constraint on the implementation. It tells an operator what not to trust and for how long, and it fixes no threshold that the code could satisfy or violate.

**Example (Recovery after atrophy, a transient event and a policy change)** · `ex:warmup:recovery`

Recovery is not one timescale. After a disturbance the measurement layer returns in hours and the memory layers return over weeks, because they are built to remember, and an operator watching only the fast layer will believe the system has recovered while its slower features still carry the disturbance.

Restriction-induced atrophy, where the host's own restriction removes the traffic a region was known by:

| Time | Core observable | Risk effect |
| --- | --- | --- |
| 0 | Scores in affected cells may normalise | Measurement-derived features fall |
| $N_\text{absent}$ cycles | Affected entries deleted (`alg:ledger:entry-deletion`) | Routing falls back to the ancestor entry |
| 14 days | Identity cell outcome features halved | Moderate identity-derived residual |
| 29 days | Ledger ancestor features reflect recent labels only | Ledger contribution reflects current evidence |

A transient adverse event that ends of its own accord:

| Time after the event ends | Core observable | Risk effect |
| --- | --- | --- |
| Hours | Measurement scores normalise | Measurement features return to baseline |
| 14 days | Identity cell averages halved | Moderate residual through the identity pathway |
| 29 days | Ledger averages halved | Modest residual |
| 87 days | Ledger averages at about an eighth | Negligible |

A host-initiated temporal policy change, where nothing is wrong and the ground has moved anyway: at the next batch report the reported cell set may change and is reconciled by cell-set maintenance (`alg:runtime:cell-set-maintenance`); over hours to days the set stabilises and new entries begin accumulating outcome history; over days to weeks time-indexed decay dissolves the pre-change Ledger state and post-change labels establish new averages.

The two decay horizons behind every figure above are the identity layer's fourteen days (`tab:keyspace:decay-rates`) and the Ledger's twenty-nine (`def:ledger:time-decay`). Neither is a recovery mechanism that acts; both are bounds on how long a stale impression can persist, and the timelines are arithmetic on them rather than claims of their own.

### Chapter (Convergence and Resource Bounds) · `chap:spec:convergence-and-resources`

What the system costs to converge and what it costs to run, in one place. The chapter is arithmetic on parameters fixed elsewhere: the dimension the feature chapter constructs, the update the label chapter performs, the buffer the publication chapter sizes. It fixes one thing of its own, which is the reference configuration every figure in the document is quoted at.

The cost tables are cited from the operational chapters rather than duplicated into them. That is deliberate and it is the discipline that keeps two copies of a cost from drifting apart: the tables mint here and the chapters that spend the cost cite them.

**Bound (The convergence budget)** · `bound:resource:convergence-budget`

A Bayesian linear model with $p$ parameters needs on the order of $2p$ observations before its posterior is shaped by data rather than by its prior. That is the whole of the rule of thumb, and everything below is it evaluated somewhere.

The system has two independent timelines and the host waits for the later:

$$T_{\text{system}} = \max\!\big(T_{\text{core}},\; T_{\hat{q}_c}\big)$$

The Core's own timeline is sequential, because each stage needs the one before it:

$$T_{\text{core}} = T_{\text{spatial}} + T_{\text{warm-up}} + T_{\text{baseline}} + T_{\text{assayer}}$$

| Stage | Formula | Typical range |
| --- | --- | --- |
| $T_{\text{spatial}}$ | $\theta / R_{\text{obs}}$ | Seconds to hours |
| $T_{\text{warm-up}}$ | $\text{noise rounds} / R_{\text{noise}}$ | Seconds to minutes |
| $T_{\text{baseline}}$ | $400 / R_{\text{batch}}$ | Minutes to hours |
| $T_{\text{assayer}}$ | $2p / (R_{\text{label}} \times f_{\text{elig}})$ | Days to months |
| $T_{\hat{q}_c}$ | $20 / R_{\text{contributing}}$ | Days to months |

The first three stages belong to the spatial index and the spectral Sentinel rather than to this system, which observes their completion only indirectly through the batch report; they are named here because a host waiting for calibrated output waits for them too, and they are measured in seconds and minutes against the last stage's days and months. The Companion's timeline runs in parallel and is not part of the sum; its row above is the no-decay arrival time, and at untargeted rates the effective sample size ceilings below twenty, so the wait there is not long but unending (`bound:companion:convergence`). A host reading risk assessments alone waits for $T_\text{core}$ and nothing else.

The last stage is the one that matters, and it is $2p$ eligible labels divided by the rate at which eligible labels arrive — the single convention used wherever a convergence figure appears in this document. Eligibility is what turns a label rate into an arrival rate (`tab:eligibility:training`), so a deployment that labels heavily but confounds most of it converges slowly:

| Deployment | $n$ | $m_s$ | $p$ | Eligible labels to the sister | At 200 labels/day, 60% eligible |
| --- | --- | --- | --- | --- | --- |
| Minimal | 3 | 0 | 286 | 572 | ~4.8 days |
| Standard | 8 | 1 | 638 | 1,276 | ~10.6 days |
| Standard+ | 8 | 3 | 682 | 1,364 | ~11.4 days |
| Large | 12 | 1 | 910 | 1,820 | ~15.2 days |

Each $p$ above is the dimension formula evaluated at that deployment's Sentinel and axis counts rather than an estimate (`tab:feature:dimension-formula`), and each day figure is $2p$ divided by the hundred and twenty eligible labels a day that the stated rates produce. The anchor is exempt from all of it: fifteen parameters converge within about thirty labels whatever the deployment's size, which is why the system has a usable direction long before it has a converged sister (`def:risk:anchor-model`).

**Proposition (Adding a Sentinel or axis after convergence)** · `prop:resource:incremental-addition`

Adding a Sentinel to a converged system does not reconverge it. The new dimensions are independent of the existing ones under the prior (`thm:gaussian:extension`), so the model needs on the order of $q + 1$ eligible labels to learn the new Sentinel's weights rather than the $2p$ a cold start costs, and the existing dimensions carry on at their converged values while the new ones fill in.

Adding an outcome axis costs on two accounts, which are worth separating. Every existing model gains the axis's feature weights and needs on the order of $r_a$ labels to learn them, which is incremental in the same sense. The axis's own prediction model is a new model, starting from its prior at dimension $p + r_a$, and it needs on the order of $p + r_a$ labels — a full convergence, because it is a full model (`def:axis:per-axis-model`).

The property is what makes the registries usable in production rather than only at design time. A deployment can add a source of evidence without paying the cold start again, and the cost of doing so is legible in advance from the width the new source contributes (`def:extraction:slot`).

**Table (Per-assessment cost)** · `tab:resource:assessment-cost`

| Operation | Cost |
| --- | --- |
| Coordinate routing, all Sentinels | $O(n_s \log \lvert\mathcal{A}^*\rvert)$ |
| Per-Sentinel extraction and alarm summary | $O(n \times q)$ |
| Aggregation | $O(n)$ |
| Identity observation and features | $O(D \cdot d_\text{geo} + \sum_d \lvert\mathcal{E}_d\rvert)$ |
| Feature assembly and standardisation | $O(p)$ |
| Risk estimates, two at $p$ and the anchor at $p_a$ | $O(2p^2 + p_a^2)$ |
| Outcome predictions, $m$ models | $O(m \cdot p^2)$ |
| Pending buffer write | $O(p)$ |
| **Total at $p = 638$, $m = 1$** | **~1.22M operations, ~1.22 ms** |

The quadratic terms dominate and they are the posterior reads, which is the expected shape for a model of this class (`tab:gaussian:operation-costs`). The pipeline these operations belong to is specified with the assessment interface (`alg:runtime:assessment-pipeline`). The derivation's own cost is not in this table and does not belong in it: it is linear in the declared actions plus a constant, negligible beside the above, and paid by the host outside the Core (`sig:landscape:derivation-function`).

**Table (Per-label cost)** · `tab:resource:label-cost`

| Operation | Cost |
| --- | --- |
| Reconstruction and re-standardisation | $O(p)$ |
| Operational model update | $O(p^2)$ |
| Sister model update, if eligible | $O(p^2)$ |
| Anchor model update, if eligible | $O(p_a^2)$ |
| Outcome axis models, if eligible | $O(m_\text{elig} \cdot p^2)$ |
| Standardisation update | $O(p)$ |
| Ledger update, all-layers routing | $O(n_s \cdot d_\text{max})$ |
| Calibration refit, amortised | $O(N_\text{cal,buf}) / N_\text{refit}$ |
| **Total at $p = 638$, $m = 1$** | **~1.22M operations, ~1.22 ms** |

A label costs about what an assessment costs, which is the useful thing to know when sizing a deployment: the label path is not the cheap path. The steps are enumerated where the update is specified (`alg:runtime:update-path`), and the conditional rows are conditional on the eligibility table rather than on anything this chapter decides. No challenge-effectiveness update appears here, because the Companion is updated by the host and not on this path (`alg:companion:update`).

**Table (Memory summary)** · `tab:resource:memory`

At the reference configuration, with model matrices in double precision:

| Component | Size |
| --- | --- |
| Core models, two at $p = 638$ | ~12.4 MB |
| Anchor model, $p_a = 15$ | ~7.4 KB |
| Outcome axis models, $m = 1$ | ~6.2 MB |
| Standardisation | ~10.2 KB |
| Calibration buffer | ~64 KB |
| Identity layer, two graphs | ~30 MB |
| Signal cache, 100K entities | ~8 MB |
| Pending buffer, single-precision features | **~3.0 GB at 1M capacity** |
| Outcome Ledger | ~32 MB |
| Routing indices | ~2 MB |
| **Core total** | **~3.1 GB at 1M capacity** |
| Companion tracker, per channel | ~100 bytes |

Each full model holds a mean, a precision and a covariance, and the two matrices dominate it at $2p^2$ doubles apiece; per-model memory therefore grows as $O(p^2)$, and double precision is fixed because the precision-health machinery depends on it (`alg:gaussian:synchronisation-monitor`). Everything else on the list is rounding error beside one row. The pending buffer is ninety-seven per cent of the total, it stores features at single precision (`def:runtime:storage-precision`), and right-sizing its capacity to the deployment's request rate and labelling latency is the only memory lever worth pulling (`req:publication:pending-buffer`). The one-million figure is the capacity at which this table is computed; the label chapter instead specifies capacity as a value computed from rate and latency (`req:runtime:buffer-capacity`), so a deployment that chooses a million entries is paying three gigabytes for that capacity.

**Table (The reference configuration)** · `tab:resource:reference-configuration`

| Parameter | Value |
| --- | --- |
| Sentinels ($n$) | 8 |
| Outcome axes ($m$) | 1, with spatial features enabled, so $m_s = 1$ |
| Identity dimensions ($D$) | 2, at about ten competitive cells each |
| $p_\text{sig}$ | 0 |
| Per-Sentinel extraction width ($q$) | $60 + 2 \times 1 = 62$ |
| Slot width | $q + 1 = 63$ |
| $p_\text{agg}$ | 15 |
| $p_\text{id-dim}$, per dimension | $8 + 3 \times 1 = 11$ |
| $p_\text{id-cross}$ | 8 |
| $p_\text{int}$ | $5 \times 8 + 8 + 20 = 68$ |
| $p_\text{id-comp}$ | $10 + 10 = 20$ |
| **$p$** | $1 + 15 + 11 \times 2 + 8 + 0 + 8 \times 63 + 68 + 20 = \mathbf{638}$ |

This is the configuration every worked figure in the document is quoted at, and it is stated as a table so that a reader can tell which of those figures move when a deployment differs. The construction is the feature vector's blocks in order — the bias, the aggregate block (`tab:feature:aggregate-block`), the per-dimension and cross-dimension identity blocks (`tab:keyspace:dimension-features`), the signal block, the eight Sentinel slots, the interaction block and the competitive indicators (`def:feature:vector-structure`) — and the full construction with every block's derivation is the dimensional summary (`app:spec:dimensional-summary`).

The total is exact rather than approximate: the blocks sum to six hundred and thirty-eight. The same construction evaluated at the other three deployment sizes of this chapter's budget gives their dimensions exactly too, which is the check worth having, because a reference configuration whose arithmetic only works at one point is a worked example rather than a reference.

### Chapter (Detection Analysis) · `chap:spec:detection-analysis`

The chapter argues rather than specifies. Its subject is what the composed design achieves that no layer of it achieves alone, and its claims are about the system as assembled in the earlier Parts rather than constraints on how those Parts are built.

Two environments are exceptions and they are marked as such: what the divergence measures and where a host may read it, and the rule that the Core reports one figure per assessment and knows nothing of channels. The rest is reasoning, and it is carried rather than compressed because one piece of it — the admission that the avoidance argument covers the measurement layer and not the memory layer — is what keeps the laundering exposure honest.

**Table (The detection stack)** · `tab:detection:stack`

| Layer | Mechanism | What must be normal for avoidance | Convergence |
| --- | --- | --- | --- |
| Per-Sentinel, per-value | Ancestor chain (`tab:extraction:chain-z-scores`) | Each value normal at every scale | Hours |
| Per-Sentinel, cross-cell | Coordination (`tab:extraction:coordination`) | Cross-cell score distribution normal | Hours |
| Cross-Sentinel, per-axis | Aggregate features (`tab:feature:aggregate-block`) | Each axis's cross-Sentinel profile | Days |
| Cross-Sentinel, compound | Slots and interactions (`tab:feature:default-interaction-set`) | Sentinel-specific and joint patterns | Weeks–months |
| Risk estimation, sister | Unconfounded training (`def:risk:model-triple`) | Inherent risk survives confounding | Weeks |
| Risk estimation, anchor | Coarse and measurement-only (`prop:risk:anchor-measurement-only`) | Correct direction while the sister is starved | Days |
| Spatial history | Outcome Ledger (`def:ledger:purpose`) | Adverse regions never flagged | Days–weeks |
| Identity history | Competitive cells (`tab:keyspace:outcome-state`) | Key range profiles and transitions | Days–weeks |
| Outcome prediction | Axis models (`def:axis:per-axis-model`) | Predicted secondary outcomes | Weeks |

The table's substantive claim is in its third column. Each layer addresses a different avoidance strategy, so a source constructed to look normal at one layer is not thereby normal at another, and the layers converge on different timescales, so the stack is never uniformly cold: the fast layers cover while the slow ones learn. That is the composition's whole detection argument in one view, and the two discussions below are its two halves — what the composition buys, and what it does not.

**Discussion (Cross-Sentinel compound detection)** · `disc:detection:compound`

Per-Sentinel identity survives aggregation, because each Sentinel's features occupy a named slot rather than being averaged away (`def:extraction:slot`). The Core can therefore learn that a pattern matters at one Sentinel and not at another, which a Sentinel-count-invariant summary alone could never express. Joint structure across Sentinels is reachable two ways: explicitly, by naming the pair a host expects to matter (`def:feature:template-named-pair`), and implicitly, by letting the wildcard template enumerate the pairs (`def:feature:template-wildcard`). The aggregate features sit alongside both and provide the summary that does not depend on how many Sentinels are reporting (`sec:feature:aggregate`).

What the architecture discards is worth stating as plainly as what it keeps. Within-batch, per-observation differentiation from ambient cell state is gone. Two requests arriving in the same batch at the same Sentinel cell receive identical per-Sentinel features, and are distinguished only by their entity, their signals, cross-Sentinel variation and Ledger state. The Core reads batch summaries and never per-request scores, so nothing that happens between two reports is visible to it at all (`cav:limitation:inter-report`). This is a deliberate trade of resolution for the ability to read many Sentinels at once, and it is the reason the stack's fastest layer is measured in hours rather than in requests.

**Discussion (The multi-objective avoidance dilemma)** · `disc:detection:avoidance-dilemma`

A source seeking to avoid the whole stack faces three objectives in mutual tension. Per-value avoidance pushes towards uniformity. Cross-cell diversity requires variation. Cross-Sentinel consistency requires profiles that are jointly normal, not merely normal one at a time. Satisfying any one of them makes the others harder.

The composition therefore raises avoidance to a constrained multi-scale, multi-cell, multi-source, multi-axis optimisation with no known efficient solution except one: produce values genuinely indistinguishable from normal at every scale simultaneously, which is not avoidance but ordinary behaviour. That is the argument the stack exists to support.

Its scope is narrower than it first appears, and the qualification is the most important sentence in the chapter. The argument concerns evasion of the *measurement* layer. It does not apply to the *memory* layer. The Ledger's write path runs through labels on cells (`def:ledger:purpose`) and the identity layer's competitive weights are driven by observed, labelled traffic (`def:keyspace:competitive-set`); both can therefore be moved by traffic that is genuinely normal. Being statistically normal defeats measurement-layer detection and is precisely what memory-layer laundering requires, so the two exposures are complementary rather than alternative, and the more accessible of the two is the one this argument does not cover (`cav:limitation:laundering`).

**Table (The cross-Sentinel convergence window)** · `tab:detection:convergence-window`

| Template type | Convergence | Coverage during convergence |
| --- | --- | --- |
| Aggregate by context (`def:feature:template-aggregate`) | Days | Available from the first days |
| Per-Sentinel by context (`def:feature:template-per-sentinel`) | Weeks | Available from the first weeks |
| Cross-Sentinel, named pairs (`def:feature:template-named-pair`) | Months | Partial: aggregate features give coarse coverage |
| Cross-Sentinel, wildcard (`def:feature:template-wildcard`) | Months | Partial: aggregate features give coarse coverage |

The window is a consequence of the convergence cascade rather than a separate schedule (`tab:feature:interaction-convergence`). What is unavailable during the months the cross-Sentinel templates take is one specific capability: detecting sources that evade each Sentinel independently while producing concealment patterns correlated across them (`cav:limitation:cross-sentinel-gap`). The lower tiers provide continuous coverage throughout, so the window is a period of reduced resolution and not a gap in detection.

**Definition (What the divergence measures and where it may be read)** · `def:detection:divergence-scope`

The divergence is the difference between the blended unconfounded estimate and the operational model's realised risk:

$$\Delta(\phi) = \hat{\rho}_\text{eff}(\phi) - \hat{\rho}_\text{opr}(\phi)$$

A positive divergence means the unconfounded estimate exceeds what the host's own restricted population realises — the host's interventions are doing work (`def:risk:intervention-effectiveness`). It is reported once per assessment on the risk basis.

The scope condition is the definition's substance. The divergence is interpretable only where unconfounded coverage exists. The sister term is identified as inherent risk only where eligible labels exist (`def:risk:model-triple`), so in feature regions the host has always restricted, that term is prior extrapolation rather than measurement, and the difference there measures the distance between an extrapolation and a confounded estimate. It does not measure intervention value, and it does not announce that it is not doing so.

A host must therefore read the divergence only where eligible-label density is adequate, and the system offers two instruments for knowing where that holds: the resolution-utilisation ratio, which reports the fraction of cells whose discrimination is backed by outcome evidence (`def:monitoring:resolution-utilisation`), and the starvation guidance, which names the cells most lacking it (`def:guidance:starvation`). Host investigation is the only mechanism that restores identification in a region that has always been restricted (`conv:eligibility:ground-truth`). Where the restriction is driven by information the feature vector does not carry, the aggregate figure stays directionally correct while per-feature attribution becomes misleading (`cav:limitation:divergence-attribution`).

The divergence is computed and carried exactly as defined above, and its scope condition is read through the two instruments this definition names.

**Requirement (One figure per assessment and no channel dimension)** · `req:detection:per-channel-reporting`

The Core reports a single divergence per assessment. It carries no channel dimension, because the Core has no concept of a channel and cannot acquire one without acquiring the decision surface it is built not to have (`prin:principle:measure-not-decide`).

A host that wants intervention effectiveness per channel therefore computes it per channel, by aggregating the per-assessment figures through its own routing knowledge. This is not a deficiency to be repaired by adding a dimension. The routing is the host's, the channel definition is the host's, and a Core that reported per-channel figures would be asserting a partition it does not own (`sig:landscape:derivation-function`).

### Chapter (Health Monitoring) · `chap:spec:health-monitoring`

The system is asked to say when it is not working. This chapter is that apparatus: drift in the models' own predictions, discrimination against outcomes, the numerical health of the posteriors, statistical signatures of corrupted labels, the cross-layer quantities that reveal whether the layers are being used at all, and three scenarios that show the whole loop turning.

Every environment below states its requirement at full strength. The observability summary (`tab:monitoring:observability-summary`) indexes those environments rather than specifying an additional quantity.

**Invariant (Health monitoring reports and never acts)** · `inv:monitoring:report-only`

No diagnostic in this chapter modifies a model, a threshold, a rate, or a routing decision. Each computes a quantity, reports it, and stops.

The posture is deliberate and it is what makes the diagnostics safe to add. A monitor that acted would close a loop between the system's estimate of its own health and the estimates that health is measured on, and the resulting behaviour would be neither the specified behaviour nor a diagnosable departure from it. Reporting keeps the loop open and puts the host at the point where it closes.

**Table (Standardisation transition health)** · `tab:monitoring:standardisation-transition`

The cold standardisation ramp's progress is reported on both health surfaces and gates nothing (`inv:monitoring:report-only`).

| Reading | Compact assessment snapshot | Full health report |
| --- | --- | --- |
| Standardisation phase | Of the coordinate snapshot the result was scored in | At the moment the report was taken |
| Accepted cold-ramp observations | The count that snapshot represents | The count at the moment the report was taken |
| Model snapshot version | Present, fixing the coordinate state the result used | Present, as the report's own join key |
| Features at the variance floor | Absent; it is per-position | Present, as before |

The three phases are readings of one count rather than a second quantity (`alg:standardisation:batch-initialisation`). `WaitingForInit` is a count of zero and moments that are the class priors exactly; `Transitioning` is a positive count below $N_\text{init}$ and moments that are a mixture; `InService` is a count of $N_\text{init}$ with no cold-prior mass left. A reader holding the count holds the phase, and the phase is reported anyway, because a quantity a reader must reverse-engineer from the state it describes is one a reader will eventually reverse-engineer wrongly and then alert on.

The compact pair and the full pair answer different questions, and a host needs both. The compact pair is retrospective: it describes the coordinate system one result was computed in, which is what makes two results comparable or tells a reader that they are not. The full pair is current: it says where the ramp stands now, which is what an operator watching a cold start is asking. During a transition the two disagree by however many advances fell between them, and that disagreement is information rather than an inconsistency.

Queue refusal and contention counters remain degradation diagnostics and are not read as accepted observations. A refused observation advances no count, so the gap between requests served and observations accepted is exactly what those counters report; a reader who added them to the accepted count would be counting each skipped observation twice and would place the ramp further along than it is.

The phase and the accepted count are stored state and are carried on the compact assessment snapshot and the full health report exactly as the table specifies. Inferring the phase from variance floors is invalid because that inference cannot separate a genuinely small observed variance from a position the ramp has not moved yet.

**Algorithm (Per-model drift accumulators)** · `alg:monitoring:drift-cusums`

Each Core model — operational, sister, anchor, and each outcome axis model — maintains two one-sided accumulators over the residual between what it predicted and what the label reported:

$$S^+_t = \max\big(0,\; S^+_{t-1} + (r - \hat{\rho}(\phi)) - \kappa_\text{drift}\big)$$

$$S^-_t = \max\big(0,\; S^-_{t-1} + (\hat{\rho}(\phi) - r) - \kappa_\text{drift}\big)$$

Here $r$ is the model's training target (`def:risk:target`), or the axis target for an axis model (`def:axis:training-target`), and $\hat{\rho}(\phi)$ is the prediction made at **assessment time** and stored in the pending entry (`def:runtime:pending-entry`) rather than a prediction recomputed after the update. The distinction matters: a post-update prediction would compare the model against a target it has already moved towards, and the accumulator would understate every drift it exists to catch.

The noise allowance $\kappa_\text{drift}$ is what the accumulator forgives before it begins to accumulate, at a default of $0.1$. It sets the sensitivity of the whole mechanism: too small and ordinary noise accumulates, too large and real drift is absorbed silently.

The recursions are updated from the assessment-time prediction, and both the allowance and reset threshold are read from `MonitoringConfig`. The allowance boundary is inclusive and the reset boundary is strict.

**Table (Running drift diagnostics)** · `tab:monitoring:drift-diagnostics`

| Diagnostic | Computation | What it detects |
| --- | --- | --- |
| Mean absolute residual | Running average of $\lvert r - \hat{\rho}(\phi) \rvert$ | Overall prediction quality |
| Steps since reset | Counter since the last accumulator reset | How long drift has been accumulating |
| Residual sign | Running average of $\text{sign}(r - \hat{\rho}(\phi))$ | Sustained directional bias |

The three answer different questions and are reported together because a reader needs all three to interpret any one. A large mean residual with no sign bias is noise; a small mean residual with a persistent sign bias is a model drifting in one direction and is the more serious finding. The step counter supplies the duration that turns either into a rate.

**Table (The reset mechanisms)** · `tab:monitoring:drift-resets`

| Trigger | Condition | Effect |
| --- | --- | --- |
| Automatic | $S^+ > h$ or $S^- > h$, default $h = 10$ | Reset the accumulators and report a drift event |
| Host-initiated | A manual reset request | Reset the accumulators |
| Post-lifecycle | After a Sentinel or axis lifecycle event | Reset the accumulators for all models |
| Post-calibration | After a calibration shift exceeding $0.1$ | Reset the accumulators for the affected regime |

Three of the four resets exist because the residual's meaning changed rather than because the drift resolved. A lifecycle event alters the feature space the prediction was made in (`alg:registry:sentinel-registration`), and a calibration shift alters the scale the residual is measured on (`alg:platt:drift-integration`); in both cases the accumulated evidence is about a quantity that no longer exists, and carrying it forward would report a drift that is really a change of coordinates.

A drift flag derived from these accumulators belongs in the compact health snapshot, so that a host reading a single assessment can see that the model behind it is drifting (`schema:output:health-snapshot`). The automatic reset reports a drift event, and the compact flag rises when any accumulator reaches half the automatic-reset threshold so an assessment can report the approach before the reset clears it.

**Algorithm (Discrimination by rank)** · `alg:monitoring:auc`

At each calibration refit (`tab:platt:refit-cadence`) the Core computes rank discrimination from the prediction-outcome pairs held in the calibration buffer (`constr:platt:buffer`). Four figures:

The aggregate figure is computed over all buffer entries. The sister-regime and anchor-regime figures are computed over the entries whose blend weight falls below and at or above $0.3$ respectively, which separates the two regimes the blend actually operates in (`def:risk:subspace-blend`). Each is reported only when its partition holds at least $N_\text{auc,min}$ positive and $N_\text{auc, min}$ negative outcomes, default twenty, because a rank statistic over fewer is noise with a decimal point.

The recent figure is computed over the most recent $N_\text{recent}$ labels, default five hundred. When it departs from the aggregate figure by more than $0.1$ a **discrimination instability flag** is reported. The recent figure is the faster signal and the noisier one; the aggregate lags because the buffer spans about ten days at the reference rate (`cav:limitation:auc-lag`).

The computation uses rank discrimination with tie correction, both regime partitions, and the instability flag, with both configured sample gates. The recent window remains a fixed count rather than a proportion of the buffer, so it holds the constant span the figure's interpretation assumes instead of growing and shrinking with the buffer.

**Definition (Top-decile lift)** · `def:monitoring:top-decile-lift`

$$\text{lift}_{10} = \frac{\text{precision at the top decile}}{P_{+,\text{buffer}}}$$

The precision among the highest-scoring tenth of the buffer, divided by the buffer's own positive rate. It answers the question a rank statistic does not: not whether the ordering is right overall, but whether the top of it is worth acting on, which is the part of the ordering a host actually intervenes on.

**Definition (Per-axis correlation)** · `def:monitoring:axis-correlation`

For each active outcome axis carrying at least the minimum number of reported values, the linear correlation between the axis predictions stored at assessment time and the values the host later reported. It is the axis analogue of rank discrimination, and it is a correlation rather than a rank statistic because axis outcomes are continuous where the risk target is binary (`def:axis:training-target`). It is computed per axis from the calibration rows and reported.

**Table (Interpretation bands)** · `tab:monitoring:interpretation`

| Metric | Good | Moderate | Weak | Near-chance |
| --- | --- | --- | --- | --- |
| Rank discrimination | $\geq 0.80$ | $[0.65, 0.80)$ | $[0.55, 0.65)$ | $< 0.55$ |
| Top-decile lift | $\geq 5$ | $[2, 5)$ | $[1.5, 2)$ | $< 1.5$ |
| Axis correlation | $\geq 0.6$ | $[0.3, 0.6)$ | $[0.1, 0.3)$ | $< 0.1$ |

The bands are reading guidance and fix nothing. They exist because the three metrics have genuinely different scales, and an operator who reads a correlation of $0.5$ as though it were a rank discrimination of $0.5$ draws the opposite conclusion from the correct one.

**Algorithm (Empirical coverage of the reported uncertainty)** · `alg:monitoring:empirical-coverage`

The two-level guarantee establishes that reported uncertainty is conservative in sign — wider than a fixed-model posterior — but bounds neither the magnitude of that conservatism nor its stability (`thm:gaussian:two-level-guarantee`). Every downstream consumer of uncertainty inherits the effective width: the blend weight, the crossover intervals (`def:landscape:credible-intervals`), the fragility flip probabilities (`def:fragility:definition`), and the exploration ranking (`def:guidance:risk-informative`). An unmeasured inflation therefore reorders exploration queues and diffuses decision boundaries without announcing itself anywhere. This diagnostic measures it directly.

No additional per-request state is needed. The calibration buffer already stores the estimate, its uncertainty and the outcome per entry.

For each buffer entry, at the current regime's calibration parameter (`def:platt:objective`), form the probability-scale prediction and its uncertainty (`def:risk:probability`) and compute the standardised residual:

$$z_i = \frac{y_i - \hat{p}_i}{\sigma_{\hat{p},i}}$$

Two coverage fractions are then reported — the empirical fractions of entries with $\lvert z_i \rvert < 1$ and $\lvert z_i \rvert < 1.96$. Under calibrated Gaussian uncertainty these would sit near $0.68$ and $0.95$; the guarantee tells us not to expect exact calibration (`cav:gaussian:fixed-model-departure`), so the diagnostic measures the departure rather than testing a hypothesis. From the same residuals comes the single actionable number, the inflation factor: the $0.975$-quantile of $\lvert z_i \rvert$ divided by $1.96$, which is the factor the uncertainty would need rescaling by to achieve nominal coverage.

| Inflation factor | Interpretation | What a host may do with the intervals |
| --- | --- | --- |
| $< 1.3$ | Approximately honest | Intervals and fragility are quantitatively trustworthy |
| $1.3$–$3$ | Conservative, as the guarantee predicts | Intervals interpretable as bounds; fragility overstates flip risk modestly |
| $> 3$ | Grossly conservative | Widths and exploration ranking dominated by the artefact; read as a convergence-state signal |

The diagnostic is report-only and never rescales the uncertainty (`inv:monitoring:report-only`). Because outcomes are binary the per-entry residual is coarse and coverage is meaningful only in aggregate, so the minimum sample gate is the calibration minimum, per regime (`req:platt:minimum-samples`).

It is computed at each calibration refit and reported, with the inflation factor also carried on the compact health snapshot.

**Definition (The synchronisation error)** · `def:monitoring:synchronisation-error`

$$\epsilon_\text{sync} = \lVert B\tilde{\Sigma} - I \rVert_F$$

The Frobenius norm of the residual between the precision matrix, its maintained inverse, and the identity — a single number over the whole matrix, and the quantity whose growth means the two representations have drifted apart. The quantity this definition fixes is that whole reading, and it is what the health surface publishes under this name. Its threshold $\epsilon_\text{sync,thresh}$, default $10^{-6} \cdot p$, is the value above which a reading is abnormal at that model's width. What the recomputation cadence tests against the threshold is not the whole reading but each of its two components separately, as they are separated below: a residual over the threshold calls for a recomputation and shortens the interval, and a prior-induced component over it calls for a recomputation without shortening, because that component is one no interval reduces and one only a recomputation absorbs (`dec:posterior:adaptive-cadence`). The reading and the threshold are this definition's; which component of the reading the cadence consumes is that decision's, and the two are stated apart because a definition that folded the cadence's choice into the quantity would have to be rewritten whenever the choice moved.

This is the definition's only site in the document. The Gaussian chapter's monitoring procedure cites it rather than restating it (`alg:gaussian:synchronisation-monitor`), and the single site is deliberate: a norm over a matrix and a maximum over that matrix's diagonal are different quantities that a restatement can silently interchange, and the document protects against that by defining the quantity once.

**The reading carries its own resolution, and a residual below that resolution is not a measurement.** The reading is computed by forming the product $B\tilde\Sigma$ and taking the Frobenius norm of its distance from the identity. The standard entrywise bound for a floating-point matrix product at width $p$ is $\lvert \operatorname{fl}(B\tilde\Sigma) - B\tilde\Sigma \rvert \le \gamma_p \lvert B \rvert \lvert \tilde\Sigma \rvert$ with $\gamma_p = pu/(1-pu)$ at unit roundoff $u$, so the reading carries an absolute uncertainty of at most $\gamma_p \lVert\, \lvert B \rvert \lvert \tilde\Sigma \rvert \,\rVert_F$. That is the reading's resolution, and it is reported beside the reading. Two properties make it the right bar for the residual below. It is read off the *operands* and not off the answer, so a pair whose products cancel by orders before they reach their sum reports a resolution orders above one whose products do not — which is the case a competitive geometry produces and the case a figure scaled from the answer would misjudge in both directions. And it introduces no constant of its own: $\gamma_p$ is the width's, and the operand magnitudes are measured. A residual at or below this level is the rounding of the subtraction that produced it rather than evidence of drift, and is reported as being at resolution. It is reported and not acted on. A pair that has drifted far apart reports a resolution large enough to cover any residual at all, so a resolution is not permitted to overrule the threshold: doing so would fall silent exactly where the reading had stopped meaning anything.

**The reading has two components and they have different meanings, and both are computed and reported.** One is the rounding the incremental maintenance accumulates, which is what the monitor exists to catch and what a recomputation removes. The other is put there by the prior: the replenishment clamp raises a diagonal entry on every label at which a coordinate has decayed below the floor (`req:gaussian:prior-replenishment-floor`), and the maintained covariance does not follow that addition, so the tracked pair disagrees by the clamp's contribution seen through the covariance — $\lVert \operatorname{diag}(c)\tilde\Sigma \rVert_F$ for a clamp contribution $c$, exactly rather than approximately, since under decay the precision matrix is its decayed self plus exactly that contribution. The contribution that belongs in that expression is the one accumulated since the last adopted recomputation, not the one the model has accumulated over its life: a recomputation takes the covariance from the precision matrix the clamp has already raised, so everything added before it is absorbed into the pair and is no longer a disagreement to attribute. The two coincide only on a model that has never recomputed. On a model with coordinates resting at the floor the second component dominates the first by orders, it scales with the recomputation interval rather than with the width, and no recomputation reduces it below one interval's worth. The spectral floor has no term here, and that is a consequence of where it is applied rather than of its size: it is added to the precision matrix at the recomputation that is rebuilding the covariance from that same matrix (`dec:posterior:spectral-floor`), so the pair it leaves is consistent by construction and it contributes nothing for the monitor to read.

That component is conservative in direction — the covariance is wider than the inverse precisely along the directions the model has no evidence for — so it is not a fault, and it is reported rather than removed. What is no longer true is that it is indistinguishable from drift. Both components are quantities the model can compute, since the clamp's contribution is tracked per coordinate, and the monitor computes both at each recomputation and publishes three figures per model: the measured reading $\epsilon_\text{sync}$ as defined above, the part of it the prior put there, and the residual, which is the measured reading less that part, floored at zero. The maxima of all three across the models are carried on the compact health tier. The residual is the quantity the cadence acts on (`dec:posterior:adaptive-cadence`), which is what ends the behaviour this definition previously recorded as a consequence: a model with exhausted coordinates no longer shortens its interval to the floor and stays there on a quantity shortening does not remove. The threshold above is unchanged and is applied to the residual; the measured reading remains what this definition fixes, and is what the health surface publishes under the name. One consequence is worth stating for whoever reads the surface: on a model with coordinates at the floor the measured figure is large and grows with the recomputation interval, so lengthening the interval raises it. That is the prior's contribution scaling and not a degradation — the reading is $\sqrt{n}(1-\gamma^k)\gamma^{-k}$ for $n$ floored coordinates at forgetting rate $\gamma$ over an interval of $k$ labels, in which the floor's own value cancels — and it is conservative in direction. An operator watching for drift reads the residual (`test:crate:the-studys-reading-is-the-prior-and-its-residual-is-rounding`).

**Definition (The condition estimate, and the condition number)** · `def:monitoring:condition-number`

$$\rho(B) = \max_j B_{jj} \,/\, \min_j B_{jj} \qquad \kappa(B) = \lambda_\text{max}(B) \,/\, \lambda_\text{min}(B)$$

Two quantities, not one, and the distinction is what this definition exists to keep. The first is the ratio of the largest to the smallest diagonal entry: an $O(p)$ pass, read on every label. For a symmetric positive definite matrix it is a *lower bound* on the condition number and not an estimate of it, because the extreme diagonal entries are Rayleigh quotients at the coordinate directions and the extreme eigenvalues are the extrema over all directions. The gap is not a technicality at the widths a deployment reaches: the replenishment floor fixes the bound's denominator (`req:gaussian:prior-replenishment-floor`) while the quantity it bounds runs orders above it.

The second is the condition number itself, from the extreme eigenvalues. It costs a decomposition, so it is taken where a decomposition is already being paid for — at each recomputation of the covariance — and nowhere else (`alg:gaussian:condition-adaptive-recompute`). It is what the health surface reports as the model's conditioning, and it is absent until the model's first recomputation has measured one, because a condition number nobody has computed is not a condition number.

The cheap ratio is used relatively rather than absolutely. Growth of the ratio past the value recorded at the last recomputation, by a factor $\kappa_\text{growth}$ with a default of 2, calls for another one. It is not tested against a fixed threshold: the floor pins its denominator, so a fixed threshold on it is met permanently once any dimension rests at the floor, and permanently is not a trigger.

**Requirement (The bound is published as a bound)** · `req:monitoring:conditioning-bound-named`

Where both readings are published, each carries a name that says which it is, and the field promising the condition number is never filled from the ratio that bounds it. A lower bound published under the name of the quantity it bounds is a misreport a consumer cannot detect, because the two agree in shape, in units, and in the direction they move.

**Requirement (Dimensions at the prior floor)** · `req:monitoring:replenishment-floor`

At each recomputation the number of dimensions sitting at the precision floor is counted and reported (`req:gaussian:prior-replenishment-floor`). The count is the direct measure of how much of the feature space carries no evidence: a dimension at the floor has been replenished back to its prior and is contributing nothing but its prior to every estimate.

**Algorithm (Per-entity concordance)** · `alg:monitoring:per-entity-concordance`

For each entity carrying at least $N_\text{conc,min}$ labels, default five, the running fraction of its labels where the sign of the reported outcome agrees with the sign of the risk estimate. An entity whose concordance falls below the aggregate rank discrimination by more than $\delta_\text{conc}$, default $0.25$, is flagged.

The comparison is against the system's own aggregate performance rather than against a fixed threshold, which is what makes the diagnostic meaningful in a deployment whose discrimination is genuinely poor: it looks for entities that are anomalous relative to the system, not for entities the system happens to predict badly along with everything else. It is the sharpest available signal for label corruption concentrated on particular entities.

Label-time accumulation retains a bounded map keyed by entity, compares each sufficiently sampled running sign-agreement fraction to the aggregate AUC, and reports the flag, sample count and fraction. Insertion at capacity evicts the least recently observed entity and reports the lifetime eviction count; the diagnostic changes no model or decision.

**Definition (Alarm-outcome disagreement accumulators)** · `def:monitoring:alarm-outcome-cusums`

Two one-sided accumulators over the disagreement between what the Sentinels measure and what the host reports. The first accumulates where Sentinel alarm is high and the reported outcome is benign; the second where alarm is low and the reported outcome is adverse.

They are directional on purpose, because the two directions mean different things. Sustained accumulation in the first is consistent with a measurement layer firing on something the host does not consider harmful, or with adverse outcomes being reported as benign. Sustained accumulation in the second is consistent with harm the measurement layer cannot see. Either indicates a systematic disconnect between measurement and reporting, which is exactly the failure no single-layer diagnostic detects.

Both accumulators are maintained per Sentinel and per scoring axis. Label-time reporting crosses the frozen assessment-time alarm values with the eventual outcome, updates the two directional Page recursions with the configured thresholds and allowance below, and exposes the accumulated values without changing assessment or learning. Their boundaries are inclusive.

**Definition (Feature-stable outcome drift)** · `def:monitoring:feature-stable-drift`

The conjunction of two conditions: a rising prediction drift accumulator, and a feature distribution that is not moving. Together they say the outcome distribution is shifting while the inputs are not.

The conjunction is the diagnostic; neither half means much alone. It is consistent with label corruption and equally consistent with a legitimate regime change, and it cannot distinguish them (`cav:limitation:mimic-regime`) — which is why it is reported as a signature to investigate rather than as a finding. It is computed on the label path and reported, and its configured stability boundary is independent of the drift threshold.

**Definition (Quantile error concentration)** · `def:monitoring:quantile-concentration`

Partition the calibration buffer into ten quantiles of the risk estimate and report the ratio of the maximum quantile error to the mean quantile error. A ratio above three indicates miscalibration concentrated in part of the range rather than spread across it.

The distinction matters because a model can be well calibrated on average and badly calibrated exactly where a host acts. A concentrated error in the top quantile is an operational problem that an aggregate calibration figure conceals entirely. It is computed on the label path and reported.

**Caveat (What label-integrity monitoring cannot see)** · `cav:monitoring:integrity-scope`

The Core trusts labels unconditionally (`inv:guarantee:honest-uncertainty`), and the diagnostics above surface statistical signatures of corruption without ever establishing it. Four blind spots follow, and they are stated because a monitoring surface that is silent about its own scope invites the reading that silence means health.

Corruption that mimics a regime change is undetectable, because the signature is the same one a legitimate regime change produces (`cav:limitation:mimic-regime`). Patient targeted corruption that stays below the per-entity flagging thresholds is invisible by construction (`cav:limitation:patient-corruption`). A compromised investigation pipeline corrupts the sister model with no in-system mitigation whatever, because the ground-truth flag is trusted without qualification (`conv:eligibility:ground-truth`) and the Core has no means to audit it (`cav:limitation:investigation-pipeline`); this one is the host's responsibility and cannot be made otherwise. And every discrimination metric here lags the change it measures, the recent window being a faster and noisier signal rather than a solution (`cav:limitation:auc-lag`).

**Definition (Measurement maturity coverage)** · `def:monitoring:maturity-coverage`

$$\text{maturity coverage} = \frac{\sum_{c \,:\, \eta_c < \eta_\text{mature}} n_c}{\sum_c n_c}$$

The share of assessed traffic falling in cells whose measurement has matured, weighted by the traffic each cell carries. Below eighty per cent, the Core's per-Sentinel features are substantially influenced by immature measurement and should be read accordingly (`tab:keyspace:measurement-state`).

The traffic weighting is what makes the figure operational rather than descriptive: a deployment can have most of its cells immature and most of its traffic mature, and it is the second that determines whether the features being acted on are trustworthy.

Full health scans the current report indexes, applies the configured strict noise threshold to each cell, and weights the qualifying cells by their sample counts. It reports the resulting ratio as absent only when there is no assessed traffic to divide by.

**Definition (The resolution-utilisation ratio)** · `def:monitoring:resolution-utilisation`

$$\text{resolution utilisation} = \frac{\lvert \{c : n_{\text{elig},c} \geq N_\text{adequate}\} \rvert}{\lvert \mathcal{A} \rvert}$$

The fraction of cells whose spatial discrimination is backed by outcome evidence, where $N_\text{adequate}$, default twenty, is the eligible-label count at which a cell's discrimination is considered evidenced. Below fifty per cent, most of the system's spatial discrimination is not yet backed by Core-side outcome evidence.

The ratio carries a second duty beyond its own threshold. It is the instrument the detection chapter directs a host to consult before reading intervention effectiveness at all, because the divergence is uninterpretable exactly where eligible-label density is inadequate (`def:detection:divergence-scope`).

Full health scans the current Ledger entries against the configured adequacy threshold, counts equality as adequate, and divides by the same active-entry population.

**Definition (End-to-end feedback latency)** · `def:monitoring:feedback-latency`

$$T_\text{feedback} = T_\text{report} + T_\text{label} + T_\text{queue} + T_\text{publish}$$

The total time from a Sentinel's observation to that evidence being reflected in a published model, decomposed into the four stages that contribute to it. The decomposition is the point: the dominant term is almost always the human in the loop, and a deployment that responds to slow feedback by tuning its queue or its publication interval is optimising the two smallest terms.

All four stages are measured, and the first is measured as a lower bound.

$T_\text{report}$ runs from a Sentinel's observation to the report carrying it reaching the Assayer. The wire contract carries the age of a batch's oldest observation at the moment the Sentinel emitted the report, measured wholly on that Sentinel's own monotonic clock, and the report index retains it beside the arrival stamp. That age is a lower bound on the stage and never the stage itself: it stops at emission, and how long the host then held the report before handing it over is an explicitly unmeasured residual. Measuring the residual would mean comparing two machines' clocks, and the age's whole value is that it compares none. Every surface carrying the stage says at least, and the total inherits the qualification.

The three remaining stages are differences between instants on the Assayer's one injected monotonic clock, so each is a property of the deployment rather than of how busy the machine was. $T_\text{label}$ runs from that report's reception to the host's label arriving at the public label boundary — the human in the loop, normally the whole of the total. $T_\text{queue}$ runs from that arrival to the model owner taking the label up, which is the wait on the channel between two threads and is invisible unless the arrival travels with the label. $T_\text{publish}$ runs from the model owner taking it up to the new snapshot being stored.

An assessment reads several Sentinels, whose reports arrived at different moments. The stages are anchored to the earliest-arrived report that contributed an extraction, because the stage measures how long evidence waited and the longest wait among the contributors is the one the assessment carried. All four fold into one accumulator, one observation per published label, so they are smoothed over the same stream and the total is a total of one journey rather than a sum of averages taken over four different populations. A label that fails a numeric checkpoint publishes no model and contributes nothing: the journey it was on did not end. A label replayed from the journal contributes only the publish stage, its earlier instants belonging to a monotonic domain the restart ended. Full health reports the four stages, their total, and the worst report-stage lower bound across the reports the fleet currently holds; per-Sentinel health reports that Sentinel's own lower bound beside the arrival-measured report age, unsummed, because what separates the two is the unmeasured residual.

**Definition (Per-Sentinel weight mass)** · `def:monitoring:encoding-effectiveness`

For Sentinel $s$, let $\mathcal{S}_s$ be the feature positions in its own slot and let $w_j$ be the corresponding published operational model weights. Its weight mass is the mean absolute slot weight

$$I_s = \frac{1}{\lvert \mathcal{S}_s \rvert} \sum_{j \in \mathcal{S}_s} \lvert w_j \rvert.$$

The quantity is how much weight the model carries on the Sentinel's published features (`def:extraction:slot`), and that is the whole of what it says. It is not a reading of whether the outcome depends on those features, and the two must not be read off each other. Ridge weights on standardised coordinates that tell the outcome nothing do not settle at zero — they settle at noise scale — and a model whose fit leaves no residual has nothing left with which to shrink them further, so the figure a structureless Sentinel earns *rises* towards a working one's as the model converges. Driven against a perfectly predictive control on one labelled stream, a Sentinel fed a cryptographic hash of the request index read between $0.76$ and $1.05$ of the control's figure through nine thousand six hundred labels and $0.80$ at nineteen thousand two hundred; a Sentinel that had found nothing to split on at all settled near $0.30$ and stopped declining. A block carrying no weight is a genuine finding and this reading shows it. A block carrying weight is not thereby a block the outcome depends on.

What the outcome depends on is read at (`def:monitoring:slot-association`), and what removing the block would cost is read at (`def:monitoring:slot-contribution`). Both are taken over the same block as this one and neither is a function of the weights alone, which is what puts them out of reach of the convergence artefact above. The reading is computed from each Sentinel's own slot and reported per Sentinel on the health surface, beside those two.

An earlier design made a three-component heuristic the principal quantity:

$$E_s = \frac{1}{3}\Big[\underbrace{(1 - k_s / \text{cap}_s)}_{\text{rank headroom}} + \underbrace{\min(1, \text{coord conc}_s / \xi_\text{coord})}_{\text{coordination activity}} + \underbrace{(1 - P_s / \lvert \mathcal{A}_s \rvert)^+}_{\text{plateau efficiency}}\Big].$$

There $\xi_\text{coord}$ was a proposed coordination scale, and values below $0.1$ after several hundred batch reports were to indicate an encoding indistinguishable from random (`cav:limitation:silent-encoding`). That formula, its scale, its reading threshold, and its three-component semantics are superseded design history, not current diagnostic or configuration requirements. The legacy row is retired outright at (`entry:health:encoding-effectiveness`) rather than held open against an approximation of it, and the mean-slot-weight successor above is the public encoding-transparency reading in its place. A host with access to the Sentinel's own wavelet portrait still has a stronger structural signal: phase transitions with large baselines should fall at boundaries that mean something in the host's coordinate encoding, and repeated investment at boundaries with no domain interpretation says more than the aggregate diagnostic does.

**Definition (Per-dimension weight mass)** · `def:monitoring:dimension-informativeness`

Write $I(\mathcal{P})$ for the mean absolute published operational model weight over a non-empty set of feature positions $\mathcal{P}$:

$$I(\mathcal{P}) = \frac{1}{\lvert \mathcal{P} \rvert} \sum_{j \in \mathcal{P}} \lvert w_j \rvert.$$

The per-Sentinel reading above is $I_s = I(\mathcal{S}_s)$. For registered identity dimension $d$, let $\mathcal{D}_d$ be the $8 + 3m$ positions of that dimension's own block (`tab:keyspace:dimension-features`). Its weight mass is the same functional restricted to that block:

$$I_d = I(\mathcal{D}_d).$$

The reading carries the same meaning here as it does over a slot and inherits the same limit: it says how much weight the model holds on the dimension's own block and nothing about whether the outcome depends on that block. The two were driven apart directly. Two worlds were run whose traffic differed in exactly one thing — whether the entity a request arrived under was drawn from the half of the pool its outcome named, or from the whole pool by a hash of the request index — and the null dimension's figure sat between $67\%$ and $143\%$ of the keyed one's across the schedule, crossing it in both directions. A dimension's own association and contribution readings are (`def:monitoring:slot-association`) and (`def:monitoring:slot-contribution`) restricted to $\mathcal{D}_d$, exactly as this one is $I$ restricted to it.

Nothing is estimated or accumulated here that was not already published. The restriction is a grouping of weights the operational model already carries, taken at the block extents the published layout already names (`schema:dimension:map-record`), so a rebuild that moves a dimension's block moves the reading with it and no second copy of the layout can go stale against the first.

The functional has exactly one composition law, and it is worth stating because it is weaker than it looks. For disjoint $\mathcal{P}$ and $\mathcal{Q}$,

$$I(\mathcal{P} \cup \mathcal{Q}) = \frac{\lvert \mathcal{P} \rvert\, I(\mathcal{P}) + \lvert \mathcal{Q} \rvert\, I(\mathcal{Q})}{\lvert \mathcal{P} \rvert + \lvert \mathcal{Q} \rvert},$$

which follows from the sum over the union being the sum of the sums. So the per-dimension readings compose, weighted by position count, into the reading over the whole per-dimension family — and where exactly one dimension is registered the two coincide. What does not follow is any relation between a dimension's reading and a Sentinel's. The layout gives every block type a disjoint extent (`def:feature:vector-structure`), so $\mathcal{S}_s \cap \mathcal{D}_d = \emptyset$ for every Sentinel and every dimension: $I_s$ is not an average of per-dimension readings, and $I_d$ is not a share of a Sentinel's. The two are siblings under one recipe, asking the same question of two disjoint populations of positions, and neither refines the other.

The law is a property of averaging over positions and not of this reading in particular, so it is worth saying which of the three block readings inherit it and which does not. The association reading (`def:monitoring:slot-association`) does, over the same count weights and for the same reason: it too is a mean over the positions of a block. The contribution reading (`def:monitoring:slot-contribution`) does not, and no aggregate of it over blocks is published anywhere. What it measures is the cost of removing one block from a score every block contributes to, which is a joint quantity: two blocks carrying the same information each cost nothing to remove alone and cost the whole of it to remove together, so the count-weighted mean of their separate readings is not the reading over their union and there is no weighting that would make it one.

Three families of position are outside $\mathcal{D}_d$, and each is excluded for its own reason rather than by convention. The cross-dimension block aggregates across dimensions by maximum (`tab:keyspace:cross-dimension-features`), so it belongs to no single dimension; attributing it to each would count one set of weights once per registered dimension. The dimension's competitive indicators (`def:dimension:competitive-range`) are excluded because their count turns over with every restructuring, so folding them in would make the reading track set churn rather than encoding quality, where the block above is fixed-width for the life of the dimension; and because the host duty reads them separately, beside this reading rather than inside it (`req:keyspace:host-duties`), which a reading that had already absorbed them could not let it do — the separate reading being the cell weight mass tabulated with the identity-health metrics (`tab:monitoring:dimension-health`), the same fold over that other population. Interaction features are excluded on the same ground the per-Sentinel reading excludes them: an interaction position carries the weight of a product and is attributable to the operand pair, not to either operand alone (`sec:feature:interactions`). Each reading is over the block the layout gives its own entity, and over nothing it merely feeds.

Full health computes $I_d$ from the same published operational model the per-Sentinel reading is taken from and reports it per registered identity dimension, beside the per-Sentinel figure and under the same absence discipline: a dimension whose block the published map does not carry reports absence, while a dimension whose block carries no weight reports zero, because zero is the reading the host duty acts on. The producer path is held by (`test:crate:dimension-informativeness-reads-the-operational-dimension-block`), the separation of a weighted block from a silent one and the composition law by (`test:crate:dimension-informativeness-separates-a-weighted-block-from-a-silent-one`), and the export's treatment of absence by (`test:crate:dimension-informativeness-export-carries-measurement-and-absence`).

**Definition (Slot–outcome association)** · `def:monitoring:slot-association`

Let $\mathcal{B}$ be one entity's own block of feature positions — a Sentinel's slot $\mathcal{S}_s$ or an identity dimension's block $\mathcal{D}_d$. For each labelled request write $\hat\varphi$ for the standardised feature vector it was scored on and $y \in \{-1, +1\}$ for the sign of its outcome. Over a weighted window of such requests, let $r_j$ be the sample correlation between coordinate $j$ and the outcome, and let $\mathcal{V} \subseteq \mathcal{B}$ be the coordinates whose values varied over the window. The block's association is the mean absolute correlation over those coordinates, expressed as a multiple of the correlation a coordinate telling the outcome nothing would earn from a window of that length:

$$A(\mathcal{B}) = \frac{1}{\lvert \mathcal{V} \rvert} \sum_{j \in \mathcal{V}} \lvert r_j \rvert \Big/ \sqrt{\frac{2}{\pi n_\text{eff}}}, \qquad n_\text{eff} = \frac{(\sum_k v_k)^2}{\sum_k v_k^2}.$$

The divisor is the whole of what makes the reading legible. A coordinate that says nothing about the outcome still earns a sample correlation: over a window of $n$ its correlation is approximately normal about zero with standard deviation $1/\sqrt{n}$, so the expectation of its absolute value is $\sqrt{2/\pi n}$. That figure depends on the window and on nothing else — not on the block's width, not on the model, not on what the block is a block of — which is what lets one calibration serve every block on the surface and lets two blocks of different widths and different ages be read against each other. A block reading near $1$ is a block earning what noise earns.

The weights $v_k$ are the balancing weights the operational update carries (`def:weighting:balancing-weights`), decayed at that update's own combined per-label and time factor, so the stretch of traffic the reading describes is the stretch the operational weights describe. The effective sample size is the window those decayed weights amount to rather than a count of labels: under a constant stream at forgetting rate $\gamma$ it settles at $(1 + \gamma)/(1 - \gamma)$, and it is what the floor above is a function of. Neither reading is published until it reaches thirty, because below that the floor is an asymptotic statement about a quantity that has not yet earned it.

Coordinates that never varied leave the mean rather than entering it as zeros. A block is a fixed extent whose positions are occupied as its entity's traffic occupies them, and averaging a zero in for a position that held one value all window would report a mostly-idle block as quieter than its live coordinates are — which is the reading saying the block is uninformative when what is true is that most of it is unused.

The stored evidence is bounded by the block and not by the stream: three decayed moment vectors of the block's own width — the first moment of each coordinate, its second, and its cross moment with the outcome — and three scalars, being the decayed weight sum, the decayed sum of squared weights, and the decayed weighted outcome sum. The outcome's second moment is not among them because the outcome is a sign and its square is its weight. Summed over every block of a layout this is under three vectors of the full width; at the reference dimension it is some fifteen kilobytes. Nothing of the labelled stream itself is retained.

Composition follows the count-weighted law the weight-mass reading has (`def:monitoring:dimension-informativeness`), with the count of varying coordinates as the weight, because the reading is a mean over a block's positions exactly as that one is. Where two blocks are read over windows of the same effective length the floor is common to both and the calibrated multiples compose directly; where the windows differ the raw means compose and the calibration does not, which is the ordinary caution that two figures divided by different denominators do not average.

The alert band is three to fifteen times the floor. The band's lower edge is the side that matters and it does not move with the window: a null block earns the floor at every length, by construction. The upper side does move, because a genuinely carrying block's raw correlation is a property of its traffic while the floor shrinks as $1/\sqrt{n_\text{eff}}$, so a longer window lifts a carrying block's multiple and widens the margin rather than narrowing it. A deployment that shortens the window must re-read the upper edge before relying on it.

The producers accumulate on the model owner's thread at label time, before the label reaches any model, and the reduced reading rides the published health summary per Sentinel and per registered identity dimension under the same absence discipline the weight-mass reading keeps: a block that has not yet carried a window's worth of labels reports absence, and a block that has and earns the floor reports the floor's multiple, because that is the reading a host acts on.

Two conditions make a dimension readable. Its encoding must place entity identity where the competitive mechanism can partition it, because the block is a fold over the cells that mechanism finds: an encoding placing identity below the depth cutoff gives every entity one prefix at every depth the cutoff allows, and the block then describes one undivided population however the traffic is drawn. At least one outcome axis must also be registered, because a dimension's outcome history is its per-axis positions (`tab:keyspace:dimension-features`) and a block carrying none of them has no position an outcome is ever written to. Neither is a threshold to tune. Each is a way for a dimension to be unreadable that reads, on this surface, exactly as a dimension telling the outcome nothing — which is worth a host's knowing before it revises an encoding the reading was never able to see.

**Definition (Slot contribution)** · `def:monitoring:slot-contribution`

Over the same block $\mathcal{B}$ and the same weighted window, write $\rho = \hat\varphi \cdot \mu$ for the score the operational model gave a labelled request, $\rho_\mathcal{B} = \sum_{j \in \mathcal{B}} \hat\varphi_j \mu_j$ for the part of that score the block supplied, and $\ell(\rho, y)$ for the binary cross-entropy of the logistic read of a score against an outcome. The block's contribution is what removing it from the score would have cost, as a fraction of the loss the score carried:

$$C(\mathcal{B}) = \frac{\sum_k v_k \big[\ell(\rho_k - \rho_{\mathcal{B},k},\, y_k) - \ell(\rho_k, y_k)\big]}{\sum_k v_k\, \ell(\rho_k, y_k)}.$$

This is the removal cost itself and not a proxy for it, which is the whole reason it is the reading a retirement decision hangs on: the number a host reads is the number the deregistration would produce. The model regresses a sign rather than a probability, so the logistic is the monotone read that turns a score into one; it is applied to the full score and the ablated score alike, so it moves both by the same map and cannot manufacture a difference between them. A negative reading is a reading and not a defect: it says the block's coordinates were making the score worse.

The evidence is one decayed excess-loss accumulator per block and two scalars — the decayed loss the score carried and the decayed weight — which is three numbers per Sentinel and three per registered dimension. Both quantities are folded at label time from the score the request was actually given. That timing is not a convenience: after the rank-one updates the weights that produced the score are gone, and an ablation taken against the updated model would be an ablation of a score no request received.

The reading does not compose across blocks and no aggregate of it is published. The reason is stated with the composition law it declines to inherit (`def:monitoring:dimension-informativeness`): removal cost is joint, and two blocks that duplicate each other each cost nothing to remove alone.

The alert band is $0.10$ to $0.58$ of the score's loss. A converging model concentrates its reliance rather than spreading it, so a carrying block's contribution strengthens with convergence while a structureless block's tends towards zero.

The producers ride the same accumulation site as the association reading, and the reading is published per Sentinel and per registered identity dimension under the same absence discipline and the same evidence floor. A host uses contribution as a confirming reading per Sentinel and as context rather than a discrimination threshold per dimension. Both of its terms scale with the operational weights that formed the score, so a label count alone does not fix its convergence state; the association reading is invariant to that scale because it is a correlation divided by a floor depending on the window alone.

**Definition (The immature-cell count)** · `def:monitoring:immature-cells`

$$\text{immature cells} = \sum_s \big\lvert \{c \in \mathcal{A}_s : n_{\text{elig},c} < N_\text{material}\} \rvert$$

The fleet-wide count of Ledger cells holding fewer eligible labels than the materiality threshold, default one hundred (`data:ledger:attenuation`). It is the practical form of the materiality result: the count of cells whose Ledger features are attenuated towards uninformativeness because the evidence behind them is too thin to be material.

Full health scans every current Ledger entry against the strict threshold and reports both per-Sentinel and fleet-wide counts (`entry:memory:immature-count`). The materiality threshold has a second consumer, the per-assessment flag on the alarm summary, which applies this same strict comparison to the single entry that answered one request (`def:runtime:alarm-summary`). The comparison is shared; the two readings are not the same reading, and the two differences are worth stating so that neither is read off the other. The population differs: this count ranges over every entry the Ledger holds, the flag over the one entry the extraction routing reached. The predicate differs too, because the flag is a disjunction where this count is a single test — the flag raises on the attenuation condition as well, which this count does not take and which the fleet surface reports separately and traffic-weighted rather than counted (`def:monitoring:ledger-value`). So a deployment cannot recover how often the flag raises from this count, and a cell counted here is a cell the flag would raise on but not conversely.

**Definition (Ledger value realisation)** · `def:monitoring:ledger-value`

$$\text{value realised} = \frac{\sum_s \sum_{c \,:\, \bar{b}_c > \bar{\mu}_b + 0.5\sqrt{\bar{v}_b}} n_c}{\sum_s \sum_c n_c}$$

The share of assessed traffic falling in Ledger cells whose remembered adverse rate stands meaningfully above the fleet mean — that is, the share of traffic for which the Ledger is actually discriminating rather than merely present. The boundary is the theorem's own: a standardised departure of more than half a deviation from the standardisation mean (`thm:ledger:materiality`), and the moments are the ones held for that Sentinel's own bad-rate feature, each Sentinel occupying a distinct standardisation position (`def:extraction:slot`).

Reported alongside it, the attenuation-limited fraction: the share for which the Ledger would discriminate but for insufficient evidence. A cell counts towards it when it is not realising its value, its true adverse rate does clear the same boundary, and the attenuation implied by its own eligible arrival rate leaves the attenuated rate at or below that boundary (`data:ledger:attenuation`). The two conditions are read off separate stored evidence — the raw undecayed adverse and eligible counts for the first, the arrival window for the second (`tab:ledger:entry-state`) — because the theorem holds that neither suffices alone, and an attenuated average cannot stand in for both. A cell whose true rate is ordinary is therefore not attenuation-limited however sparse its labels, and a cell whose arrivals are dense enough to preserve its excess is not attenuation-limited however high its true rate.

The two shares are disjoint by construction, a realising cell being excluded from the attenuation-limited count before its evidence is consulted, so their sum stays inside the whole.

| Value realised | Attenuation-limited | Reading |
| --- | --- | --- |
| $> 60\%$ | $< 20\%$ | High traffic; the Ledger provides broad spatial discrimination |
| $20$–$60\%$ | $20$–$50\%$ | Mixed; discrimination in hotspots, fallback elsewhere |
| $< 20\%$ | $> 50\%$ | Low traffic; Ledger features negligible, detection via measurement and identity (`tab:ledger:low-maturity-detection`) |

The pair is diagnostic where either alone is not: a low realised value with a low attenuation-limited fraction means the Ledger has nothing to say, while a low realised value with a high attenuation-limited fraction means it would have something to say given evidence, and those two call for opposite responses.

Full health scans every current Ledger entry under the read guard, weights it by its own assessed traffic, and reports the pair per Sentinel and fleet-wide (`entry:memory:materiality`).

**Table (Per-dimension identity health)** · `tab:monitoring:dimension-health`

| Metric | What it reveals |
| --- | --- |
| Competitive set size | Whether the dimension is resolving into cells at all |
| Competitive set change rate | Churn, which resets per-cell measurement state |
| Per-cell indicator weight mass | What weight the model carries on the dimension's cells |
| Competitive cell depth distribution | Whether resolution is uniform or concentrated |
| Coverage fraction | The share of traffic falling in competitive cells |
| Active indicator count distribution | How many indicators fire per assessment |

Reported per registered identity dimension (`def:registry:identity-dimension`), these are the only view of whether a declared dimension is earning its width in the feature vector. The convergence snapshot carries the set's size and its change rate; the current competitive index produces the depth distribution; and the assessment path records how many indicators it actually routed through, which full health publishes as the lifetime distribution and folds into the coverage fraction.

The coverage fraction is read off that same tally rather than counted a second time. The tally already separates the two populations the row asks about — its zero key counts exactly the assessments that matched no cell, every other key one that matched at least one — so the share is the complement of the zero key's weight. Two counters incremented at one site can come to disagree; one counter read two ways cannot, and the shape row and the coverage row are then consistent about the same traffic by construction. The share is over the dimension's whole life, matching the tally it comes from: windowing is already represented here by the churn reading, which is smoothed precisely because set membership turns over. Absence is before any traffic has been assessed against the dimension, and zero is traffic that arrived and matched nothing — opposite findings for a host, one saying the set catches nothing and the other that there is nothing yet to catch.

The weight-mass row is the block reading's fold taken over a different population of positions: the ones the layout gives the dimension's competitive indicators, one per cell (`def:dimension:competitive-range`). It says what weight the model carries there and, like its sibling, nothing about whether the outcome depends on it. It is absent where the layout names no cell position for the dimension — a set that has not formed has nothing to fold over — and zero where the positions it names carry no weight.

The dimension's weight mass (`def:monitoring:dimension-informativeness`) is reported on the same surface and is deliberately not a seventh row here, and so are the dimension's two legibility readings (`def:monitoring:slot-association`), (`def:monitoring:slot-contribution`). Every metric above is a reading of the competitive set — its size, its churn, its shape, the traffic it catches, the weight the model puts on its cells — while those three are readings taken over the dimension's own block of the feature vector, which is why they are defined with the other monitoring quantities rather than tabulated with these. The two weight-mass readings are therefore siblings across the boundary rather than one figure reported twice: the block is fixed-width for the life of the dimension and the cell positions turn over with every restructuring, so a dimension can carry weight in the one and none in the other, which is exactly the case reading them separately is meant to catch.

Neither of them answers whether the model is using the dimension, and the table does not claim they do. Weight mass is what the model carries, which a block predicting nothing earns about as readily as one that predicts (`def:monitoring:dimension-informativeness`); use is what the association and contribution readings measure. This table and the block readings are therefore read together: the duty that sends a host here sends it to those as well, and it is the association reading among them that the duty's first term names (`req:keyspace:host-duties`). The association reading is the trigger, and the two beside it are context a host weighs rather than figures it acts on alone.

**Table (The observability metrics summary)** · `tab:monitoring:observability-summary`

| Metric | Scope | When meaningful | Alert threshold |
| --- | --- | --- | --- |
| Maturity coverage | System-wide | Once the warm-up pipeline has processed the top cells | $< 80\%$ |
| Resolution utilisation | System-wide | After roughly $N_\text{adequate}$ labels per cell | $< 50\%$ |
| Feedback latency | System-wide | Always | Report stage above a tenth of the total |
| Per-Sentinel weight mass | Per-Sentinel | After the operational weights have learned | No ruled alert threshold, and none is available: the reading does not separate a carrying block from a structureless one |
| Per-dimension weight mass | Per identity dimension | After the operational weights have learned | No ruled alert threshold, on the same ground; read with the dimension's cell weights (`req:keyspace:host-duties`) |
| Slot–outcome association | Per-Sentinel and per identity dimension | Once the block has carried a window of labels | Below three times the null floor, sustained; the same edge applies at both scopes |
| Slot contribution | Per-Sentinel and per identity dimension | Once the block has carried a window of labels | Below $0.10$ of the score's loss, confirming a sustained association alert per Sentinel; contextual per dimension (`def:monitoring:slot-contribution`) |
| Immature Ledger cells | System-wide | After initial warm-up | Deployment-dependent |
| Ledger value realisation | System-wide | After roughly $N_\text{adequate}$ labels per cell | $< 20\%$ in high traffic |
| Uncertainty inflation | System-wide | After the first calibration refit | $> 3$ in steady state |

The table is the operator's index into the cross-layer metrics. The feedback-latency row reads against the total, and the total is a lower bound: the alert compares the report stage against a total that is itself at least what it says, so a deployment crossing the threshold has certainly crossed it and one sitting below it may not have. The per-Sentinel weight-mass row names the canonical reading (`def:monitoring:encoding-effectiveness`).

The per-assessment Ledger immaturity predicate is separate from this table's Ledger rows. The evidence its second arm needs is stored on every entry (`tab:ledger:entry-state`), the attenuation floor it compares against is configured (`def:config:attenuation-materiality-floor`), and the assessment path reads the entry the flag is a statement about (`def:runtime:alarm-summary`). The flag is per assessment and this table is cross-layer, so nothing here aggregates it.

**Scenario (Restriction-induced spatial atrophy)** · `scenario:monitoring:atrophy`

The host restricts a region; traffic there falls; the measurement features that described it normalise because there is nothing left to measure. The risk estimate for that region drifts towards the prior, and a naive reading concludes the region became safe.

Recovery is bounded by the convergence cascade (`bound:resource:convergence-budget`) plus the Ledger's time-indexed decay at a twenty-nine-day half-life (`def:ledger:time-decay`). Throughout, coverage is continuous rather than absent: ancestor routing supplies features from the surviving parent entry, and the measurement-only anchor keeps the direction right while the sister is starved (`prop:risk:anchor-measurement-only`).

**Scenario (Source displacement)** · `scenario:monitoring:displacement`

A source moves, and the competitive set rebalances as importance shifts from the cells it left to the cells it entered (`alg:keyspace:cell-entry`). The old cells lose their standing and the new ones acquire it (`alg:keyspace:cell-exit`). Between the move and the rebalance there is a detection gap, bounded by the same cascade as the atrophy scenario, and ancestor routing again provides continuous coverage across it. The scenario differs from atrophy in cause and not in shape: the system is briefly describing a distribution that has moved.

**Scenario (The restriction-induced starvation loop)** · `scenario:monitoring:starvation`

The loop that closes: a cell is flagged adverse, the host restricts it, the restriction stops eligible labels arriving from it, and the absence of contradicting evidence leaves the flag standing. The cell's reputation sustains itself on the consequences of its own reputation (`alg:valence:contamination-loop`).

Time-indexed Ledger decay breaks it regardless of label flow, because it acts on elapsed time rather than on evidence and therefore acts precisely when no evidence is arriving (`tab:ledger:loop-timescales`). Host investigation accelerates the resolution rather than waiting for it (`conv:eligibility:ground-truth`), and the starvation-relief guidance names the cells where investigation would pay most (`def:guidance:starvation`). The Ledger's own starvation score is the one mechanism here that acts rather than waits (`def:ledger:starvation-score`).

This scenario is also the structural template for the Companion's policy-dependence loop (`cav:limitation:challenge-policy-loop`): better targeting shifts the challenged population, the tracker re-converges over its decay horizon (`def:companion:challenge-decay`), and the shifted estimate feeds back into targeting. That loop is slower and damped, but it has this shape — a host-mediated loop bounded by a decay timescale rather than eliminated.

**Remark (What an operator should take from the scenarios)** · `rem:monitoring:operator-message`

The loop is stable under all three scenarios and recovery is bounded in each. The system provides continuous coverage throughout every recovery period — at degraded resolution, through ancestor routing and the measurement-only anchor, rather than at zero coverage — and that distinction is the one an operator should carry away, because the instinct during a recovery is to intervene as though the system had gone blind.

The three scenarios share a structure worth naming. Each is a loop that passes through the host, and each is bounded by a decay timescale rather than by anything the Core decides (`inv:guarantee:feed-forward`). None of them is eliminated by the design; all of them are made finite by it, and the difference between a bounded loop and an eliminated one is what the health report exists to keep visible (`chap:spec:output-structures`). The parameters governing every threshold in this chapter are tabulated with the configuration surface (`tab:config:monitoring`).

## Part (Reference) · `part:spec:reference`

Parts I through VII say what the system is, how it computes, how it runs and what follows from it. Part VIII is where a reader looks something up. Three chapters: the configuration surface, in tables; the output structures, which are defined elsewhere and gathered here as pointers; and the document's two registers of record, the formal properties it guarantees and the limitations it discloses.

A reference Part earns its keep by being checkable. Every number below is stated once, in the table that owns it, and the chapters that spend it cite the table rather than repeating the value — which is the only arrangement under which two copies of a constant cannot drift apart.

### Chapter (Configuration) · `chap:spec:configuration`

Sixteen tables and roughly ninety parameters: every value a host may set, and every default the system assumes when the host sets nothing. The chapter fixes no behaviour of its own. Each table is the configuration face of an environment specified earlier, and the constraint column is part of the specification — a value outside it is a configuration error, not a tuning choice.

The chapter's purpose is that its numbers are right. Every row names a constant or an exposed field, and a mechanical projection checks those relationships rather than relying on a hand count.

**Convention (Citing a parameter)** · `conv:config:parameter-citation`

A parameter is a row, and a row is not an environment. Nothing in this document cites a parameter directly; a chapter that fixes a parameter's meaning cites the table that carries its value, and the table cites back to the environments whose parameters it tabulates.

The two directions do different work and both are needed. Outward from the table, the citation says what a number means: a reader who finds a rate of $0.9998$ follows the table's citation to the environment that explains what is being forgotten and how fast. Inward from a chapter, the citation says where the number lives, so that a chapter discussing a bound never restates the bound's value and can never disagree with it. What no citation does is name a row, which is why a parameter's identity is always the pair of its table and its symbol.

#### Core configuration · `sec:config:core`

Thirteen tables covering the Core's own surface, ordered as the models that consume them appear in the earlier Parts. The Derivation Function's configuration, the optional rendering layer's, and the Companion's follow the division, because none of the three is Core state.

**Table (Core risk model parameters)** · `tab:config:risk-model`

| Parameter | Default | Constraint | Reference |
| --- | --- | --- | --- |
| $\gamma_\text{opr}$ (operational forgetting) | 0.9995 | $(0, 1)$ | (`tab:risk:forgetting-rates`) |
| $\gamma_\text{inh}$ (sister and anchor forgetting) | 0.9998 | $(\gamma_\text{opr}, 1)$ | (`tab:risk:forgetting-rates`) |
| $\lambda_\text{prior}$ (prior precision) | 0.1 | $> 0$ | (`def:risk:model-triple`) |
| $\lambda_\text{floor}$ (prior replenishment) | 0.001 | $> 0$, $\leq \lambda_\text{prior}$ | (`req:gaussian:prior-replenishment-floor`) |
| $c$ (leverage safety factor) | 5 | $\geq 1$ | (`alg:update:sherman-morrison`) |
| $N_\text{recompute}$ (Cholesky recomputation interval) | 1,000 | $\geq 100$ | (`alg:gaussian:condition-adaptive-recompute`) |
| $\kappa_\text{growth}$ (conditioning-growth trigger) | 2 | $> 1$ | (`alg:gaussian:condition-adaptive-recompute`) |
| $\varepsilon_\text{Schur}$ (Schur regularisation) | $10^{-4}$ | $> 0$ | (`alg:gaussian:regularised-schur`) |
| $\kappa_\text{posture}$ (host posture ceiling on a removed block) | $10^8$ | $> 1$ | (`alg:gaussian:regularised-schur`) |
| $\gamma_\kappa$ (feature compression rate) | 0.9998 | $(0, 1)$ | (`def:risk:compression-scale`) |
| $w_\text{ceiling}$ (importance weight ceiling) | 100 | $\geq 1$ | (`def:weighting:balancing-weights`) |
| $P_{+,0}$ (initial positive-valence rate) | 0.5 | $(0, 1)$ | (`def:weighting:initial-value`) |

The two class-rate trackers are not configured here. Their decay rates are the model forgetting rates above — the global tracker takes the operational rate and the eligible tracker the inherited one — because a tracker that forgot at a different speed from the model it weights would balance the model against a population the model no longer sees (`tab:weighting:trackers`).

The two Schur parameters are worth one qualification: the relative epsilon is tabulated here, and the absolute term is formed where the marginalisation runs against the prior precision of the model being marginalised — which for an outcome axis is the precision that axis's own registration set rather than the prior tabulated above. Both parameters reach every general and rank-one marginalisation call site. A factor or ceiling outside the domains tabulated here is refused at construction rather than carried into the arithmetic.

The conditioning-growth trigger is a multiple rather than a threshold, and the change of kind matters more than the figure. It is compared against the cheap diagonal ratio the last recomputation recorded, not against the ratio itself, because the replenishment floor pins that ratio's denominator and a fixed threshold on it is therefore met permanently as soon as one dimension rests at the floor — permanently being no trigger at all (`alg:gaussian:condition-adaptive-recompute`). The default of two is chosen for the rate it implies rather than for the level it names: a recomputation records the ratio it has just measured, so the trigger fires at most once per doubling of that ratio, which is logarithmic in how far the matrix has grown rather than proportional to the labels absorbed. A smaller multiple would fire on the ordinary movement of the diagonal under decay and replenishment; a much larger one would let the ratio climb through an order of magnitude before the counter's own interval elapsed, which leaves the trigger doing nothing the counter was not going to do anyway.

The spectral floor the posterior maintains is not tabulated, because a deployment does not set it. It is derived at each recomputation from the largest eigenvalue measured there and the model's width, as the least value for which a floating-point factorisation of the matrix is assured (`dec:posterior:spectral-floor`), and it is applied there too — where the covariance is being rebuilt from the same matrix, so that the pair stays consistent. A figure a host cannot move belongs at the decision that derives it, on the same reasoning the lower condition guard is stated at its own algorithm.

The posture ceiling is the upper of the two condition guards and the only one tabulated. The lower guard is a package-side arithmetic limit rather than a parameter a deployment sets, and it is stated at the algorithm that owns it (`alg:gaussian:regularised-schur`); a table of host parameters is the wrong place for a figure a host cannot move.

**Table (The eligibility policy)** · `tab:config:eligibility`

| Parameter | Default | Constraint | Reference |
| --- | --- | --- | --- |
| Challenge failure counts as unconfounded | true | boolean | (`req:host:eligibility-policy`) |

One switch, set at construction, deciding whether a failed Challenge that was not independently investigated is admitted as unconfounded evidence for the sister and anchor models. The default admits it, on the reasoning that a challenge failure is informative about the entity rather than about the challenge, and a host whose challenge population is selected by the very estimate it feeds should set the switch the other way.

The predicate follows the selected row for Challenge while preserving the ground-truth override and the fixed eligibility of the other actions.

**Table (Outcome axis per-axis defaults)** · `tab:config:axis`

| Parameter | Default | Constraint | Reference |
| --- | --- | --- | --- |
| $\gamma_a$ (axis forgetting) | $\gamma_\text{inh}$ | $(0, 1)$ | (`def:axis:training-target`) |
| $\kappa_{a,0}$ (initial compression scale) | 1.0 | $> 0$ | (`alg:registry:axis-registration`) |
| Training eligibility mode | Eligible only | eligible only, or all labels | (`def:registry:outcome-axis`) |
| Spatial features | Enabled | enabled or disabled | (`disc:registry:spatial-policy`) |

Every value is a per-axis default supplied at registration, so two axes on one deployment may differ in all four. The forgetting rate inherits the sister and anchor rate rather than the operational one because an axis model learns from the same slow-moving evidence the inherited models do.

The two enumerations are the ones that cost: the eligibility mode decides which question the axis answers, and the spatial policy decides whether the axis adds per-cell state to every identity dimension.

**Table (Temporal parameters)** · `tab:config:temporal`

| Parameter | Default | Constraint | Reference |
| --- | --- | --- | --- |
| $\gamma_{t,\text{core}}$ (core model time decay) | 0.9999 per hour | $(0, 1)$ | (`tab:temporal:decay-inventory`) |
| $\gamma_{t,L}$ (Ledger time decay) | 0.999 per hour | $(0, 1)$ | (`def:ledger:time-decay`) |
| $\gamma_{t,\text{id}}$ (identity cell time decay) | 0.998 per hour | $(0, 1)$ | (`tab:keyspace:decay-rates`) |

Three rates, each indexed by elapsed time rather than by label count, and ordered by how fast the thing they govern should be allowed to be forgotten. The Core's own models decay slowest because their parameters are the deployment's accumulated knowledge; the identity layer decays fastest because a competitive landscape reshapes itself in days.

The identity layer's per-label rates are a separate mechanism and are not host-settable, so a host tuning temporal behaviour tunes these three and nothing else.

**Table (Identity layer parameters)** · `tab:config:identity`

| Parameter | Default | Constraint | Reference |
| --- | --- | --- | --- |
| $L_d$ (competitive depth cutoff) | 3 | $\geq 1$ | (`alg:keyspace:observation-protocol`) |
| $\lambda_\text{id}$ (outcome EWMA rate) | 0.95 | $(0, 1)$ | (`tab:keyspace:outcome-state`) |
| $\lambda_m$ (measurement EWMA rate) | 0.95 | $(0, 1)$ | (`tab:keyspace:measurement-state`) |
| Signal cache capacity | 100,000 | $\geq 1$ | (`tab:keyspace:signal-cache`) |
| Graph budget per dimension | Host-supplied | $> 0$ | (`def:registry:identity-dimension`) |
| Split threshold per dimension | Host-supplied | $> 0$ | (`def:registry:identity-dimension`) |

The table is mixed by construction and says so in its own rows: the last two are declared per dimension at registration and have no system-wide default, and the depth cutoff is likewise supplied rather than defaulted. The measurement rate sets how long a competitive cell takes to be trusted — about twenty assessments per cell per dimension at the value above — which is the figure the maturity diagnostics are read against.

The two host-supplied rows have no default, which is what the table already claims of them.

**Table (Ledger parameters)** · `tab:config:ledger`

| Parameter | Default | Constraint | Reference |
| --- | --- | --- | --- |
| $\lambda_L$ (Ledger EWMA rate) | 0.999 | $(0, 1)$ | (`tab:ledger:entry-state`) |
| $\gamma_{t,L}$ (time-indexed decay) | 0.999 per hour | $(0, 1)$ | (`def:ledger:time-decay`) |
| $N_\text{absent}$ (consecutive-absence deletion threshold) | 3 | $[1, 255]$ | (`alg:ledger:entry-deletion`) |
| $N_\text{ledger}$ (recent eligible label window) | 200 | $\geq 10$ | (`def:ledger:starvation-score`) |
| Collection floor on the entry average | $10^{-6}$ | $> 0$ | (`alg:ledger:garbage-collection`) |
| Collection horizon | 60 days | $> 0$ | (`alg:ledger:garbage-collection`) |

The time-indexed rate is the same parameter as the Ledger row of the temporal table and is repeated here because a reader sizing the Ledger needs it beside the label-indexed rate it competes with. The starvation window is a count of recent eligible labels and is a different parameter from the resolution-adequacy threshold in the monitoring table, which is also a label count and is permanently separate from it. The absence threshold's upper end is where the count that carries it ends: consecutive absences are counted in eight bits, so a threshold above 255 is one the count can never reach, and a deletion rule that can never fire is not a slower rule but an absent one.


**Table (Calibration parameters)** · `tab:config:calibration`

| Parameter | Default | Constraint | Reference |
| --- | --- | --- | --- |
| $N_\text{cal,buf}$ (buffer capacity) | 2,000 | $\geq 100$ | (`constr:platt:buffer`) |
| $N_\text{refit}$ (periodic refit cadence) | 200 | $\geq 50$ | (`tab:platt:refit-cadence`) |
| $N_\text{cal,min}$ (minimum records per regime) | 30 | $\geq 10$ | (`req:platt:minimum-samples`) |
| $n_\text{cal,pos}$ (minimum positive outcomes) | 3 | $\geq 1$ | (`req:platt:minimum-samples`) |
| $n_\text{cal,neg}$ (minimum negative outcomes) | 3 | $\geq 1$ | (`req:platt:minimum-samples`) |
| $\kappa_0$ (initial calibration parameter) | 1.0 | $> 0$ | (`def:platt:initial-value`) |
| $\kappa_\text{min}$ (search lower bound) | 0.01 | $> 0$ | (`alg:platt:fitting`) |
| $\kappa_\text{max}$ (search upper bound) | 100 | $> \kappa_\text{min}$ | (`alg:platt:fitting`) |
| $\gamma_\text{cal}$ (recency weighting) | $\gamma_\text{inh}$ | $(0, 1)$ | (`def:platt:objective`) |
| $s_\kappa$ (soft transition steepness) | 20 | $> 0$ | (`def:platt:regimes`) |
| $\delta_\text{cal}$ (drift-reset significance) | 0.1 | $> 0$ | (`alg:platt:drift-integration`) |

Eleven parameters over three concerns: what the buffer holds, when a refit is attempted and whether it is allowed to proceed, and how the fitted parameter is searched for and blended across the two regimes. The three sample floors are the ones a host is most likely to want lower and should not: a calibration fitted below them is a curve through noise, and a host that lowers them buys calibrated output that is not calibrated.

The transition steepness is a constant of the numerics rather than a field of the calibration configuration and is therefore not settable.

**Table (Standardisation parameters)** · `tab:config:standardisation`

| Parameter | Default | Constraint | Reference |
| --- | --- | --- | --- |
| $\gamma_\text{std}$ (standardisation EWMA rate) | 0.9998 | $(0, 1)$ | (`alg:standardisation:label-time-procedure`) |
| $v_\text{floor}$ (variance floor) | $10^{-4}$ | $> 0$ | (`alg:standardisation:label-time-procedure`) |
| $n_\text{std}$ (feature clip width) | 10 | $> 0$ | (`alg:standardisation:label-time-procedure`) |
| $N_\text{init}$ (cold prior-mass ramp horizon) | 100 | $\geq 10$ | (`alg:standardisation:batch-initialisation`) |
| $N_\text{boot}$ (per-Sentinel bootstrap sample count) | 100 | $\geq 10$ | (`alg:standardisation:sentinel-bootstrap`) |
| $\alpha_\text{boot}$ (bootstrap blend factor) | 0.8 | $(0, 1]$ | (`alg:standardisation:sentinel-bootstrap`) |

The clip width and the variance floor are the two that bound the damage a pathological feature can do: the floor stops a constant feature from dividing by nothing, and the clip stops an extreme value from dominating an update. The bootstrap pair governs how quickly a newly registered Sentinel's features become comparable with the rest.

The two per-Sentinel bootstrap fields are a sample count and a blend factor, and the mechanism whose name they carry reads them as such.

The horizon names the accepted-observation span over which the class priors' standardisation mass falls to zero, one share per accepted observation, rather than a count after which those priors are replaced in one publication.

**Table (Concurrency parameters)** · `tab:config:concurrency`

| Parameter | Default | Constraint | Reference |
| --- | --- | --- | --- |
| Label queue capacity | 10,000 | $\geq 100$ | (`req:publication:label-queue`) |
| Label queue overflow policy | Refuse newest and count | — | (`req:publication:label-queue`) |
| Publication interval, in labels per publish | 1 | $\geq 1$ | (`req:publication:interval`) |
| Identity dimension lock granularity | Per dimension | — | (`tab:publication:tiers`) |
| Deferred identity queue capacity | Bounded, drop oldest | $\geq 1$ | (`alg:publication:identity-draining`) |

Two of the five are structural rather than numeric and are tabulated because a host reading this table needs to know they are not choices: the lock granularity is a property of the design, and the overflow policy follows from the non-blocking discipline, since a queue that refused work would push back on a caller that must not be pushed back on.

The publication interval sets how many labels the model owner may absorb before publishing the next snapshot. The queue capacity and both bounded structures follow the policies tabulated above.

**Definition (Reference peak request rate)** · `def:config:reference-peak-request-rate-100-per-second`

The reference peak request rate is 100 requests per second — the $R$ of the pending-buffer capacity computation. The value is part of this label's name, so revising it is a re-mint that visits every citation.

**Definition (Reference label latency)** · `def:config:reference-label-latency-3600-seconds`

The reference median labelling latency is 3,600 seconds, one hour — the $L$ of the same computation. The same name-carries-value rule applies: the value is part of this label's name, so revising it is a re-mint that visits every citation.

**Table (Pending buffer parameters)** · `tab:config:pending-buffer`

| Parameter | Default | Constraint | Reference |
| --- | --- | --- | --- |
| $R$ (expected peak request rate) | 100 per second | $> 0$ | (`def:config:reference-peak-request-rate-100-per-second`) |
| $L$ (expected median label latency) | 3,600 s | $> 0$ | (`def:config:reference-label-latency-3600-seconds`) |
| Buffer capacity | Computed as $2 \times R \times L$ | $\geq 1$ | (`req:runtime:buffer-capacity`) |
| Expiry horizon | 24 hours | $> 0$ | (`req:runtime:buffer-capacity`) |
| Feature storage precision | Single | single or double | (`def:runtime:storage-precision`) |

The capacity is stated as a computation rather than as a number because the buffer is the system's dominant memory consumer at any serious request rate: it is twice the request rate multiplied by the expected label latency, so a fixed default is wasteful at low rates and silently lossy at high ones. The horizon is the second half of the same sizing argument — an entry older than the horizon is one whose label is not coming.

The capacity is $2 \times R \times L$ from the reference peak request rate (`def:config:reference-peak-request-rate-100-per-second`) and the reference median labelling latency (`def:config:reference-label-latency-3600-seconds`), and the horizon is twenty-four hours.

**Table (Extraction parameters)** · `tab:config:extraction`

| Parameter | Default | Constraint | Reference |
| --- | --- | --- | --- |
| $D_\text{chain}$ (chain length normalisation) | 16 | $\geq 1$ | (`tab:extraction:chain-structure`) |

One parameter, and it is the divisor that turns a raw ancestor-chain depth into a feature on a comparable scale. It is the maximum chain depth the encoding is expected to produce, so setting it is a statement about the host's key space rather than a tuning knob.

The parameter rescales two features every Sentinel slot carries — the normalised chain length of the chain-structure block, and the normalised peak context depth of the coordination block (`tab:extraction:coordination`). The chain-length feature is logarithmic in depth before it is divided, while the peak context depth is divided directly.

**Table (Label guidance parameters)** · `tab:config:guidance`

| Parameter | Default | Constraint | Reference |
| --- | --- | --- | --- |
| Default scan limit | 10,000 | $\geq 100$ | (`sig:guidance:interface`) |
| Starvation score threshold | 0.01 | $\geq 0$ | (`def:guidance:starvation`) |

The scan limit is what bounds guidance's cost. Guidance reads stored assessments and never recomputes a model, so its work is linear in this limit and independent of the feature dimension, and a host that raises it pays in scan time and in nothing else. The starvation threshold is the score above which a cell is offered as a candidate for relief.


**Table (Health monitoring parameters)** · `tab:config:monitoring`

| Parameter | Default | Constraint | Reference |
| --- | --- | --- | --- |
| $\kappa_\text{drift}$ (drift noise allowance) | 0.1 | $\geq 0$ | (`alg:monitoring:drift-cusums`) |
| $h$ (accumulator threshold) | 10 | $> 0$ | (`alg:monitoring:drift-cusums`) |
| $N_\text{auc,min}$ (minimum class count) | 20 | $\geq 5$ | (`alg:monitoring:auc`) |
| $N_\text{recent}$ (recent discrimination window) | 500 | $\geq 50$ | (`alg:monitoring:auc`) |
| $\epsilon_\text{sync,thresh}$ (synchronisation threshold) | $10^{-6} \cdot p$ | $> 0$ | (`def:config:synchronisation-threshold`) |
| $N_\text{conc,min}$ (minimum labels for concordance) | 5 | $\geq 3$ | (`def:config:concordance-minimum-labels`) |
| $\delta_\text{conc}$ (concordance deficit threshold) | 0.25 | $(0, 0.5)$ | (`def:config:concordance-deficit-threshold`) |
| $\theta_\text{alarm}$ (strong alarm threshold) | 3.0 | $> 0$ | (`def:config:strong-alarm-threshold`) |
| $\theta_\text{quiet}$ (no-alarm threshold) | 0.5 | $\geq 0$ | (`def:config:no-alarm-threshold`) |
| $\kappa_\text{lab}$ (alarm-outcome noise allowance) | 0.02 | $\geq 0$ | (`def:config:alarm-outcome-noise-allowance`) |
| $\theta_\text{stability}$ (feature stability threshold) | 0.001 | $> 0$ | (`def:config:feature-stability-threshold`) |
| $\eta_\text{mature}$ (maturity threshold) | 0.1 | $(0, 1)$ | (`def:config:maturity-threshold`) |
| $N_\text{adequate}$ (resolution adequacy threshold) | 20 | $\geq 1$ | (`def:config:resolution-adequacy-threshold`) |
| $\gamma_\text{latency}$ (feedback latency EWMA rate) | 0.99 | $(0, 1)$ | (`def:config:feedback-latency-ewma-rate`) |
| $N_\text{material}$ (Ledger materiality threshold) | 100 | $\geq 1$ | (`def:config:ledger-materiality-threshold`) |
| $A_\text{min}$ (attenuation materiality floor) | 0.25 | $(0, 1)$ | (`def:config:attenuation-materiality-floor`) |

Sixteen thresholds over five concerns — drift, discrimination, numerical precision, label integrity, and cross-layer observability — and they are gathered into one table because they govern diagnostics rather than model meaning. Fifteen are reading thresholds on reported quantities. The synchronisation threshold is the one operational exception: crossing it halves the recomputation interval under the adaptive-cadence rule while leaving the posterior update itself unchanged (`dec:posterior:adaptive-cadence`).

The monitoring configuration carries all sixteen rows, defaults them to the tabulated values, and rejects values outside their tabulated domains at construction.

**Definition (Synchronisation threshold)** · `def:config:synchronisation-threshold`

The synchronisation threshold is $\epsilon_\text{sync,thresh} = 10^{-6} \cdot p$. It governs when the reported synchronisation error calls for the covariance to be recomputed (`def:monitoring:synchronisation-error`). The comparison is against the residual — the reading less the part of it the replenishment clamp put there — because that is the part a recomputation can remove. The same threshold governs the other component of the reading, the part the replenishment clamp put there: a contribution over it calls for the recomputation that is the only operation which absorbs it, though it does not shorten the interval (`dec:posterior:adaptive-cadence`). A third figure is published and governs nothing — the resolution the reading carries, derived from the width and the operand magnitudes rather than set, which says how much of the residual the arithmetic could have manufactured. Both health reports carry the Frobenius-norm diagnostic, the prior-induced component, the residual, the resolution and its flag, and the comparison reads the configured per-dimension coefficient. Equality leaves the cadence unchanged and the smallest represented crossing calls for a recomputation, which shortens the interval where it is adopted.

**Definition (Minimum labels for concordance)** · `def:config:concordance-minimum-labels`

The per-entity concordance gate is $N_\text{conc,min} = 5$ labels. Below it an entity's agreement between reported outcomes and risk estimates is not read (`alg:monitoring:per-entity-concordance`). Equality reaches the gate.

**Definition (Concordance deficit threshold)** · `def:config:concordance-deficit-threshold`

The concordance deficit threshold is $\delta_\text{conc} = 0.25$. It governs when an entity's concordance falls far enough below aggregate discrimination to be flagged (`alg:monitoring:per-entity-concordance`). Equality is not flagged; a threshold immediately below the same gap flags it.

**Definition (Strong alarm threshold)** · `def:config:strong-alarm-threshold`

The strong alarm threshold is $\theta_\text{alarm} = 3.0$. It supplies the high-alarm side of the alarm-outcome disagreement accumulators (`def:monitoring:alarm-outcome-cusums`). The configured boundary is inclusive while a value immediately outside it stays quiet.

**Definition (No-alarm threshold)** · `def:config:no-alarm-threshold`

The no-alarm threshold is $\theta_\text{quiet} = 0.5$. It supplies the low-alarm side of the alarm-outcome disagreement accumulators (`def:monitoring:alarm-outcome-cusums`). The configured boundary is inclusive while a value immediately outside it stays quiet.

**Definition (Alarm-outcome noise allowance)** · `def:config:alarm-outcome-noise-allowance`

The alarm-outcome noise allowance is $\kappa_\text{lab} = 0.02$. It is exactly the disagreement either alarm-outcome accumulator forgives before growing (`def:monitoring:alarm-outcome-cusums`).

**Definition (Feature stability threshold)** · `def:config:feature-stability-threshold`

The feature stability threshold is $\theta_\text{stability} = 0.001$. It is the feature-distribution half of the feature-stable outcome-drift conjunction (`def:monitoring:feature-stable-drift`). It is independent of the drift accumulator threshold: equality keeps the flag down and a crossing raises it.

**Definition (Maturity threshold)** · `def:config:maturity-threshold`

The maturity threshold is $\eta_\text{mature} = 0.1$. A cell strictly below it contributes its traffic to measurement-maturity coverage (`def:monitoring:maturity-coverage`).

**Definition (Resolution adequacy threshold)** · `def:config:resolution-adequacy-threshold`

The resolution adequacy threshold is $N_\text{adequate} = 20$ eligible labels. It is the per-cell evidence gate in resolution utilisation (`def:monitoring:resolution-utilisation`). Equality is adequate and the entry immediately below it is not.

**Definition (Feedback latency EWMA rate)** · `def:config:feedback-latency-ewma-rate`

The feedback-latency EWMA rate is $\gamma_\text{latency} = 0.99$. It is the weight kept on the existing reading when a new one arrives, and it smooths all four stages of the end-to-end latency — report, label, queue and publication (`def:monitoring:feedback-latency`). One observation is folded per published label, on the model owner's thread, so the rate sets a horizon in labels rather than in time: at this default a stage's reading reflects roughly the last hundred labels, which is what makes it a diagnostic of the deployment rather than of its most recent label.

A stage's first observation is taken as its reading outright rather than smoothed from zero. At a rate this close to one an accumulator started at zero would spend hundreds of labels climbing out of a figure nothing measured, and the early reading is exactly when an operator is watching.

**Definition (Ledger materiality threshold)** · `def:config:ledger-materiality-threshold`

The Ledger materiality threshold is $N_\text{material} = 100$ eligible labels. It governs the fleet-wide immature-cell count and one arm of the per-assessment immaturity predicate (`def:monitoring:immature-cells`). Full health reads the strict threshold and reports the count (`entry:memory:immature-count`). The per-assessment predicate reads the same value and applies the same strict comparison to the one entry the extraction routing reached (`def:runtime:alarm-summary`), so a deployment that moves this number moves both readings together and cannot move one alone.

**Definition (Attenuation materiality floor)** · `def:config:attenuation-materiality-floor`

The attenuation materiality floor is $A_\text{min} = 0.25$. It is the other arm of the per-assessment Ledger immaturity predicate (`def:runtime:alarm-summary`). The attenuation it is compared against is computable at every entry from the stored arrival window (`tab:ledger:entry-state`). The domain is the open unit interval, which is what lets the empty-window reading be settled once and for all: an entry whose window has decayed to nothing takes the attenuation formula's limit of zero, and zero lies below every floor this definition admits.

**Beyond the Core's own tables.** Three surfaces remain, and none of them is Core state: the policy the Derivation Function is called with, the display constants of the optional rendering layer, and the Companion's own configuration.

**Table (Derivation function configuration)** · `tab:config:derivation`

| Parameter | Default | Constraint | Reference |
| --- | --- | --- | --- |
| $R_\text{pass}$ (opportunity cost of denial) | 1.0 | $> 0$ | (`tab:channel:reward-parameters`) |
| $R_\text{friction}$ (cost of challenging a good source) | 0.3 | $> 0$ | (`tab:channel:reward-parameters`) |
| $R_\text{missed}$ (cost of missing an adverse outcome) | 3.0 | $> 0$ | (`tab:channel:reward-parameters`) |
| $R_\text{caught}$ (reward for identifying an adverse source) | 2.0 | $> 0$ | (`tab:channel:reward-parameters`) |
| $R_\text{blocked}$ (cost of blocking a good source) | 1.5 | $> 0$ | (`tab:channel:reward-parameters`) |
| $\alpha_s$ (throttle severity fraction) | 0.2 | $(0, 1)$ | (`def:landscape:extended-rewards`) |
| $\beta_c$ (block catches fraction) | 1.0 | $[0, 1]$ | (`def:landscape:extended-rewards`) |
| $\beta_b$ (adverse-class sensitivity) | 1.5 | $> 0$ | (`def:channel:sensitivity-exponents`) |
| $\beta_g$ (benign-class sensitivity) | 1.0 | $> 0$ | (`def:channel:sensitivity-exponents`) |
| Neutral zone half-width | 0.1 | $[0, 0.5)$ | (`def:channel:neutral-zone`) |

The Derivation Function takes no configuration of its own beyond this policy, and the policy is supplied per call rather than registered, so one assessment may be derived under several policies at once. Where the declared action set omits an action, the parameters that affect only that action are accepted and ignored rather than rejected, because a host narrowing its action set should not have to prune its cost declaration to match.

The reward and sensitivity fields feed the derivation; the neutral-zone field is accepted and deliberately unread, as its own environment specifies. The display constants the landscape excludes are tabulated separately, immediately below.

**Table (Rendering configuration)** · `tab:config:rendering`

| Parameter | Default | Constraint | Reference |
| --- | --- | --- | --- |
| $c_Q$ (bandwidth scale factor) | 1.0 | $> 0$ | (`def:rendering:bandwidths`) |
| $c_{\text{susp},Q}$ (Suspicious bandwidth) | 0.5 | $> 0$ | (`def:rendering:bandwidths`) |
| $Q_\text{floor}$ (display bandwidth floor) | 0.3 | $> 0$ | (`def:rendering:bandwidths`) |
| $c_A$ (uncertainty attenuation coefficient) | 10.0 | $\geq 0$ | (`def:rendering:magnitudes`) |
| $c_\ell$ (classification location compression) | 0.5 | $> 0$ | (`def:rendering:magnitudes`) |
| $c_\text{susp}$ (Suspicious magnitude coefficient) | 0.5 | $> 0$ | (`def:rendering:magnitudes`) |
| $\varepsilon_\text{mono}$ (dominated tag magnitude) | $10^{-6}$ | $> 0$ | (`alg:rendering:dominated-treatment`) |
| $Q_\text{min}$ (dominated tag bandwidth) | 0.01 | $> 0$ | (`alg:rendering:dominated-treatment`) |

Eight presentation constants, and the table is new: two environments of this document cite a rendering configuration table and until now the document did not contain one. The constants belong here rather than on the landscape precisely because the landscape is specified to be free of them, so gathering them into a table of their own is what keeps that separation checkable — a reader can see at a glance that no number in this table reaches any decision quantity.


**Table (Companion tracker configuration)** · `tab:config:companion`

| Parameter | Default | Constraint | Reference |
| --- | --- | --- | --- |
| $\alpha_0$ (prior failure pseudo-count) | 1.0 | $> 0$ | (`def:companion:prior`) |
| $\beta_0$ (prior pass pseudo-count) | 1.0 | $> 0$ | (`def:companion:prior`) |
| $\gamma_{q,t}$ (pseudo-count time decay) | 0.9998 per hour | $(0, 1)$ | (`alg:companion:decay`) |
| Injection ceiling, per call | 1,000 | $> 0$ | (`alg:companion:injection`) |

The prior pair is uniform by construction, which is the honest starting point for a quantity the deployment has no evidence about yet. The decay rate is the Companion's alone and is deliberately independent of every Core rate, since the Companion holds no Core state and shares no timeline with it.

### Chapter (Output Structures) · `chap:spec:output-structures`

Everything the system hands back is defined where it is produced, and this chapter names nothing new. It exists so that a reader looking for *the outputs* as a group has one place to start, and it is a set of pointers rather than a second description: a restatement that repeated the structures would be a second copy able to disagree with the first, and a specification with two accounts of its own output has no account of it.

Four structures, in the order a host meets them. The risk assessment is what an assessment call returns — a risk basis, the per-axis outcome predictions, the per-Sentinel alarm summaries, a compact health snapshot and the identifier that links the assessment to a label reported later (`schema:output:assessment`). The decision landscape is what the derivation returns from one of those assessments together with a channel policy and a challenge posterior (`sig:landscape:output`). The Companion returns two shapes over the same estimate, the posterior the derivation consumes and the scalar moments a host consumes directly (`sig:companion:posterior`). And the system health report is the Core's health surface, emitted on its own cadence rather than per assessment, and joined to the assessment stream by the snapshot version the compact snapshot carries (`schema:output:health-snapshot`). The Companion's health arrives separately, from the tracker the host owns (`tab:companion:health`); a host that wants one view of both composes them, because Companion state reaches no Core structure (`dec:contracts:companion-boundary`) and a Core report carrying a Companion member would be that reach.

Health is what a reader is most likely to come here for, because it is the only output whose fields are scattered across a whole chapter rather than gathered into one signature: the drift state, the discrimination and coverage figures, the calibration and standardisation summaries, the buffer and queue health, the precision figures, the label-integrity flags, the convergence estimate and the cross-layer observability quantities are each specified beside the diagnostic that computes them (`chap:spec:health-monitoring`), and the Companion's own fields beside the tracker that holds them (`tab:companion:health`). That scattering is deliberate — a field belongs with its computation — and this chapter does not repeat them into a second description.

The derivation has no health report and needs none. It holds no state between calls, so there is nothing about it to observe that its inputs do not already determine (`sig:landscape:derivation-function`).

### Chapter (Guarantees and Known Limitations) · `chap:spec:guarantees-and-limitations`

The document's two registers of record. The first says what the system guarantees; the second says what it cannot do. They are written as one chapter because they answer one question between them — what a host may rely on — and a reader who has only the first half of that answer has been misled by omission.

Two rules govern the chapter. **A formal property is stated once.** Most are stated here; seven are stated at the chapter that owns them, where the property is the whole content of an environment, and this chapter cites those seven rather than restating them. **A limitation is stated once, here**, and every chapter that creates an exposure cites it. There are forty-two.

**Architectural and operational properties.** Each names a property of the system's structure or behaviour.

**Invariant (Assessment writes only what is enumerated)** · `inv:guarantee:assess-only`

The Core never selects an action. An assessment reads state, computes an estimate, and writes only the enumerated set of measurement updates that its own specification lists (`inv:runtime:enumerated-writes`); no branch of it chooses, recommends or applies a disposition (`prin:principle:measure-not-decide`).

The property is the architecture's load-bearing separation rather than a convenience: it is what lets a host change its policy without retraining, and what makes every other guarantee here a guarantee about measurement rather than about behaviour.

**Invariant (The derivation is pure)** · `inv:guarantee:derivation-purity`

The derivation reads no state outside its arguments, writes no state, and is deterministic: the same inputs produce the same output on any call, in any order, on any process (`pf:landscape:purity`).

Purity is what makes the derivation cheap to call and safe to call repeatedly, and it is the premise every replay argument in this document rests on (`sig:landscape:derivation-function`).

**Invariant (Assessment preserves evidence authority)** · `inv:guarantee:evidence-authority`

An assessment advances only the bookkeeping its own specification enumerates (`inv:runtime:enumerated-writes`) and the observational measurement its two acquisition mechanisms take; it advances no state that an outcome taught. An accepted cold standardisation observation reaches only a snapshot later than the one its own request was answered from, retires at most $1/N_\text{init}$ of the ramp's base mass, and travels with the phase, the accepted count and the snapshot version that place it (`alg:standardisation:batch-initialisation`).

The property is the honest form of a promise this document once made too broadly. Repeated derivations of one held assessment are bit-identical, which is the invariant above; repeated assessments of one subject are not promised identical risk values across coordinate versions — an accepted observation is residue, deliberately, and the ramp exists in order to leave it. What is guaranteed instead is that the residue is observational. No model parameter, no calibration state, no outcome-memory value and no continuing standardisation average moves anywhere but on the label path, so a host that reads two different values for one subject knows the difference is a coordinate system it can name from the version rather than learning it cannot see. Each accepted cold observation publishes one ramp advance, and every assessment carries the phase, accepted count and version that place its coordinate state.

**Invariant (The Companion is independent of the Core)** · `inv:guarantee:companion-independence`

The Companion reads no Core state and no Core state depends on Companion state. The two meet only in the derivation, which consumes an output of each and writes back to neither (`inv:companion:boundary`).

The independence is why the Companion can be replaced wholesale by a host with its own estimator, and why a Companion failure degrades the landscape's uncertainty rather than corrupting the Core's posterior.

**Invariant (The blend operates within the anchor's subspace)** · `inv:guarantee:blend-subspace`

Anchor activation depends only on uncertainty in the anchor's own subspace, and is insensitive to Sentinel-specific and axis-specific uncertainty (`def:risk:subspace-blend`).

Without the restriction the anchor would activate whenever any part of the feature space was uncertain, which is most of the time on a growing deployment, and the system would fall back to its coarsest model exactly when its finer ones were becoming useful.

**Invariant (Ledger influence is floored, never eliminated)** · `inv:guarantee:ledger-floor`

Stale reputation decays to insignificance within a bounded interval regardless of label flow, because the Ledger's decay is indexed by elapsed time and not only by labels (`thm:temporal:ledger-decay-bound`). The bound holds under the stated condition that time-indexed decay is enabled (`def:ledger:time-decay`).

The guarantee is directed at the contamination loop rather than at freshness: a memory that decayed only on labels would hold a restricted entity's reputation fixed for as long as the restriction suppressed its labels.

**Invariant (Axis predictions never enter the derivation)** · `inv:guarantee:outcome-neutrality`

Outcome axis predictions are not inputs to the derivation. Axis features influence risk through the Core, as designed; axis *outputs* reach the host and stop there (`def:landscape:outcome-neutrality`).

The boundary is what keeps an axis a measurement rather than a second opinion. A host wanting an axis to affect a decision composes it into its own policy, where the composition is visible and reviewable, instead of having it silently enter a crossover.

**Invariant (Model drift is detected and reported)** · `inv:guarantee:drift-visibility`

Each Core model's calibration drift is measured against its own predictions and reported, so that a model degrading against the population it serves is observable without an external evaluation (`alg:monitoring:drift-cusums`).

The measurement is one-sided in each direction and accumulates, which is what lets a small persistent bias be detected long before it is visible in an aggregate error rate. The allowance the accumulators forgive before growing is tabulated with the monitoring configuration (`tab:config:monitoring`).

**Invariant (The Companion's update is conjugate)** · `inv:guarantee:conjugacy`

The Companion's update is exact conjugate arithmetic: no approximation is introduced and none accumulates across any number of updates (`alg:companion:update`).

Exactness matters here more than its cheapness. The Companion's estimate is built from a very thin stream of contributing labels, so an approximation error that accumulated would be indistinguishable from evidence and would be trusted as such.

**Invariant (A recorded assessment replays exactly)** · `inv:guarantee:replay`

Given a stored risk assessment, the channel policy it was derived under, and the challenge posterior of that moment, the landscape is reproducible exactly, without access to any model state (`ex:runtime:composition`).

This is what makes an audit trail an audit trail: the three stored values are the complete input, so a decision can be reconstructed months later on a system whose models have moved on entirely.

**Invariant (Empirical coverage is tracked and reported)** · `inv:guarantee:discrimination-visibility`

Rank discrimination, top-decile lift, per-axis correlation and the empirical coverage of the reported uncertainty are all computed and reported, so that a host can see both whether the ordering is right and whether the stated uncertainty is honest (`alg:monitoring:auc`).

The last of the four is the one the property is really about. Discrimination says the ordering is useful; coverage says the intervals mean what they claim, and the two can fail independently. All four are computed and reported. The coverage fractions and the inflation factor are formed from the calibration rows at each refit, gated at the calibration minimum per regime, and the inflation factor rides the compact snapshot besides (`alg:monitoring:empirical-coverage`).

**Lifecycle and numerical properties.** Each states a property of a dimension change or posterior computation.

**Invariant (Lifecycle structure is preserved and approximation is explicit)** · `inv:guarantee:structural-exactness`

Extension is an exact transformation of the posterior, and the unregularised Schur identity is its exact marginalisation (`thm:gaussian:extension`), (`thm:gaussian:marginalisation`). The regularised marginalisation preserves the declared partition and mean subvector while applying the correction or its explicit kept-block fallback. Its numerical posterior is therefore an approximation whenever the offset changes a non-zero correction.

The structural property is what makes a changing feature space tractable at all: the same dimensions are removed from every model and each result is adopted atomically. The approximation is separately observable, and as a computed figure rather than only as an outcome: every event reports what its regularisation retained over the exact marginal, labelled as a measurement or as a bound, and a discarded correction reports the whole of itself (`req:gaussian:marginalisation-error-reported`). The corpus still states no cumulative bound for repeated regularised or fallback marginalisations. What it now carries is a cumulative report of them, which is a monitor standing where the bound is absent rather than the bound itself.

**Invariant (Axis lifecycle preserves structure with declared marginalisation)** · `inv:guarantee:axis-lifecycle`

Registering an outcome axis is exact extension on every model it touches. Deregistering one applies the declared regularised marginalisation on every surviving model, so an axis set may be reshaped at runtime without retraining anything (`alg:axis:lifecycle`).

The guarantee is the structural preservation above applied to the one lifecycle a host is most likely to exercise repeatedly, since axes are cheap to propose and easy to retire. The registration handler's own under-extension is recorded at that algorithm rather than here (`alg:registry:axis-registration`).

**Invariant (Precision is maintained within its bounds)** · `inv:guarantee:precision`

A spectral floor bounds the posterior precision matrix's least eigenvalue from below at every recomputation, which is where the floor is measured and applied and where the factorisation that needs it is performed, so the matrix is factorisable in the working arithmetic at every point at which it is factorised (`dec:posterior:spectral-floor`). Between recomputations the forgetting scales the spectrum by a positive factor, which preserves positive definiteness exactly and costs only margin — bounded by that factor over the interval, and absorbed by the next recomputation's own measurement. The replenishment floor bounds each diagonal entry and does not bound the spectrum (`req:gaussian:prior-replenishment-floor`); the synchronisation monitor detects the drift between the tracked precision and covariance before it becomes numerically material (`alg:gaussian:synchronisation-monitor`).

The three are separate guarantees and the distinction is the whole content of the statement. The spectral floor is what makes factorisability a precondition the model maintains rather than an outcome it hopes for, which is what lets a refused factorisation be read as a verdict against the invariant rather than as a condition to repair. It is a bound on a definite matrix and not a route into the definite cone: a matrix already outside the cone is refused rather than floored, since the amount that would carry a negative eigenvalue in is a repair of the damage rather than the least value the arithmetic requires. The coordinatewise floor is a modelling statement about how much confidence a single feature may lose and bounds no eigenvalue: equal positive diagonal entries can coexist with an arbitrarily small eigenvalue through the off-diagonal structure, which the requirement it names says in its own words. The monitor catches the drift neither floor addresses.

What the statement previously claimed was that the replenishment floor bounded the condition number, which the requirement it cited refutes in its own text. The clause is not weakened here but relocated: it becomes true of a floor on the spectrum, and the floor on the diagonal keeps the narrower guarantee it always had.

**Algebraic and conditional properties.** Each is a claim a deployment must satisfy under its stated premises.

**Invariant (The per-observation update is exact)** · `inv:guarantee:per-observation-exactness`

Each update is exact Bayesian inference given the current posterior and the step's effective precision, and the composite posterior that results from a sequence of them is conservative — wider than a fixed-model posterior over the same evidence (`thm:gaussian:two-level-guarantee`).

The two halves are different kinds of claim and the second is the one that matters operationally: exactness is per step, and what a host relies on across steps is the direction of the error rather than its absence (`prop:gaussian:conservatism`).

**Invariant (The blend's variance takes the stated form)** · `inv:guarantee:blend-variance`

The blended estimate's uncertainty is the mixture variance under the blend weight, in the three-term form the derivation gives, and not the weighted average of the component variances (`prop:risk:blend-variance`).

The distinction is the whole content of the property. The missing term is the disagreement between the two models being blended, and a consumer that dropped it would report a confident estimate at exactly the moments when the system's two views of an entity disagreed most.

**Invariant (The crossover set translates rigidly)** · `inv:guarantee:rigid-translation`

Every crossover decomposes exactly into a shared term and a per-action offset, so a change in the shared term moves the whole crossover set together without altering the spacing or the order of its members (`thm:landscape:rigid-translation`).

Rigidity is what makes a landscape summarisable: a host can reason about one scalar rather than about a vector of positions. The landscape carries the shared term once and the per-crossover offsets separately.

**Invariant (Landscape uncertainty has exactly two sources)** · `inv:guarantee:two-source-uncertainty`

All uncertainty in the landscape is attributable, per crossover, to the risk estimate's variance or to the challenge posterior's, and to nothing else (`thm:landscape:crossover-covariance`).

The exhaustiveness is the useful part: a host reading a wide crossover interval can always decompose it into the two, and therefore always knows whether more labels or more challenge evidence is what would narrow it (`def:fragility:definition`). The landscape's shared risk uncertainty and per-crossover sensitivities carry the two contributions.

**Invariant (Dominance is reported and never enforced)** · `inv:guarantee:dominance`

Dominance is reported as a probability in closed form, and the landscape's shape does not depend on it: the crossover vector keeps one entry per adjacent action pair whether or not an action is dominated (`thm:landscape:dominance`).

Reporting rather than pruning is what keeps the landscape's structure fixed across calls, so a host comparing two assessments compares like with like. Any suppression of a dominated action is a display decision taken downstream and has no counterpart here. The landscape reports one domination probability per fixed-shape regime and leaves suppression to the renderer.

**Invariant (Reported uncertainty is never narrower than the evidence)** · `inv:guarantee:honest-uncertainty`

The posterior reflects the evidence as it was labelled. The guarantee is conditional on the host's side of the contract: label correctness is the host's responsibility, and the Core trusts every label it is given (`thm:gaussian:two-level-guarantee`).

The condition is not a disclaimer but the property's actual content. A system that corrected for suspected label error would be reporting a narrower uncertainty than its evidence supports, which is the failure this property exists to forbid; the cost is that a corrupted label stream produces confident wrong answers, and the limitations below say so.

**Boundary and exploration properties.** Each states what the host may discover or vary without changing the measured structure.

**Invariant (Encoding consequences are visible to the host)** · `inv:guarantee:encoding-transparency`

Each Sentinel's coordinate semantics — what its dimensions mean, what a shared prefix asserts, and what the Core has therefore assumed about them — are recorded at registration and reported back (`schema:registry:sentinel-record`), so that a host can discover the consequences of its encoding choices without reading the Core's internals (`chap:spec:output-structures`).

Transparency here is the counterweight to the encoding contract's demands: the document asks a host to supply a hierarchical key space and owes it, in exchange, a legible account of what was made of that space.

**Invariant (Regime structure is independent of posture)** · `inv:guarantee:posture-independence`

The landscape is computed without reference to the host's posture. Posture is a cursor a host moves over a completed landscape, so two hosts reading one assessment at two postures read the same structure and differ only in where they stand on it (`def:channel:posture`).

The independence is what allows a single derivation to serve every channel and every reviewer at once, and it is why a landscape can be stored and re-read later at a posture nobody had chosen when it was computed. Derivation takes no posture, and the two posture-indexed utilities read the completed landscape afterward.

**Invariant (The system exposes an exploration signal)** · `inv:guarantee:exploration`

Exploration is a joint property of two halves. The Core supplies the estimate's uncertainty, which says where evidence is thin (`def:guidance:risk-informative`); the derivation supplies fragility, which says where that thinness would change a decision (`def:fragility:definition`). Together they identify the assessments worth learning from.

Either half alone is much weaker. Uncertainty without fragility ranks cases the system is unsure about, most of which do not matter; fragility without uncertainty ranks cases near a boundary, most of which are near it for good reason. Core guidance ranks risk-informative pending requests, and the landscape's fragility utility reports decision sensitivity at a caller's posture for host-side composition.

**The seven stated elsewhere.** Each of these is the whole content of an environment at the chapter that owns it, and repeating the statement here would create a second copy able to disagree with the first. Four keep this register's own area because the guarantee is what they are: the feed-forward invariant (`inv:guarantee:feed-forward`), non-blocking assessment (`inv:guarantee:non-blocking`), bounded staleness (`inv:guarantee:staleness`) and lifecycle publication (`inv:guarantee:lifecycle-publication`). Three take their own chapter's area because the environment there is the statement and the guarantee is its force: the two dimension-map properties (`inv:dimension:covering`), (`inv:dimension:contiguity`), and the landscape's freedom from presentation (`inv:landscape:presentation-free`). A reader wanting the whole picture reads the twenty-two above and these seven at their own sites — twenty-nine properties in all.

Three further properties are stated of the optional rendering layer and are scoped to it rather than to the primary output (`app:spec:resonance-rendering`); they constrain a display and bind nothing a host decides on.

**Figure (Information flow)** · `fig:architecture:information-flow`

```
Sentinels ──► Core ──► Host
                ▲         │
                │         ├──► derivation ──► landscape ──► Host
                │         │        ▲
   Host ──labels──┘       │        │
                          │   Companion ──posterior──┘
                          │        ▲
                          │        │
                          └──challenge results──► Companion
```

Every arrow is unidirectional. The Core never reads from the derivation or from the Companion. The derivation reads from both and writes to neither. The Companion reads challenge outcomes from the host and writes to neither of the others. The only cycle in the diagram passes through the host, which is the feed-forward invariant drawn rather than stated.

The figure is a summary and binds nothing of its own; the complete inventory of what crosses each boundary, in which direction, and what never crosses at all, is the interface appendix's (`tab:boundary:verification`).

**The limitations.** Forty-two of them, each naming something the system cannot do or cannot see, with the cause that makes it so and whatever bounds the damage. They are graded four ways — structural, managed under stated conditions, empirical or default-valued, and the host's responsibility — and the grade is part of the statement: a structural limitation is one no mechanism in this design removes, and the rest are bounded and instrumented.

Three deserve a reader's attention before the rest, because each names an exposure that a naive design leaves entirely silent rather than merely unsolved. Symmetric decay forgives an adversary on the same schedule it forgives a stale reputation (`cav:limitation:laundering`). The challenge population is chosen by the estimate the challenge evidence feeds (`cav:limitation:challenge-policy-loop`). And the honesty of reported uncertainty is measurable only by a coverage diagnostic that must actually be computed (`alg:monitoring:empirical-coverage`). A design without those three named would not be wrong; it would be quiet.

**Caveat (Outcomes are observed on one side of the action only)** · `cav:limitation:valence`

The host acts on the system's estimate and then observes only what happened under that action, so the counterfactual is never labelled. The system learns from a censored bandit (`prin:valence:censored-asymmetry`). Exploration, the investigation criterion and the measurement-only anchor bound the damage and none of them removes it (`disc:valence:mitigation-layers`).

**Caveat (The Ledger absorbs the consequences of the actions it informs)** · `cav:limitation:ledger-contamination`

Spatial outcome memory is written from labels produced under decisions the same memory shaped, so a cell's history is partly a record of how it has been treated (`alg:valence:contamination-loop`). Time-indexed decay and the measurement-only anchor bound the loop under the stated conditions (`def:ledger:time-decay`).

**Caveat (The same loop reaches spatially-enabled axes)** · `cav:limitation:axis-contamination`

An outcome axis registered with spatial features carries per-cell state written from the same censored labels, so the contamination is not confined to the risk Ledger (`disc:valence:axis-pathway`). The axis pathway is graded rather than binary, which changes the loop's shape and not its existence, and the same mitigations apply.

**Caveat (The host's policy shapes the population the Core learns from)** · `cav:limitation:host-loop`

Restriction reduces the label flow from the entities restricted, so the population the models train on is the population the host has chosen to keep serving (`scenario:monitoring:atrophy`). Time-indexed decay and ancestor routing bound the effect; three scenarios in the monitoring chapter work through what it looks like when they do not.

**Caveat (Interaction features converge slowly and unevenly)** · `cav:limitation:interaction-convergence`

An interaction feature is informative only where its two constituents co-occur, and co-occurrence rates vary by orders of magnitude across a template set, so interactions arrive in a cascade rather than together (`tab:feature:interaction-convergence`). A deployment reading its convergence as a single number will misread it.

**Caveat (The challenge estimate converges only as labels arrive)** · `cav:limitation:challenge-convergence`

The Companion's posterior narrows as the inverse of its contributing label count, and nothing accelerates that (`bound:companion:convergence`). The override and the injection interface let a host supply prior evidence, which shortens the wait without changing the rate.

**Caveat (Contributing labels are thin in realistic deployments)** · `cav:limitation:challenge-thin`

Only challenged requests whose outcome was observed contribute, and on a typical deployment that is a small fraction of one per cent of traffic (`data:companion:contributing-rate`). The posterior expresses the resulting cold start honestly, as width, which is a correct report of a real shortage rather than a fix for it.

**Caveat (The challenge population is chosen by the policy it informs)** · `cav:limitation:challenge-policy-loop`

Challenge effectiveness is a property of the challenged population, and that population is selected by the estimate the effectiveness figure feeds (`scenario:monitoring:starvation`). The loop is slow and damped rather than divergent, and an override inherits the caveat rather than escaping it.

**Caveat (Narrowing the action space narrows the evidence)** · `cav:limitation:challenge-narrow`

A channel that declares fewer actions produces fewer distinguishable outcomes, and the challenge regime in particular can be narrowed until it is never chosen (`prop:channel:challenge-width`). The dominance probability is reported per request so that the narrowing is visible rather than silent (`cav:channel:action-space-sizing`).

**Caveat (Symmetric decay is exploitable in the forgiving direction)** · `cav:limitation:laundering`

The Ledger forgets an adverse history at the same rate it forgets a stale good one, and an entity that generates benign volume can accelerate its own forgiveness (`disc:ledger:laundering`). The exposure is accepted deliberately: an asymmetric decay would make stale reputation permanent, which is the worse failure. The Ledger is one of several detection pathways and the others still fire.

**Caveat (The Ledger is structurally uninformative for low-traffic cells)** · `cav:limitation:ledger-low-traffic`

Time-indexed decay attenuates a cell's accumulated evidence below materiality whenever labels arrive more slowly than the decay removes them, which is the normal condition for most cells (`data:ledger:attenuation`). Detection in those cells runs through the immediate-convergence features instead (`tab:ledger:low-maturity-detection`).

**Caveat (A fresh entry carries no evidence and inherits none)** · `cav:limitation:fresh-entry`

A newly created Ledger entry starts from the prior on every layer, and nothing is transferred from a parent or a neighbour (`alg:ledger:entry-creation`). The interval before it says anything is long by the attenuation analysis, and it is covered by the per-Sentinel features that converge immediately.

**Caveat (The anchor floor bounds how far the blend can be pulled)** · `cav:limitation:anchor-floor`

The anchor retains a small residual influence at every blend weight, arising from the leverage differential and Schur inflation rather than from a declared minimum (`dec:risk:anchor-floor`). It provides contamination resistance, and because it is emergent its exact magnitude is a property of the deployment rather than a guarantee.

**Caveat (The models are linear in the feature vector)** · `cav:limitation:linear`

The Core's models are linear, so any structure that is not expressible as a weighted sum of features must be supplied as a feature (`mot:feature:interaction-overview`). Interaction templates give quadratic reach and no more, and a genuinely non-linear relationship is approximated rather than learned.

**Caveat (The wildcard template's feature count grows quadratically)** · `cav:limitation:interaction-type-three`

A cross-Sentinel wildcard template creates one feature per unordered pair, so its count grows as the square of the Sentinel count and its convergence takes months (`def:feature:template-wildcard`). It is off by default, which is the mitigation: a host that enables it should know it has bought a long wait.

**Caveat (Cross-Sentinel structure is learned only through interactions)** · `cav:limitation:cross-sentinel-gap`

Nothing but an interaction feature can express a pattern that is unremarkable in each Sentinel alone and meaningful across two, so joint concealment is undetectable for as long as the relevant interaction is unconverged (`tab:detection:convergence-window`). The anchor supplies partial coverage during the window.

**Caveat (The posterior can become ill-conditioned)** · `cav:limitation:conditioning`

Forgetting erodes precision in directions the data stops exercising, and a posterior whose condition number grows without bound eventually produces unreliable updates (`alg:gaussian:condition-adaptive-recompute`). The prior replenishment floor and adaptive recomputation bound it (`req:gaussian:prior-replenishment-floor`), under those stated conditions and not otherwise.

**Caveat (The importance-weight ceiling binds at extreme class rates)** · `cav:limitation:ceiling`

The static ceiling on balancing weights binds once the positive-valence rate falls below roughly half a per cent, after which rare positives are under- weighted relative to the balance the weighting exists to restore (`def:weighting:balancing-weights`). The ceiling is configurable. Health reports the binding fraction and positive gradient balance independently for the operational and eligible sister streams, accumulated from the weights each label actually used.

**Caveat (The compression scale assumes a stationary target)** · `cav:limitation:kappa-nonstationary`

An axis's compression scale adapts to the distribution of values it has seen, so the target a model was trained against shifts retroactively when the distribution moves (`def:axis:adaptive-compression`). Forgetting makes the shift self-correcting over time, and a large jump is a quantity worth watching rather than an error.

**Caveat (Axis training inherits the eligibility confound)** · `cav:limitation:axis-confounding`

An axis trained on eligible labels only sees values from the population the host's policy admitted, so the axis answers a question conditioned on that policy (`def:axis:training-target`). The eligibility mode is per axis precisely so that the choice is deliberate; neither setting removes the confound, they choose which one to carry.

**Caveat (Cross-axis structure is emergent and unmodelled)** · `cav:limitation:cross-axis`

Where two axes are registered, each model can predict the other's values only through their shared features, and the quality of that prediction depends on how often the two are co-reported (`prop:axis:cross-axis-prediction`). Nothing models the relationship directly and nothing reports how well it is holding.

**Caveat (Each axis costs features and labels)** · `cav:limitation:axis-cost`

Registering an axis adds features to every model that carries axis features and creates a model of its own, so the cost is quadratic in the feature dimension and the convergence cost is a fresh label budget (`tab:registry:axis-scaling`). The spatial feature policy is the lever, and the per-axis models parallelise.

**Caveat (Chain depth is bounded and visible in the features)** · `cav:limitation:chain-depth`

The statistical properties of every chain-derived view vary with the chain's length, so a value from a shallow chain and the same value from a deep one do not mean the same thing (`tab:extraction:chain-z-scores`). The chain-length feature is supplied alongside so that the model can condition on it (`tab:extraction:chain-structure`), which disambiguates rather than removes the dependence.

**Caveat (The maximum view carries an order-statistics bias)** · `cav:limitation:chain-maximum`

The expected maximum of independent draws grows with the number of draws, so a deep chain produces a larger maximum than a shallow one on identical data (`rem:extraction:order-statistic-bias`). The chain-length feature lets the model absorb the growth, and an explicit correction is available and not applied by default.

**Caveat (Nothing between reports is observable)** · `cav:limitation:inter-report`

The Core sees batch reports, so anything that begins and ends between two reports leaves no trace in any feature (`def:extraction:report-staleness`). Report staleness and the batch context features tell the model how much time a report is standing for, which is what makes the trade legible rather than what closes the gap.

**Caveat (The signal cache evicts, and eviction loses evidence)** · `cav:limitation:signal-cache`

Host signals are held in a bounded cache, and an entry evicted before its label arrives is simply absent from the reconstructed vector (`tab:keyspace:signal-cache`). Eviction is least-recently-used and the cache is decoupled from competitive standing, so an eviction costs a signal rather than an entity's history.

**Caveat (Competitive churn resets measurement state)** · `cav:limitation:competitive-churn`

An entity entering or leaving a competitive set triggers extension or marginalisation, and the measurement state of the affected range restarts its convergence (`alg:keyspace:cell-exit`). The operations are low-rank and infrequent — a few times a day on a typical deployment — which bounds the cost without eliminating the restart.

**Caveat (A bad encoding degrades everything and announces nothing)** · `cav:limitation:silent-encoding`

A key space supplied without real hierarchical structure produces features that carry no information, and every downstream layer degrades gracefully enough that nothing fails visibly (`rem:encoding:downstream-fallback`). The designated early warning is the association reading (`def:monitoring:slot-association`), the mean absolute correlation between the block's coordinates and the outcome read against the correlation a coordinate telling the outcome nothing earns. It is computed and reported per Sentinel and per identity dimension, and a block that has fallen to the null floor is a block whose coordinates are telling the outcome nothing over the window the reading remembers.

The role sits there rather than on the weight-mass reading (`def:monitoring:encoding-effectiveness`) because that reading moves the wrong way. Standardised-space ridge weights on coordinates that predict nothing settle at noise scale rather than at zero, and a model that has fit its stream has no residual left with which to shrink them, so a hash-fed Sentinel's mean slot weight *rises* towards a working one's as the model converges — measured, between $0.76$ and $1.05$ of a perfect control's figure through nine thousand six hundred labels. An indicator that climbs while the thing it warns about is getting worse is not a weak warning; it is the opposite of one, and a deployment reading it as an early warning would be reassured by exactly the convergence that should have alarmed it.

The warning is still a prompt to look rather than a diagnosis. An association at the floor says the model's stream carries no relationship between this block's coordinates and the outcome, and does not say whether the encoding or the phenomenon is the reason. What it now supports that it did not before is the second step: whether the block can be retired is the contribution reading (`def:monitoring:slot-contribution`), which is the removal cost itself.

**Caveat (The Core sees effects, never Sentinel-internal operations)** · `cav:limitation:abstraction`

The Core observes cells appearing and disappearing in reported sets and never the operations inside a Sentinel that produced them, so it can trace causation and cannot correspond operations (`alg:runtime:cell-set-maintenance`). The contour snapshot supplies diagnostic context for a human, not a correspondence for the model.

**Caveat (A Sentinel added late standardises against a moving target)** · `cav:limitation:late-standardisation`

A newly registered Sentinel's features enter a running standardisation whose statistics reflect the features it does not have, so its own values are mis-scaled until its statistics accumulate (`alg:standardisation:sentinel-bootstrap`). The per-Sentinel bootstrap shortens the interval substantially and does not remove it.

**Caveat (Outputs before the first refit are uncalibrated)** · `cav:limitation:pre-calibration`

The calibration parameter is unfitted until enough labelled outcomes have accumulated, so probabilities emitted before the first refit are ordered correctly and scaled arbitrarily (`rem:warmup:pre-calibration`). Pre-seeding and the reported calibration maturity are what let a host tell the two intervals apart.

**Caveat (A label arriving after eviction is lost)** · `cav:limitation:buffer-eviction`

An outcome reported for an assessment already evicted from the pending buffer cannot be applied to any model: the call fails and the evidence is gone (`req:runtime:buffer-capacity`). Sizing capacity and expiry to the deployment's actual label latency is the whole of the mitigation.

**Caveat (Discrimination metrics lag the change they measure)** · `cav:limitation:auc-lag`

Every discrimination figure is computed over a buffer spanning roughly ten days at the reference label rate, so a change in the models' ordering quality is visible only after it has been true for a while (`alg:monitoring:auc`). The recent window is the faster and noisier signal, and reading it against the aggregate is what the instability flag is for.

**Caveat (A regime change can mimic feature-stable drift)** · `cav:limitation:mimic-regime`

Outcomes moving while features hold steady is the signature of corrupted labels and also the signature of a legitimate shift in what the same behaviour means (`def:monitoring:feature-stable-drift`). The flag reports the conjunction and cannot distinguish the two causes; distinguishing them is the host's, with context the Core does not have.

**Caveat (Patient corruption is not detectable by these means)** · `cav:limitation:patient-corruption`

Label corruption held below the per-entity flagging threshold and spread across many entities passes every integrity diagnostic in this document (`cav:monitoring:integrity-scope`). Its impact is bounded by the same forgetting that bounds everything else, which limits persistence rather than providing detection.

**Caveat (Investigation requests presuppose a pipeline the Core lacks)** · `cav:limitation:investigation-pipeline`

The Core trusts the ground-truth flag unconditionally, and has no means to audit the pipeline that sets it (`conv:eligibility:ground-truth`). A compromised investigation pipeline is therefore indistinguishable from a working one, and its integrity is the host's responsibility in the full sense.

**Caveat (Severity discrimination is cold until severity labels arrive)** · `cav:limitation:severity-cold`

The binary risk target carries no severity information, so a deployment has no severity discrimination at all until an axis trained on severity has converged (`rem:host:severity-discrimination`). Registering that axis early is the mitigation, and it costs the axis's own label budget.

**Caveat (Severity enters only through the target, not the features)** · `cav:limitation:severity-path`

Because the risk target is binary, severity reaches the risk models only insofar as severe events change the features, and never as a magnitude (`def:risk:target`). The design commits to this: a dedicated outcome axis is the route by which severity becomes a first-class quantity.

**Caveat (The landscape is only as good as the declared costs)** · `cav:limitation:reward-sensitivity`

Crossover positions move substantially under small changes in the ratios between declared costs, so a landscape is a statement about the host's declared preferences rather than an objective fact (`data:channel:reward-sensitivity`). A surprising crossover position is a reason to check the declaration first (`rem:channel:verify-costs`).

**Caveat (Fragility assumes a Gaussian crossover distribution)** · `cav:limitation:fragility-gaussian`

The default flip-probability computation uses the stated Gaussian marginal approximation, which is an approximation wherever the challenge posterior is far from symmetric (`def:fragility:definition`). An evaluation exact in the challenge parameter is available for conjugate posteriors and is not the default.

**Caveat (The divergence attributes to no single cause)** · `cav:limitation:divergence-attribution`

External restriction signals are not in the feature vector, so the measured divergence between what was predicted and what occurred cannot be decomposed per feature with any reliability (`def:detection:divergence-scope`). The aggregate direction is trustworthy; the per-feature attribution is approximate and should be read as a hint.

**Caveat (Hibernation preserves self-structure only)** · `cav:limitation:hibernation`

Hibernating a Sentinel retains the parameters that concern it alone and discards every cross-term it participated in, so waking it relearns its interactions from nothing (`alg:registry:hibernation`). The archive copies the departing entity's own positions out of every full-dimension model before the marginalisation removes them, and the restore writes that block back over the positions an extension has just created, where the couplings the extension zeroed stay zero because the record carries none to overwrite them with. A deployment that asks for the hibernating disposition therefore gets the entity's own mean, precision and covariance back, aged on both clocks, and falls back to the prior wherever the archive has expired, the arrangement has changed, or a model's block has aged past the replenishment floor (`req:gaussian:prior-replenishment-floor`). What it does not get is the couplings, which return at exactly zero however strong they were, nor the outcome memory, which ends with the entity that fed it (`rem:registry:axis-hibernation`).

## Appendix (Resonance Rendering) · `app:spec:resonance-rendering`

The decision landscape is the primary, presentation-free output of the Derivation Function (`sig:landscape:output`). This appendix specifies an optional layer that presents that landscape, together with the classification belief, as a smooth spectrum of competing tags for display, audit visualisation and host interfaces. Nothing here carries decision content beyond the landscape and the risk basis: all decision semantics live on the landscape, and all exploration semantics on fragility (`def:fragility:definition`).

The subordination is deliberate and it is recorded where the alternative was rejected (`dec:landscape:rendering-optional`). The presentation-free landscape comes first; the display constants, Cauchy kernel, crossover-matching placement with its existence test, bandwidths with their floor, magnitudes and dominated-tag treatment form a separate rendering over it.

**Signature (The rendering contract)** · `sig:rendering:contract`

The rendering is a function of a landscape, a risk basis and a display configuration, and of nothing else.

```rust
fn render_resonances(
    landscape: &DecisionLandscape,
    risk: &RiskBasis,
    config: &ResonanceConfig,
) -> ResonanceProfile;

struct ResonanceProfile {
    tags: Vec<TagResonance>,
    posture_domain: (f64, f64),   // (0,1), exclusive
    config_echo: ResonanceConfig, // the profile describes its own rendering
}

struct TagResonance {
    tag: Tag,
    location: f64,  // the tag's peak posture, in (0,1)
    magnitude: f64, // peak endorsement strength, positive
    q: f64,         // bandwidth, positive
    dominated: bool,
}
```

The configuration supplies the eight display constants the primary landscape deliberately excludes, and they are tabulated once, in the reference chapter (`tab:config:rendering`). The echoed configuration is what makes a stored profile self-describing: rendering replay needs no external state, because the profile carries the constants it was rendered under.

The configuration record and its defaults agree field for field with the table that owns them, the profile carries the three fields above, and the separate rendering call takes a landscape, its risk basis and the display configuration (`inv:landscape:presentation-free`).

**Table (The tag spectrum)** · `tab:rendering:spectrum`

Two tag families are presented on one shared posture axis.

| Family | Tags | What the family expresses |
| --- | --- | --- |
| Classification | Good, Suspicious, Malicious | What the system believes the observation *is* — a display of the risk probability and its uncertainty (`def:risk:probability`) |
| Action | Allow, Challenge, Slow, Block | What the system reckons about *doing* — a display of the landscape's crossovers (`def:landscape:action-crossover`) |

Each rendered tag carries three parameters and no others.

| Parameter | Symbol | Domain | Meaning |
| --- | --- | --- | --- |
| Location | $\mu_a$ | $(0,1)$ | The posture at which the tag's relevance peaks |
| Magnitude | $A_a$ | $(0,\infty)$ | Peak endorsement strength |
| Bandwidth | $Q_a$ | $(0,\infty)$ | Certainty of the placement |

Classification belief is posture-independent, and placing Good and Malicious *on* the posture axis is a display choreography that lets the two families overlap legibly (`def:rendering:magnitudes`). It is not a claim that belief peaks at a particular caution level, and a reader who takes it for one has read a drawing convention as a result.

**Equation (The kernel and its normalisation)** · `eq:rendering:kernel`

Each tag contributes at posture $\pi$ through a Cauchy kernel on the logit scale:

$$f_a(\pi) = \frac{A_a}{1 + Q_a^2\,\bigl[\ell(\pi) - \ell(\mu_a)\bigr]^2}, \qquad \ell(x) = \ln\frac{x}{1-x}$$

At any posture the contributions normalise across all rendered tags to a distribution over them:

$$P(\text{tag}_a \mid \pi) = \frac{f_a(\pi)}{\sum_{a'} f_{a'}(\pi)}$$

The joint normalisation across the two families is a display convention and nothing more. These are normalised kernel evaluations under the display constants, not posterior probabilities of any event, and a consumer wanting probabilities within one family renormalises within that family (`sig:rendering:ambiguity-gauge`).

**Proposition (Every rendered tag is present everywhere)** · `prop:rendering:completeness`

Every tag has $f_a(\pi) > 0$ at every $\pi \in (0,1)$, so every rendered tag carries positive probability at every posture. The property follows from Cauchy tail positivity alone: the kernel's denominator is finite for finite argument and its numerator is positive by the domain of $A_a$.

The consequence a display depends on is that no posture ever has an empty or degenerate tag distribution, however extreme the evidence. The property is scoped to this layer and constrains a display; it says nothing a host may rely on about the landscape, which is why it is stated here and not in the register of guarantees.

**Proposition (The rendering is proper)** · `prop:rendering:properness`

$\sum_a P(a \mid \pi) = 1$ for every $\pi \in (0,1)$, by construction of the normalisation (`eq:rendering:kernel`). The statement is worth making because properness is the one thing the joint normalisation does buy, and it is routinely confused with the thing it does not: the rendered distribution sums to one over tags, and that fact carries no evidential claim about any tag. Like completeness, the property is scoped to the display layer.

**Proposition (No parameter is clamped to an endpoint)** · `prop:rendering:no-clamping`

Every location satisfies $\mu_a \in (0,1)$ strictly, and no magnitude or bandwidth is clamped to a boundary of its domain. Boundary-action locations are additionally constrained to $(0.01, 0.49)$ and $(0.51, 0.99)$ (`alg:rendering:placement`) so that a peak stays legible at extreme postures. That constraint is a display bound and not an endpoint clamp: it keeps a tag inside a drawable interval, and it never collapses a tag onto the axis endpoints, where the logit is undefined and the kernel would carry no information about the tag's placement.

**Algorithm (Action tag placement)** · `alg:rendering:placement`

Action tags are placed from the landscape's crossovers (`def:landscape:action-crossover`). An interior action locates at the midpoint of its two bounding crossovers on the logit scale:

$$\ell_{\mu,a_j} = \frac{\ell^*_{j-1 \to j} + \ell^*_{j \to j+1}}{2}, \qquad \mu_{a_j} = \sigma\bigl(\ell_{\mu,a_j}\bigr)$$

A boundary action has only one bounding crossover, and is placed by the **crossover-matching** condition: at the crossover $\ell^*$ between a boundary action $a$ and its adjacent interior action $b$, equal rendered contribution requires $f_a(\ell^*) = f_b(\ell^*)$, which fixes the offset

$$d = \frac{1}{Q_a}\sqrt{\rho}, \qquad \rho \equiv \frac{A_a}{A_b}\Bigl(1 + Q_b^2 w_b^2/4\Bigr) - 1$$

with $w_b$ the interior action's regime width on the logit scale. The first action locates at $\ell^*_{1 \to 2} - d$ and the last at $\ell^*_{J-1 \to J} + d$. Where a channel declares exactly two actions both are boundary tags sharing one crossover, and they are placed at $\ell^* \mp 1/Q$.

1. **Locate the interior actions** at the midpoints of their bounding crossovers.
2. **Test existence at each boundary action.** Where $\rho < 0$ no offset satisfies the matching condition, and the action is treated as a dominated display tag (`alg:rendering:dominated-treatment`).
3. **Locate the boundary actions** at their matched offsets, and constrain them to the drawable intervals (`prop:rendering:no-clamping`).

Two reductions are worth recording because they check the formula. At $A_a = A_b$ and $Q_a = Q_b$ the offset is $w_b/2$, a reflection of the interior action's half-width; at $\rho = 1$ it is $1/Q_a$, the bandwidth's own half-power point.

**Definition (Bandwidths and the display floor)** · `def:rendering:bandwidths`

Classification bandwidths derive from the risk uncertainty, and action bandwidths from the crossover variances.

$$Q_\text{Good} = Q_\text{Malicious} = \frac{c_Q\,\kappa_\text{eff}}{\sigma_\text{eff}}, \qquad Q_\text{Suspicious} = c_{\text{susp},Q}$$

$$Q_{a_j} = \frac{c_Q}{\sqrt{\operatorname{Var}(\ell^*_{j-1 \to j}) + \operatorname{Var}(\ell^*_{j \to j+1})}}, \qquad Q_{a_1} = \frac{c_Q}{\sqrt{2\operatorname{Var}(\ell^*_{1 \to 2})}}$$

The action form treats the two bounding crossovers as independent. That is a deliberate and conservative display simplification, and the exact joint structure is available where it matters (`thm:landscape:crossover-covariance`).

The classification form is an identity rather than a choice. Starting from $Q_\text{class} = c_Q \bigl/ \bigl(\sigma_{\hat{p}} / [\hat{p}(1-\hat{p})]\bigr)$ and substituting $\sigma_{\hat{p}} = \hat{p}(1-\hat{p})\,\sigma_\text{eff} / \kappa_\text{eff}$, the $\hat{p}(1-\hat{p})$ factors cancel and the bandwidth is independent of the point estimate (`def:risk:probability`). The classification compression governs where a tag is drawn, not how sharply (`def:rendering:magnitudes`).

**The display floor.** Each action bandwidth is raised to a floor,

$$Q_{a_j} \leftarrow \max\bigl(Q_{a_j},\, Q_\text{floor}\bigr), \qquad Q_\text{floor} = 0.3,$$

and the floor is falsifiable rather than decorative because the condition under which it binds is stated. It binds exactly when a crossover's total uncertainty exceeds $c_Q^2 / (2 Q_\text{floor}^2) = 5.56$. At full challenge sensitivity $\beta_c = 1$ that threshold is never reached and the floor never binds. At $\beta_c = 0$ with the Companion at cold start the Slow-to-Block crossover uncertainty is $6.48$, and the floor binds for Slow and Block until roughly twenty contributing labels have accumulated.

The floor is a display safeguard for wide-interval regimes only, and it is not a substitute for honest uncertainty: the landscape's credible intervals carry the cold-start width undiminished (`def:landscape:credible-intervals`), and a consumer wanting the width the evidence supports reads them rather than the rendered bandwidth. The display configuration carries the floor at $0.3$, applied as a maximum against the raw bandwidth.

**Definition (Magnitudes, relevance and attenuation)** · `def:rendering:magnitudes`

Each tag's magnitude is a relevance term attenuated by the risk uncertainty:

$$A_a = R_a \cdot \alpha(\sigma_{\hat{p}}), \qquad \alpha(\sigma_{\hat{p}}) = \frac{1}{1 + c_A\,\sigma^2_{\hat{p}}}$$

**Action relevance** is the width of the action's optimal regime on the posture axis, $R_{a_j} = \pi^*_{j \to j+1} - \pi^*_{j-1 \to j}$, with the outermost regimes closed at $0$ and $1$. A dominated action receives the dominated magnitude instead (`alg:rendering:dominated-treatment`).

**Classification relevance and placement** render the risk probability and its uncertainty:

$$\mu_\text{Good} = \sigma\bigl(-c_\ell\,\ell(\hat{p})\bigr), \qquad \mu_\text{Malicious} = \sigma\bigl(c_\ell\,\ell(\hat{p})\bigr), \qquad \mu_\text{Suspicious} = 0.5$$

$$R_\text{Good} = 1 - \hat{p}, \qquad R_\text{Malicious} = \hat{p}, \qquad R_\text{Suspicious} = c_\text{susp} \cdot 4\hat{p}(1-\hat{p})$$

The compression $c_\ell$ keeps the Good tag off the extreme tail at very low risk, where an uncompressed logit would drive the location past the drawable interval. Suspicious is centred at $0.5$ and peaks in magnitude near $\hat{p} = 0.5$, which is where irreducible ambiguity belongs. The neutral zone a channel declares is a policy quantity and is not this band (`def:channel:neutral-zone`): the two are drawn on one axis and mean different things, and the display is the only place they meet.

**Algorithm (Dominated-tag display treatment)** · `alg:rendering:dominated-treatment`

Three conditions mark a tag as dominated for display, and they have one effect between them.

1. **Reported domination.** The landscape reports the action's domination probability above the display threshold, which defaults to $0.5$ (`thm:landscape:dominance`).
2. **Crossover inversion.** The action's regime has zero or negative width at the point estimate, so its two bounding crossovers do not bracket it.
3. **Existence failure.** The boundary placement test fails, $\rho < 0$, and no matched offset exists (`alg:rendering:placement`).

Where any of the three holds, the tag's magnitude is set to $\varepsilon_\text{mono}$, its bandwidth to $Q_\text{min}$, and its dominated flag to true. A tag so treated has negligible rendered probability at every posture — of order $10^{-4}$ or below — though never zero, by completeness (`prop:rendering:completeness`).

The suppression is purely presentational and has no counterpart in the landscape, where dominance is always a continuous reading and is reported rather than enforced (`inv:guarantee:dominance`). That asymmetry is the point: a display must choose whether to draw a tag, and the landscape must not, because a structure that appeared and disappeared with the evidence would not be a continuous function of it. Rendering evaluates the continuous domination probability on the landscape against the display threshold and combines that first trigger with crossover inversion and existence failure before marking the tag.

**Example (The four landscapes, rendered)** · `ex:rendering:examples`

The four worked landscapes render at the default display configuration, and the renderings are stated here rather than beside the landscapes so that the presentation-free reading of each example stands on its own (`setup:landscape:worked-assumptions`).

The low-risk landscape (`ex:landscape:low-risk`), at $\hat{p} = 0.05$ and $\sigma_\text{eff} = 0.42$, renders as follows.

| Tag | Location | Magnitude | Bandwidth |
| --- | --- | --- | --- |
| Good | $0.814$ | $0.946$ | $2.38$ |
| Suspicious | $0.500$ | $0.095$ | $0.50$ |
| Malicious | $0.186$ | $0.050$ | $2.38$ |
| Allow | $0.257$ | $0.580$ | $2.48$ |
| Challenge | $0.606$ | $0.048$ | $2.48$ |
| Slow | $0.731$ | $0.181$ | $2.48$ |
| Block | $0.874$ | $0.187$ | $2.48$ |

Renormalised within the action family, the same rendering gives the following probabilities at the postures a host is likeliest to read.

| Posture | Allow | Challenge | Slow | Block |
| --- | --- | --- | --- | --- |
| Permissive, $0.10$ | $91.9\%$ | $1.6\%$ | $4.0\%$ | $2.5\%$ |
| Normal, $0.30$ | $96.5\%$ | $0.9\%$ | $1.8\%$ | $0.8\%$ |
| $0.582$, the Allow-to-Challenge crossover | $30.0\%$ | $30.2\%$ | $32.3\%$ | $7.4\%$ |
| Elevated, $0.60$ | $25.8\%$ | $30.3\%$ | $36.2\%$ | $7.7\%$ |
| Emergency, $0.90$ | $5.4\%$ | $1.5\%$ | $11.4\%$ | $81.7\%$ |

The three-action landscape (`ex:landscape:three-action`) renders Challenge as a broad tag — location $0.702$, magnitude $0.216$, bandwidth $2.48$ — reaching plurality at the Allow-to-Challenge crossover and majority by the elevated posture. That is the wider challenge regime made visible (`prop:channel:challenge-width`), and it is the clearest demonstration of what the rendering is for: a width that the landscape states as a number becomes a shape a reader can see. The medium-risk landscape (`ex:landscape:medium-risk`) and the high-risk landscape (`ex:landscape:high-risk`) render by the same construction, and the crossover-matching condition holds at every crossover of all four to the precision stated.


**Signature (Tag probabilities and the ambiguity gauge)** · `sig:rendering:ambiguity-gauge`

Two utilities read a rendered profile at a posture.

```rust
fn tag_probabilities(profile: &ResonanceProfile, posture: f64) -> HashMap<Tag, f64>;
fn profile_ambiguity(profile: &ResonanceProfile, posture: f64) -> ProfileAmbiguity;

struct ProfileAmbiguity {
    total: f64,          // entropy of the joint rendered distribution
    classification: f64, // entropy within the classification family
    action: f64,         // entropy within the action family
}
```

The first applies the kernel and normalises (`eq:rendering:kernel`). The second returns the Shannon entropy of the rendered tag distribution, decomposed by renormalising within each family.

**The gauge is not a value of information, and the disclaimer is part of the specification rather than a note about it.** Entropy of the rendered distribution weights uncertainty by nothing; a value of information weights it by decision sensitivity. The two agree only by coincidence, and they disagree exactly where a host would act on the difference — at a posture where the distribution is broad but every tag leads to the same action, the entropy is high and the information value is nil. Decision-aware exploration therefore uses fragility, which is defined on the landscape and not on this rendering (`def:fragility:definition`), and the fragility interpretation is where the two surfaces are told apart (`tab:fragility:interpretation`). The type is named for ambiguity precisely to sever the association.

## Appendix (Extraction Reference) · `app:spec:extraction-reference`

The per-Sentinel extraction is specified feature group by feature group in the extraction chapter (`setup:extraction:from-batch-report`); this appendix states the result as one index, position by position, so that a reader holding an offset can find out what sits there without reconstructing the groups. It is a reference table in the strict sense: it adds nothing to the specification, and its whole value is that it can be looked up.

**Table (The extraction index)** · `tab:extraction:reference-index`

Every index below is extraction-relative — a position within $\mathbf{g}_s \in \mathbb{R}^{60 + 2 m_s}$, not within the Sentinel's slot. To convert a row to a slot-relative offset, add one for the occupancy indicator (`conv:extraction:offsets`). The four per-axis features are named for the axes in their fixed order: novelty, drift, spread and coordination.

| Index | Group | View | Features |
| --- | --- | --- | --- |
| $0$–$3$ | Chain z-scores | Cell | The four axes |
| $4$–$7$ | Chain z-scores | Root | The four axes |
| $8$–$11$ | Chain z-scores | Maximum | The four axes |
| $12$–$15$ | Chain z-scores | Mean | The four axes |
| $16$–$19$ | Chain z-scores | Gradient | The four axes |
| $20$–$23$ | Chain z-scores | Spread | The four axes |
| $24$–$27$ | Chain CUSUMs | Cell | The four axes |
| $28$–$31$ | Chain CUSUMs | Root | The four axes |
| $32$–$35$ | Chain CUSUMs | Maximum | The four axes |
| $36$–$43$ | Chain structure | — | Rank and energy at two views each, maturity at two views, chain length, report staleness |
| $44$–$47$ | Coordination | Per-axis maximum z-score | The four axes |
| $48$–$51$ | Coordination | Per-axis maximum CUSUM | The four axes |
| $52$–$55$ | Coordination | — | Concordance, context fraction, depth, root z-score |
| $56$ | Ledger | — | Cell exponentially-weighted adverse rate |
| $57$ | Ledger | — | Compressed valence, exponentially weighted |
| $58$ | Ledger | — | Raw valence, exponentially weighted |
| $59 \ldots 58 + 2 m_s$ | Ledger, per axis | — | Compressed and raw axis means, one pair for each of the $m_s$ eligible axes |
| $59 + 2 m_s$ | Batch context | — | The logarithm of one plus the sample count |

The groups partition the range without gap or overlap, and the partition is checkable from the table alone: six z-score views at four axes each fill $0$–$23$, three CUSUM views fill $24$–$35$, chain structure fills $36$–$43$, coordination fills $44$–$55$, the Ledger's fixed features fill $56$–$58$, the per-axis pairs follow, and batch context is last. Summing the groups gives $60 + 2 m_s$, which is the width the extraction declares (`tab:extraction:chain-z-scores`).

The width formula, the dimension-map allocation and the slot conversion must agree (`def:dimension:layers`). Because every row mirrors a constant, this appendix is the document's best candidate for a mechanical check — a table that is generated from the extraction's own indices cannot drift from them, and a table transcribed by hand eventually will.

## Appendix (Dimensional Summary) · `app:spec:dimensional-summary`

The feature vector's dimension is an accounting identity over eight blocks, and this appendix states it, evaluates it at the reference configuration, and tabulates how it scales. The identity matters beyond bookkeeping: the Gaussian algebra's costs are stated in the dimension (`tab:gaussian:operation-costs`), so a reader sizing a deployment reads this appendix first and the resource chapter second.

**Equation (The dimension identity)** · `eq:dimension:total`

$$p = 1 + p_\text{agg} + (8 + 3m)D + 8\,\mathbb{1}[D > 0] + p_\text{sig} + n(q+1) + p_\text{int} + \sum_d \lvert\mathcal{E}_d\rvert$$

The terms are, in order: the bias; the Sentinel aggregate block; the identity per-dimension features over $D$ declared identity dimensions with $m$ outcome axes; the cross-dimension block, present whenever any identity dimension is declared; the host signal block; the $n$ Sentinel slots of $q+1$ positions each (`def:extraction:slot`); the interaction block; and the competitive indicators of each identity dimension (`def:keyspace:competitive-indicators`). Every term is a block of the dimension map and appears in the map in this order (`def:dimension:layers`).

**Table (Dimensions at reference, and their scaling)** · `tab:dimension:summary`

At the reference configuration (`tab:resource:reference-configuration`) the eight blocks contribute as follows.

| Block | Formula | At reference |
| --- | --- | --- |
| Bias | $1$ | $1$ |
| Sentinel aggregate | $p_\text{agg}$ | $15$ |
| Identity per-dimension | $(8 + 3m) D$ | $11 \times 2 = 22$ |
| Identity cross-dimension | $8\,\mathbb{1}[D > 0]$ | $8$ |
| Host signals | $p_\text{sig}$ | $0$ |
| Sentinel slots | $n(61 + 2 m_s)$ | $8 \times 63 = 504$ |
| Interactions | $5n + 8 + \sum_d \lvert\mathcal{E}_d\rvert$ | $40 + 8 + 20 = 68$ |
| Competitive indicators | $\sum_d \lvert\mathcal{E}_d\rvert$ | $10 + 10 = 20$ |
| **Total** | | **638** |

Holding two identity dimensions and no host signals, the dimension scales in the Sentinel count and the per-Sentinel eligible-axis count as follows.

| Sentinels | $m_s = 0$ | $m_s = 1$ | $m_s = 3$ | $m_s = 5$ |
| --- | --- | --- | --- | --- |
| $3$ | $286$ | $298$ | $322$ | $346$ |
| $8$ | $616$ | $638$ | $682$ | $726$ |
| $12$ | $880$ | $910$ | $970$ | $1{,}030$ |
| $15$ | $1{,}078$ | $1{,}114$ | $1{,}186$ | $1{,}258$ |
| $20$ | $1{,}408$ | $1{,}454$ | $1{,}546$ | $1{,}638$ |

The reference configuration is the $638$ entry, at eight Sentinels and one eligible axis apiece. The marginal cost of one further Sentinel is $66 + 2 m_s$ features, and of one further outcome axis $2n\,\mathbb{1}[\text{spatial}] + 3D$ features, which is what makes adding a source after convergence a bounded operation rather than a re-architecture (`prop:resource:incremental-addition`).

At the reference configuration, the aggregate block is fifteen, the slot width is sixty-three, eight slots give five hundred and four, and the dimension identity totals six hundred and thirty-eight. The Sentinel-slot row is the arithmetic link to the extraction reference — sixty-one is the occupancy indicator plus the sixty fixed extraction positions, and the $2 m_s$ is the per-axis Ledger pair (`tab:extraction:reference-index`).

## Appendix (Glossary) · `app:spec:glossary`

A glossary of this document is a table of terms, one-line glosses and pointers to the environments that define them. It is a navigational surface and it mints nothing: every term it would list is defined once, at its own environment, and a second statement of the same thing in a shorter form is a second thing that can disagree with the first.

This appendix therefore carries no environments of its own. Its content is produced from the document's mints rather than written beside them, which is what keeps it honest in the one way a glossary is usually not: a generated glossary names what the document actually defines, and cannot list a type that no environment introduces. That failure was real in the superseded text, where seven glossary rows named decision-landscape and rendering types that no part of the specification minted, and a hand-maintained glossary is the kind of surface where such rows survive indefinitely because nothing checks them.

## Appendix (Honest Notes Register) · `app:spec:honest-notes`

The limitations of this specification are stated once, in the register that owns them, and graded there (`chap:spec:guarantees-and-limitations`). A grouping of those same forty-two limitations by grade adds no statement to the document: it is exact restatement, and it was exact — every limitation appeared in one grade list, the four lists held sixteen, twelve, twelve and two members, and the sum was forty-two.

This appendix therefore carries no environments of its own. The grade grouping is produced from the limitation register's own rows and their grades, which is the right treatment for a table whose only content is a second arrangement of another table: an arrangement cannot drift from what it arranges when it is derived from it. The one part of this appendix that was authored rather than arranged — the naming of three limitations that deserve a reader's attention before the rest — is not a restatement and does not belong to a grouping; it is stated where the limitations are, in that register's own preamble, and the coverage diagnostic is one of the three (`alg:monitoring:empirical-coverage`).

## Appendix (Interface Map) · `app:spec:interface-map`

This appendix is the boundary verification. Every other statement in the document about what crosses a boundary is local to the chapter that makes it; this is the one place the crossings are enumerated together, and it is normative on that account — the feed-forward invariant either has a complete inventory behind it or it does not (`inv:guarantee:feed-forward`).

Three interfaces meet in this system: the one between a Sentinel and the Core, the one between a Sentinel and the graph it observes into, and the one between the Core and the host. The second of these was missing from the superseded text, which described itself as a complete interface map while enumerating two of its three interfaces; it is restored below, because an inventory that omits a boundary cannot be checked against the invariant it exists to support.

**Table (What the Core consumes)** · `tab:boundary:consumed`

The Core's relationship with each Sentinel is mediated entirely through periodic batch reports, received at one entry point (`alg:runtime:report-reception`). The batch report is the complete interface: there is no other call, query or subscription, and a record that does not appear in a report does not reach the Core at all.

| Record the Sentinel publishes | Where the Core consumes it |
| --- | --- |
| Cell analysis report, competitive and ancestor | The chain z-score features (`tab:extraction:chain-z-scores`) |
| Scoring records across the four axes | The chain z-scores and the chain CUSUMs (`tab:extraction:chain-cusums`) |
| Maturity record | The chain structure features (`tab:extraction:chain-structure`) |
| Scoring geometry record | The chain structure features, as structural reliability (`tab:extraction:chain-structure`) |
| Coordination report | The coordination features (`tab:extraction:coordination`) |
| Contour snapshot | Diagnostic only — cell-set maintenance and the health snapshot (`schema:output:health-snapshot`) |
| Analysis set summary | The chain structure features (`tab:extraction:chain-structure`) |

A record consumed here that the reception section does not read would be an interface this appendix invented, so the two inventories must agree.

**Table (What the Core does not consume)** · `tab:boundary:not-consumed`

Three published records are not read, and the reason in each case is a statement about the Core's design rather than about the record's quality.

| Record the Sentinel publishes | Why the Core does not read it |
| --- | --- |
| Per-sample scores | The Core reads batch summaries; per-request scoring would make the Core's cost linear in the observation stream rather than in the report |
| Inline health report | Sentinel-internal diagnostics, which the Core neither interprets nor relays |
| Wavelet portrait | The formation history of a spatial partition, not its current alarm state; it is a direct Sentinel-to-host diagnostic and reaches the host without passing through the Core |

Non-consumption is not a judgment of value. The portrait in particular explains how a Sentinel's partition came to be, which is exactly the question the Core has no machinery to ask and no reason to: the Core consumes alarm state, and formation history is the host's to read.

The Sentinel corpus also publishes a contour query interface — point query, range query, iteration, plateau count — at its output chapter's contour-queries section, which the Core does not use because it maintains its own spatial index over the reported cells.

**Table (The graph-to-Sentinel interface)** · `tab:boundary:graph-interface`

A Sentinel observes into a dual-tree value-stratified index, and this is the third of the system's interfaces. The Core participates in it in one direction only and in one capacity, and the pointers below are into the two upstream corpora rather than into this document, in the qualified form the front matter fixes (`conv:spec:upstream-references`).

| What crosses | Which corpus states it |
| --- | --- |
| The capabilities a Sentinel uses of the graph | The capability section of the spectral Sentinel specification, which enumerates the operations a Sentinel invokes |
| The mathematical properties the Sentinel assumes of the graph | The properties appendix of the dual-tree value-stratified index specification, assumed by the Sentinel at its stated-assumptions section |
| Observations, from the Sentinel to the graph | Unit-volume observation at a coordinate; the graph owns everything downstream of the call |
| Anything at all, from the graph to the Sentinel | Nothing: the graph has no knowledge of any Sentinel's existence |

The row that matters to a reader of *this* document is the last. A Sentinel's graph is not the Core's graph, and the Core has no interface to it in either direction (`tab:boundary:verification`). The Core does operate graphs of the same kind, one for each declared identity dimension, and those it owns, sizes and observes into itself (`def:registry:identity-dimension`) — a structural relationship, and not the observational one it has with every Sentinel.

**Table (The Sentinel symbol cross-reference)** · `tab:boundary:symbols`

A translation aid, and nothing more: the upstream corpus and this document name the same quantities differently, and a reader moving between them needs the correspondence written down once.

| Symbol | What the Sentinel corpus means by it | Where this document uses it |
| --- | --- | --- |
| $\eta$ | Noise fraction, as a maturity measure | Maturity features, entering as $1 - \eta$ (`tab:extraction:chain-structure`) |
| $k$ | Cell sample count | The batch context feature (`tab:extraction:batch-context`) |
| $\sigma_1$ | Primary singular value of the score geometry | Structural reliability (`tab:extraction:chain-structure`) |
| $\bar{s}$, $\bar{v}$ | Per-axis baseline mean and variance | Z-score normalisation (`tab:extraction:chain-z-scores`) |
| $\bar{\rho}$ | Per-axis CUSUM accumulators | The drift features (`tab:extraction:chain-cusums`) |

The table explains and binds nothing. A correspondence does not gain force by being written in an appendix, and stating it as a translation aid is what keeps a reader from treating it as an interface in its own right.

**Table (Feed-forward boundary verification)** · `tab:boundary:verification`

The complete inventory. Every boundary in the system appears here with its permitted direction and the mechanism that holds it.

| Boundary | Direction | Mechanism |
| --- | --- | --- |
| Core to Sentinel | Never | The Core reads batch reports and never writes (`inv:guarantee:feed-forward`) |
| Sentinel to its graph | Observe only | Unit-volume observation at a coordinate; the graph handles everything internal to itself |
| Derivation Function to Core | Never | It reads the risk basis and writes nothing (`inv:guarantee:derivation-purity`) |
| Derivation Function to Companion | Never | It reads a challenge posterior and writes nothing (`sig:companion:posterior`) |
| Companion and Core, in either direction | Never | No interface exists in either direction (`inv:companion:boundary`) |
| Core to a Sentinel-owned graph | Never | No interface exists (`tab:boundary:graph-interface`) |

Six boundaries, of which five are never crossed in either direction and the sixth is observe-only. The only cycle in the system passes through the host: the Core assesses, the host decides, acts and reports, and the Core learns. The Derivation Function and the Companion are consulted and never consulted back, which is the same claim the information-flow figure draws rather than states (`fig:architecture:information-flow`).


## Appendix (Cross-Layer Terminology) · `app:spec:terminology-mapping`

Three layers meet in this system and each has its own vocabulary for what is partly the same subject matter. This appendix translates between them. It is expository throughout — it introduces no obligation and constrains no implementation — but the error it exists to prevent is one the Core's design depends on not making, which is why it carries a rule for reading rather than only two tables.

**Table (Structural operations across the three vocabularies)** · `tab:terminology:structural`

The Core responds to observable changes in the batch report, and never to a Sentinel-internal event.

| The shared concept | The index corpus calls it | The Sentinel corpus calls it | What the Core observes, and does |
| --- | --- | --- | --- |
| Finer cells appear | Refinement | Catalytic split | A cell appears, and a fresh Ledger entry is created (`alg:ledger:entry-creation`) |
| A cell disappears | Eviction | Various mechanisms | After the persistence threshold, the entry is deleted (`alg:ledger:entry-deletion`) |
| An absent cell reappears | Restoration | Restoration | A cell appears, indistinguishable from any other appearance (`alg:ledger:entry-creation`) |
| Mass on removal | Exact conservation | The same | No merge is performed — the ancestor entry is already current (`just:ledger:no-state-transfer`) |
| The observation-receiving surface | Bottom contour, at the bottom-contour section of the dual-tree value-stratified index specification | Contour, at the domain-and-spatial-partitioning section of the Sentinel corpus | Not observed directly — the Core reads only the aggregate snapshot |

The fourth column is the only one this document owns, and its entries are effects rather than counterparts. The Core performs no refinement, eviction or restoration; it makes two responses to two observations.

**Table (What "importance" means in each layer)** · `tab:terminology:importance`

One word, three meanings, and the differences are load-bearing.

| Layer | What "importance" means there |
| --- | --- |
| The index and Sentinel corpora | Accumulated unit-volume observation at a coordinate, stated at the Sentinel corpus's observation-and-importance section and fixed algebraically at the index corpus's coding-theoretic foundation's algebraic-interface section |
| The Core, reading a Sentinel | Not read directly — it reaches the Core only through the competitive selection the batch report already performed |
| The Core, in its identity layer | Read directly — competitive sets are defined by importance-ranked depth in the dimension's own graph (`def:keyspace:competitive-set`) |

The second and third rows are the same word doing opposite work, and the distinction is exactly the one the Core's two relationships with a graph rest on: what a Sentinel's graph ranks is the Sentinel's business, and what the Core's own graphs rank is the Core's.

**Convention (The abstraction boundary)** · `conv:terminology:abstraction-boundary`

Causation is traced; operations are not corresponded. The structural mapping above is offered to a reader who understands all three layers, and its Core column describes observable effects, not corresponding operations. Nothing in this document licenses the reading that the Core performs a split, an eviction or a restoration, or that it detects one.

What the Core does is narrower and is stated as such wherever it acts: it observes cell-set changes and responds with entry creation or deletion, and the Sentinel-internal cause of a change is not merely unavailable to it but irrelevant to what it does (`alg:runtime:cell-set-maintenance`). The convention binds as a rule for reading because the alternative reading is the tempting one and it is wrong in a way that propagates: a reader who believes the Core detects evictions will expect it to distinguish an eviction from a restoration, will look for the mechanism that does so, and will find a limitation where there is a boundary (`cav:limitation:abstraction`). No Sentinel-internal event type reaches the maintenance path.

## Appendix (Background) · `app:spec:background`

The Core depends on two upstream components: the dual-tree value-stratified index, which it uses directly as the adaptive spatial index inside each declared identity dimension (`mot:keyspace:purpose`), and the spectral Sentinel, whose batch reports it consumes and whose internals it never touches (`tab:architecture:sentinel-properties`). A reader who knows neither corpus needs an orientation to both before the rest of this document reads as anything but a list of assumptions.

The overviews below restore that orientation without reproducing the upstream specifications. They describe each component only far enough to make its relationship with the Core intelligible, cite the current heads that own the mechanics, and leave every deeper result with its owner (`conv:spec:upstream-references`).

**Overview (Mudlark and the G-V Graph)** · `preview:architecture:mudlark`

Mudlark is the upstream library that provides the G-V Graph, a dual-tree value-stratified index over a dyadic integer domain. Its geometric tree partitions the domain into cells and answers spatial questions; its value tree ranks those same cells by accumulated importance. Observation concentrates resolution where activity warrants it, while decay and eviction let quiet regions become coarse again. The two trees therefore turn an observation stream into both an adaptive spatial partition and a significance ordering (`[MUDLARK-sec:mudlark:readme-architecture-in-brief]`).

The Core owns a separate G-V Graph for each registered identity dimension. At assessment it supplies the host-encoded entity coordinate as a unit-volume observation; asynchronously it reads the graph's current competitive cells and turns containment in those cells into model features (`alg:keyspace:observation-protocol`). What the Core consumes from Mudlark is structural state — which ranges have earned attention — rather than an anomaly judgement. What it supplies is observation volume and the host's declared decay policy, never a risk estimate or label-derived score. This is an ownership boundary: these graph instances are Core state, created to discover significant ranges in the host's own key spaces (`mot:keyspace:purpose`).

A Sentinel also owns a G-V Graph, but that is a different instance behind a different boundary. Its graph decides where the Sentinel spends statistical modelling effort; the Core neither receives that graph nor calls it. The shared substrate explains the common vocabulary, while separate ownership prevents a Core identity graph from becoming a back door into a Sentinel.

**Overview (Spectral Sentinel)** · `preview:architecture:spectral-sentinel`

Spectral Sentinel is a hierarchical online anomaly-measurement system for streams of positionally structured integer coordinates. It uses Mudlark to adapt a spatial partition and select significant cells, closes that selection under spatial ancestry, and maintains low-rank statistical trackers at the selected cells and coordination contexts. Those trackers measure novelty, displacement, surprise, coherence, drift, maturity and cross-cell structure at several spatial scales; they emit measurements rather than threat levels or actions (`[SENTINEL-sec:sentinel:readme-architecture-in-brief]`).

The host, not the Core, encodes and feeds observations to each Sentinel and controls its temporal policy. An observation cycle produces a batch report with competitive-cell and ancestor measurements, coordination reports and structural summaries (`[SENTINEL-sec:sentinel:readme-report-structure]`). The Core consumes that detached report, compresses its variable multi-scale contents into a fixed per-Sentinel feature slot (`setup:extraction:from-batch-report`), and preserves the reported cell ranges in its own outcome memory. It deliberately does not consume Sentinel configuration, internal trackers, live graph state or query surfaces; the complete consumed and refused surfaces are (`tab:boundary:consumed`) and (`tab:boundary:not-consumed`).

The Core supplies nothing to a Sentinel. It holds no Sentinel handle, sends no configuration, requests no per-observation score and returns no risk estimate, label or action. Periodic reports are the whole interface (`alg:runtime:report-reception`). That one-way boundary prevents a learned judgement from reshaping the instrument that measured it: Sentinels remain independently useful, and Core failures can travel upward to the host but never downward into measurement (`inv:guarantee:feed-forward`).
