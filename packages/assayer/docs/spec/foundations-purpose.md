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
