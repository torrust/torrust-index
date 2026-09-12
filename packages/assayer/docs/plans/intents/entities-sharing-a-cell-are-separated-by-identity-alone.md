# Entities sharing a cell are separated by identity alone · `plan:assayer:intent-entities-sharing-a-cell-are-separated-by-identity-alone`

Keeping (´claim:identity:entities-sharing-a-cell-are-separated-by-identity-alone´) establishes that two subjects with indistinguishable Sentinel geometry acquire a useful risk ordering from identity features while their one shared spatial outcome average continues to describe only the balanced population.

## What the promise says, precisely · `sec:assayer:intent-entities-sharing-a-cell-are-separated-by-identity-alone-promise`

The fixture contains two stable entity keys, one identity dimension, one channel, one reporting Sentinel, no declared signals, and no outcome axes. Both entities carry the same Sentinel identifier and coordinate through every assessment, while the identity encoder places their keys in opposite halves of its domain; the encoding follows the host contract (´req:keyspace:encoding-contract´), and the resulting cell indicators are the features on which the model learns per-range weights (´def:keyspace:competitive-indicators´).

The Sentinel report contains only its permanent root cell. Every label therefore updates the same Ledger entry, whose role is the adverse rate of all traffic routed through the Sentinel rather than either entity's rate (´def:ledger:root-semantics´); this is the intended separation between Sentinel spatial memory and identity state (´tab:ledger:versus-identity´).

The stimulus is exactly five hundred alternating assess-label cycles, beginning with the benign entity and ending with the adverse entity, so each entity contributes two hundred and fifty labels and the shared population is exactly balanced. Every label records `Action::Allow`, which admits both classes to all three risk models under the eligibility table (´tab:eligibility:training´).

Discrimination is the tie-corrected pairwise rank probability over the assessment-time `p_bad` readings, with one vector for each outcome class as in the specified AUC computation (´alg:monitoring:auc´). The contractual floor is `auc > 0.65`: it is significantly above chance, enters the specification's moderate band (´tab:monitoring:interpretation´), and matches the existing five-hundred-label public convergence witness (´test:integration:scalar-signal-population-split-converges-directionally´).

The post-training point estimate supplies a second quantity: the adverse entity's held-out `p_bad` exceeds the benign entity's by more than `0.10`. This prevents a chronology-driven prequential AUC from standing in for separation in the model that exists after the five-hundredth label.

The spatial control is `abs(root_adverse_rate - 0.5) < 0.01`. The fixture sets the legal Ledger rate `lambda_l` to `0.99` under the configured interval (´tab:config:ledger´); for an alternating balanced stream from zero, the specified EWMA recurrence (´tab:ledger:entry-state´) ends near one half within five hundred updates rather than merely tending there after the witness ends.

The identity dimension uses depth cutoff one, two depth-one cells, the default identity outcome rate `0.95`, and an encoder that changes only the most significant domain bit. Its distinct indicators and its cross-dimension maximum adverse-rate feature are identity inputs (´tab:keyspace:cross-dimension-features´), while every non-identity input remains identical between the two classes.

The standing testing plan has no gap Entry covering this promise; it carries only the threshold correction that replaces “close to one” with a materially above-chance AUC plus the near-half cell control (´obs:assayer:reframe-identity-threshold´).

## What the code offers today · `sec:assayer:intent-entities-sharing-a-cell-are-separated-by-identity-alone-code`

The scenario builder exposes `scenario_with_config`, so the test can select the Ledger rate while retaining the seeded `World` construction and tracing lifetime.

The world harness exposes `World::register_sentinel`, `World::receive_report`, `World::request_with_sentinel`, `World::assess`, `World::settle_cold_ramp_with`, `World::flush_observations`, `World::flush_labels`, and `cycle_request`; the completed scenario skeleton records these public-path verbs and their deterministic barriers (´entry:assayer:harness-stage-skeleton´).

The report fixtures expose `minimal_report`, a valid root-only `BatchReport`; the label builder exposes the adverse and benign labels; the detailed health report exposes a `ledger.per_sentinel` entry carrying `entry_count`, `max_depth`, and `root_adverse_rate`.

The existing public convergence witness already owns the five-hundred-cycle constant, the `0.65` AUC floor, the `0.10` held-out gap, and the private tie-corrected `pairwise_auc` helper, so the new witness extends the same integration module without creating a second rank oracle (´test:integration:scalar-signal-population-split-converges-directionally´).

The nearest identity witness demonstrates a public `IdentityDimensionRegistration`, a high-bit encoder, a seeded closed loop, and health-based evidence that competitive geometry exists before a reading is taken (´test:integration:the-association-reading-separates-a-keyed-dimension-from-a-null-one´).

The graph itself has a dedicated asynchronous owner (´dec:memory:graph-owner´), and competitive sets publish independently after its maintenance pass (´dec:memory:competitive-publication´). `World` currently exposes no barrier for that queue, so the nearest test settles geometry by driving extra labeled traffic until a health predicate holds.

The identity maintenance loop already implements `MaintenanceCommand::PrepareCheckpoint`, which drains identity observations, detects competitive changes, emits lifecycle events, publishes graph snapshots, and acknowledges, including the backlog-before-snapshot ordering kept by (´claim:identity:a-checkpoint-drains-the-backlog-first-so-its-snapshot-omits-no-submitted-observation´). The shared liveness support supplies `ACK_DEADLINE`, while the model-owner checkpoint used by `World::flush_labels` supplies the subsequent publication barrier.

The label path reconstructs current competitive indicators and separately updates the assessment-time identity cells (´alg:runtime:identity-outcome-update´), following the general label update order (´alg:runtime:update-path´) and the record's frozen-versus-rederived split (´dec:vector:label-time-assembly´). Settling the final two-cell layout before labeled stimulus removes lifecycle reconstruction from the quantity under test.

## The witness · `sec:assayer:intent-entities-sharing-a-cell-are-separated-by-identity-alone-witness`

The integration test is named `entities_sharing_a_cell_are_separated_by_identity_alone` and lives beside the existing public convergence witness (´test:integration:scalar-signal-population-split-converges-directionally´), with its claim and derived test labels added to that module's index.

Setup builds an explicit `AssayerConfig` with `ledger.lambda_l = 0.99`, one default channel, a fixed seed, and otherwise ordinary harness values. It registers one Sentinel, ingests `minimal_report`, and registers one 128-bit identity dimension at cutoff one with a positive split threshold and an encoder mapping the benign and adverse keys to opposite high bits.

Balanced, unlabelled requests for the two entities are offered in fixed bursts. Each burst ends on the new identity barrier and the model-owner publication barrier, and the loop stops only when detailed health reports two depth-one cells with no dropped identity observations.

`World::settle_cold_ramp_with` then retires the standardisation prior against the same two request shapes under the final layout. A last identity and model publication barrier confirms that warm-up introduced no further structural event before the measured stream.

Fixture guards require one Sentinel Ledger entry at depth zero, two identity cells at depth one, equal Sentinel coordinates in the two request constructors, no signals or axes, an initial root adverse rate of zero, and no identity observation drops. These guards prove that spatial geometry and auxiliary feature families cannot supply the class distinction.

The measured loop alternates benign then adverse for two hundred and fifty pairs. Each request is built with the same channel, Sentinel, and coordinate; `cycle_request` returns its pre-label reckoning, whose `p_bad` is appended to the vector for that entity before `LabelSpec::benign` or `LabelSpec::adverse` is applied and flushed.

Observation begins after the final label acknowledgement. The existing `pairwise_auc` ranks all adverse pre-label risks against all benign pre-label risks, two fresh unlabelled assessments provide the post-training risk gap, and `full_health_report` supplies the Sentinel's root adverse rate and structural controls.

The assertions require finite in-range risks, `auc > 0.65`, adverse-minus-benign held-out risk greater than `0.10`, `abs(root_adverse_rate - 0.5) < 0.01`, the unchanged one-entry root-only Ledger, the unchanged two-cell identity geometry, and clean engine health.

The fails-before is an implementation that zero-fills identity indicators and identity aggregates, omits the identity block from assembly, or maps both keys to the same active cell. Both classes then present the same risk features, the AUC falls to chance and the held-out gap collapses, while the shared root still approaches one half; the joint oracle therefore fails on identity separation rather than on the control.

## What is missing · `sec:assayer:intent-entities-sharing-a-cell-are-separated-by-identity-alone-missing`

**Entry (Deterministic identity-maintenance barrier)** · `entry:assayer:intent-entities-sharing-a-cell-are-separated-by-identity-alone-identity-barrier`

This entry adds approximately thirty lines to the world harness for `World::flush_identity`: send `MaintenanceCommand::PrepareCheckpoint`, await its acknowledgement under `ACK_DEADLINE`, then run the existing model-owner settlement so emitted cell lifecycle events are present in the published layout before return. It depends on the completed harness skeleton (´entry:assayer:harness-stage-skeleton´), widens no production API, uses no sleep, and has no sibling-lane dependency.

**Entry (Two-entity one-cell fixture)** · `entry:assayer:intent-entities-sharing-a-cell-are-separated-by-identity-alone-fixture`

This entry adds approximately fifty lines to the convergence integration module for the high-bit encoder, root-only Sentinel attachment, bounded balanced observation bursts, depth-one health predicate, and fixture guards. It depends on the identity-barrier entry and the configured identity contract (´tab:config:identity´), and it has no sibling-lane dependency.

**Entry (Closed-loop separation witness)** · `entry:assayer:intent-entities-sharing-a-cell-are-separated-by-identity-alone-public-witness`

This entry adds approximately fifty lines to the convergence integration module for the alternating cycle, prequential AUC collection, held-out gap, root-rate control, health assertions, test documentation, and module-index row. It depends on the fixture entry, reuses the existing rank helper and thresholds, and has no sibling-lane dependency.

## Risks and open questions · `sec:assayer:intent-entities-sharing-a-cell-are-separated-by-identity-alone-risks`

**Observation (The promised ceiling becomes an informative floor)** · `obs:assayer:intent-entities-sharing-a-cell-are-separated-by-identity-alone-threshold`

“Close to one” has no numeric boundary and would turn ordinary finite-sample movement into an uninformative failure. The chosen strict floor of `0.65` follows the standing correction (´obs:assayer:reframe-identity-threshold´), while the independent held-out gap prevents that lower threshold from accepting a ranking that existed only early in the stream.

**Observation (The near-half control requires a fixture rate)** · `obs:assayer:intent-entities-sharing-a-cell-are-separated-by-identity-alone-ledger-rate`

At the default `lambda_l = 0.999`, the Ledger formula leaves a balanced rate from zero near `0.197` after five hundred labels, so a literal near-half assertion contradicts the promise's horizon. The configured `0.99` is inside the specified domain (´tab:config:ledger´) and puts the alternating endpoint near `0.499`; the test therefore keeps the compositional promise at a declared operating point rather than claiming that the default has already converged.

**Observation (Geometry settlement is state-based)** · `obs:assayer:intent-entities-sharing-a-cell-are-separated-by-identity-alone-geometry-settlement`

Identity observation is deliberately deferred, so label count and wall time do not identify a competitive layout (´dec:memory:graph-owner´). An acknowledged queue drain, a model publication barrier, and a structural health predicate replace scheduler timing; a fixed burst ceiling remains only a liveness failure for a graph that never forms the two configured branches.

**Observation (Warm-up cannot carry the outcome)** · `obs:assayer:intent-entities-sharing-a-cell-are-separated-by-identity-alone-warmup-purity`

The setup assessments are unlabelled and therefore advance observational geometry and standardisation but no outcome-learned state (´dec:ordering:evidence-authority´). Settling the cold ramp after the last cell entry ensures both held-out entities are compared in one final coordinate system rather than across the finite prior-mass transition (´dec:vector:prior-mass-ramp´).

**Observation (Alternation controls chronology)** · `obs:assayer:intent-entities-sharing-a-cell-are-separated-by-identity-alone-chronology`

Grouping all benign labels before all adverse labels would let general learning progress correlate with class and inflate a prequential rank statistic. Pairwise alternation balances chronology at every adjacent pair, and the post-training gap remains the control against any residual time ordering.

## Acceptance · `sec:assayer:intent-entities-sharing-a-cell-are-separated-by-identity-alone-acceptance`

The implementation report names every harness and integration-test file changed, the fixed seed, identity registration and budget, Sentinel coordinate, Ledger rate, label and class counts, AUC floor, held-out gap, root-rate tolerance, geometry predicate, and liveness ceiling.

The report shows the observed AUC, both held-out risks and their gap, the root adverse rate, Sentinel Ledger entry count and depth, identity depth histogram, dropped-observation count, and final health verdict.

The report includes a red result from the identity-blind fails-before or an equivalent deliberate weakening, a green result after the witness lands, and the required formatting, lint, package integration, no-default-feature, documentation, and release-test gates with their commands and verdicts.

Acceptance requires repeated fixed-seed runs to retain the same verdict without sleeps or widened production visibility, exactly five hundred measured labels with balanced classes, a single shared Sentinel root throughout, two distinct identity branches before the first measured label, AUC above `0.65`, held-out separation above `0.10`, and root adverse rate within `0.01` of one half.
