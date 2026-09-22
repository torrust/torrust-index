# Keeping Restore-Time Decay to One Application · `plan:assayer:intent-restore-time-decay-is-applied-exactly-once`

This plan keeps the promise that a restore discounts checkpointed state for the outage once while a journalled label replayed immediately afterwards carries no second charge for the same interval (´claim:persistence:restore-time-decay-is-applied-exactly-once´).

## What the promise says, precisely · `sec:assayer:intent-restore-time-decay-is-applied-exactly-once-promise`

Let a checkpoint be captured at persistent time $t_0$, restored at $t_1$, and let $\Delta t=(t_1-t_0)$ hours. For the configured Core time rate $r=\gamma_{t,\mathrm{core}}$, the restore factor is $f=r^{\Delta t}$, independently of the label-indexed factors that later updates apply (´def:temporal:two-mechanisms´).

Immediately after restore and before replay, every Core model must satisfy $\mu_R=\mu_C$, $B_R=fB_C$, and $\Sigma_R=f^{-1}\Sigma_C$: elapsed decay widens confidence without moving the posterior mean (´alg:temporal:lazy-application´). The checkpoint-to-restore interval is read once in the persistent domain and applied once per model family (´rem:clock:persistence-inheritance´) (´dec:durability:decay-once´).

For a journalled label $\ell$ replayed after that restore, the time-indexed factor at the head of the label path is unity when the scenario clock remains at $t_1$: reconstruction initialises `last_label_time` from the engine clock's current monotonic reading, and replay reads that same present (´dec:clock:two-domains´). The label still receives its configured label-indexed factor; the already-accounted outage interval is absent from its update (´dec:ordering:decay-at-head´) (´dec:durability:decay-once´).

The measurable equivalence is $\operatorname{replay}_{t_1}(\operatorname{decay}_{t_0\rightarrow t_1}(C),\ell)=\operatorname{live}_{t_1}(C,\ell)$ for the published model state, where the live path carries $C$ across the same interval and applies its one elapsed factor at the label-path head. Both arms share the checkpoint prefix, assessment-time feature vector, label data, label-indexed factor, and virtual-time interval, leaving only restore-plus-replay versus uninterrupted live application as the variable (´cor:durability:replay-exactness´).

The fixture sets `temporal.gamma_t_core` to $0.81$ and uses $\Delta t=1$ hour, both admitted by the temporal configuration surface (´tab:config:temporal´). One elapsed application therefore multiplies covariance by $1/0.81$, while a second multiplies it by $1/0.81^2$; the scale factors differ by about $0.29$, far beyond the harness's generic one-shot tolerance.

The decay-only arrangement compares every covariance-diagonal ratio with the reciprocal of `decay_recurrence(1.0, 1.0, 0, 0.81, 1.0)` under `World::tol().default`; the replay arrangement compares complete published Core means, covariances, dimensions, and floor masses with its live-label arm under the same tolerance (´dec:harness:oracle-tier´) (´tab:assayer:harness-scenario-tolerances´). Setup guards require finite positive covariance diagonals and at least one covariance entry to differ from its cold value by more than that tolerance; the result oracle separately requires the observed once-only scale to differ from unity and the twice-applied scale by more than the same tolerance (´dec:harness:guarded-fixtures´) (´dec:harness:separate-validation´).

This is an orchestration promise rather than another proof of exponential arithmetic. The shared decay functions and zero-interval identity already have focused witnesses (´dec:clock:shared-functions´) (´test:integration:decay-factor-since-same-time´); the missing fact is that the restore and replay paths together charge the interval once.

## What the code offers today · `sec:assayer:intent-restore-time-decay-is-applied-exactly-once-current-code`

The test-facing construction path is `WorldBuilder::persistence_dirs`, `WorldBuilder::clock`, and `WorldBuilder::build`; the last delegates to `AssayerBuilder::build`. `World::derive_default`, `World::label`, and `World::flush_labels` drive the public assessment and label path, while construction selects restore when the checkpoint is usable (´dec:construction:two-starts´). Recovery reads the checkpoint timestamp through the injected persistent clock, applies one configured factor to every model family, reconstructs `last_label_time` from the current monotonic reading, filters journal entries above the checkpoint mark, and sends them through the ordinary label pipeline (´dec:durability:checkpoint-journal´) (´dec:durability:decay-once´) (´cor:durability:replay-exactness´).

`World::fork_persistence` and `World::fork_persistence_after` now quiesce the source through the complete barrier set, copy its checkpoint and journal into isolated owned storage, retain one shared `VirtualClock`, and rebuild the restored arm. The elapsed form moves the shared clock through `World::advance` before reconstruction; `PersistenceFork::arms`, `PersistenceFork::into_arms`, `PersistenceFork::roots`, and `PersistenceFork::journal_bytes` expose the two worlds and the narrow durable evidence this witness needs (´cor:durability:harness-fork´) (´entry:assayer:harness-persistence-fork´).

The unified harness supplies deterministic seeds, complete builders, forward-only `World::advance` and `World::travel_to`, `World::settle_cold_ramp_with`, `World::flush_observations`, `World::flush_labels`, `World::flush_identity_maintenance`, owner shutdown, `LabelSpec`, and the liveness bounds `ACK_DEADLINE` and `STATE_DEADLINE`; no sleep, poll, raw-clock movement, or test-authored deadline is needed (´tab:assayer:harness-implementation-library-roster´) (´entry:assayer:harness-closed-barriers´) (´entry:assayer:harness-scenario-time´) (´dec:harness:no-ad-hoc-waits´).

Long alternating prefixes can be expressed as subject-owned `PlaybackRow` values interpreted by `playback` under a `PlaybackBarrierPolicy` that selects the observation and label barriers, with `PlaybackProgress` supplying row diagnostics rather than timing evidence (´dec:harness:declarative-playback´) (´entry:assayer:harness-tape-runner´). The specification-formula oracle `decay_recurrence` computes the elapsed factor independently of the production exponentiation route (´dec:harness:oracle-tier´) (´entry:assayer:harness-oracle-tier´).

**Entry (Published Core-model probe)** · `entry:assayer:intent-restore-time-decay-is-applied-exactly-once-published-model-probe`

This entry is supplied by `PublishedModelBlock` and `World::published_model_block`. Each call crosses the model-publication barrier, loads one immutable snapshot, and returns owned model identity, version, layout generation, observation time, dimension, mean, full column-major covariance, and floor masses without exposing precision or a mutation path (´dec:harness:probe-contract´) (´entry:assayer:harness-probe-contract´). Focused tests already establish barrier freshness, one-version coherence, independence from later publication, and scenario-clock provenance (´test:crate:published-model-block-crosses-publication-barrier´) (´test:crate:published-model-block-is-from-one-version´) (´test:crate:published-model-block-is-independent-of-later-publication´) (´test:crate:published-model-block-uses-scenario-time´).

The closest landed witnesses divide the promise without joining it. `persistence_fork_decay_once` checks the configured restore factor on operational precision, rejects an independently double-decayed durable projection, and compares a common live suffix after restore (´test:crate:persistence-fork-decay-once´). `persistence_fork_keeps_the_label_decay_clock` checks that an immediate post-restore live label does not repeat a pre-checkpoint interval (´test:crate:persistence-fork-keeps-the-label-decay-clock´). `label_reports_model_owner_shutdown_journal_replays_on_restart` proves durable replay and journal truncation with a fixed clock but compares only the label count and journal length (´test:integration:label-reports-model-owner-shutdown-journal-replays-on-restart´). `recovery_decay_applied_to_precision` checks restore-decay direction without replay (´test:crate:recovery-decay-applied-to-precision´), and `from_snapshot_resets_last_label_time` checks reconstruction's clock reset outside checkpoint-plus-journal recovery (´test:crate:from-snapshot-resets-last-label-time´).

No test mints or cites the promise's claim label, and none of those tests compares a label journalled before an outage with its uninterrupted application after that outage. The promise therefore remains unkept, although the finished harness and the landed partial witnesses leave only the integration witness itself missing.

## The witness · `sec:assayer:intent-restore-time-decay-is-applied-exactly-once-witness`

The witness is the serde-gated integration test `persistence_decay::restore_time_decay_is_applied_exactly_once`. Its module index states the promise and its test documentation cites this intent. It uses the public `World` lifecycle plus gated read-only probes; precision remains confined to the existing crate witness.

- Setup fixture: build a seeded persistent `World` at $t_0$ with one default channel, no runtime registrations, `temporal.gamma_t_core = 0.81`, and a shared `VirtualClock`. Capture cold operational, sister, and anchor `PublishedModelBlock` values, complete the cold ramp through `World::settle_cold_ramp_with`, then drive a declared even alternating ground-truth prefix over one fixed bias-only request through `playback` under an ordered `PlaybackBarrierPolicy` of `PlaybackBarrier::FlushObservations` followed by `PlaybackBarrier::FlushLabels`. Capture checkpoint-prefix blocks $P_C$; before returning, the setup guard requires the expected dimension for each block, one published version across the three blocks, finite positive covariance diagonals, and at least one covariance entry changed from its cold value by more than `World::tol().default` (´dec:harness:declarative-playback´) (´dec:harness:guarded-fixtures´).

- Decay-only arrangement: build one guarded prefix, call `World::fork_persistence_after` for exactly one hour, and read the restored arm before any suffix label. The fork moves time by its internal `World::advance` and reconstructs under the resulting shared present (´entry:assayer:harness-scenario-time´) (´obs:assayer:harness-implementation-clock-domain-shape´). For every operational, sister, and anchor covariance diagonal, compare $P_D/P_C$ with the reciprocal of the factor returned by `decay_recurrence`; require every mean to equal its checkpoint value, and require the observed ratio to be separated from both unity and the twice-applied factor.

- Replay branch point: build a second guarded prefix and call `World::fork_persistence` at $t_0$, retaining the original external-root arm as the journal source and the zero-gap restored arm as the live-label reference. Before assessing, require both arms' three published blocks to equal the guarded prefix, excluding zero-gap reconstruction as a confound. On both arms, assess the same fixed request and retain their separate assessment identifiers. Capture the journal header bytes through `PersistenceFork::journal_bytes` before either suffix label, then take ownership of both worlds through `PersistenceFork::into_arms`.

- Durable pre-outage label: stop the journal-source model owner through `World::shut_down_model_owner_for_test`, submit its ground-truth label through `World::label`, require the established owner-shutdown error, and require the journal to differ from its captured header. The synchronous error follows the durable append, so this step needs no file wait (´test:integration:label-reports-model-owner-shutdown-with-journal-preserves-durable-record´).

- Outage and live reference: move the live-label arm exactly one hour with `World::advance`, submit the already-prepared matching label, cross `World::flush_labels`, and capture its three blocks as $P_L$. Holding the pending assessment across the advance makes the assessment-time context equal to the journalled context, while the live label path applies the interval once at $t_1$.

- Replay stimulus: drop the stopped source, rebuild its original durable root at the shared clock's unchanged $t_1$ reading with the same configuration, seed, channel, and clock, cross `World::flush_labels` to complete replay and its checkpoint, and capture its three blocks as $P_R$. Require the restored label count to advance exactly once beyond the checkpoint count and the journal bytes to equal the captured header.

- Exact-once assertion: for operational, sister, and anchor blocks, require equal model identity, dimension, mean, full covariance, spectral floor mass, and clamp floor masses in $P_R$ and $P_L$ under `World::tol().default`, and require one version within each three-block reading. The live-label arm applies the hour once at its label; the rebuilt arm applies it once before replay, so omission or repetition separates the blocks.

- Completion observation: assess the fixed request once on both completed arms and compare the complete public `RiskBasis` values through `assert_risk_basis_near` under the same tolerance. Apply `assert_health_clean` to both worlds so matching model probes cannot conceal a degraded recovery path.

The fails-before is a restore that initialises the replay path from the checkpoint's old label-clock mark after already applying bulk decay. Replay then applies the same hour again before the journalled evidence, so $P_R$ diverges from the once-decayed live $P_L$. Omitting bulk restore decay likewise leaves $P_R$ on the undecayed checkpoint while $P_L$ applies the hour at its live label; the decay-only arrangement additionally distinguishes the configured once-only factor from both mutations.

## What is missing · `sec:assayer:intent-restore-time-decay-is-applied-exactly-once-missing`

**Entry (The once-only restore integration witness)** · `entry:assayer:intent-restore-time-decay-is-applied-exactly-once-integration-witness`

Only the integration target remains. It adds a subject-owned playback row and guarded persistent-prefix setup, an elapsed decay-only fork, a zero-gap fork whose journal-source owner stops before the clock advances, live-versus-replay `PublishedModelBlock` comparisons, the `decay_recurrence` factor oracle, label-count and journal-truncation checks, public-risk confirmation, and clean-health assertions. Every shared mechanism it consumes is already present; no new harness module, probe, barrier, fixture API, clock verb, oracle, or production surface is required.

## Risks and open questions · `sec:assayer:intent-restore-time-decay-is-applied-exactly-once-risks`

**Observation (A restart takes its current monotonic reading from the engine)** · `obs:assayer:intent-restore-time-decay-is-applied-exactly-once-monotonic-epoch`

`World::fork_persistence` keeps both arms on one `VirtualClock`, and the live-label arm alone advances it through `World::advance`; rebuilding the stopped original arm then reads that same current monotonic value. No monotonic timestamp crosses the checkpoint boundary, so replay at the unchanged $t_1$ present has zero elapsed time to charge. The existing constructor-clock fork test covers that reset without the journal tail this witness adds (´test:crate:persistence-fork-keeps-the-label-decay-clock´).

**Observation (The oracle separates restore from replay)** · `obs:assayer:intent-restore-time-decay-is-applied-exactly-once-two-part-oracle`

The decay-only arrangement checks the specification-formula factor independently through `decay_recurrence`; the live-versus-replay arrangement checks where that factor is applied relative to the journalled evidence. The first rejects zero or two bulk applications, and the second rejects an extra replay application; fixture guards remain separate from both result checks (´dec:harness:oracle-tier´) (´dec:harness:separate-validation´).

**Observation (Only model-equivalent context belongs in the branch comparison)** · `obs:assayer:intent-restore-time-decay-is-applied-exactly-once-context-equivalence`

Both fork arms assess the same request at $t_0$ before either label is submitted. With no Sentinels, outcome axes, or identity dimensions, their stored feature vectors and assessment-time risk bases are model-equivalent; one label is journalled before the outage and the other pending label is submitted live after it. Comparing model state avoids calibration, latency, or pending-buffer diagnostics whose histories legitimately differ.

**Observation (Acknowledgements define every file boundary)** · `obs:assayer:intent-restore-time-decay-is-applied-exactly-once-file-boundaries`

`World::fork_persistence` quiesces the prefix before copying it, the failed live enqueue returns only after its journal append, and the rebuilt branch's `World::flush_labels` completes replay publication and checkpoint truncation before any reading. Journal bytes are read only at those acknowledged boundaries; file polling and sleeps add no evidence (´entry:assayer:harness-closed-barriers´) (´dec:harness:no-ad-hoc-waits´).

**Observation (One conspicuous configured hour is a discriminating fixture)** · `obs:assayer:intent-restore-time-decay-is-applied-exactly-once-configured-factor`

The reference Core rate would move covariance only slightly over one hour. The configuration contract permits $0.81$, numerical retuning is structurally compatible with restore, and `decay_recurrence` derives the resulting factor from the stated rate and interval rather than fitting a tolerance (´tab:config:temporal´) (´dec:durability:structural-compatibility´). The chosen rate is test stimulus, not a production default.

No maintainer decision remains open: the durability and ordering records fix when decay occurs, the temporal specification fixes its arithmetic, and the finished harness supplies every lifecycle, observation, completion, playback, fork, and oracle surface the witness needs.

## Acceptance · `sec:assayer:intent-restore-time-decay-is-applied-exactly-once-acceptance`

The implementing report names cargo integration target `persistence_decay` and Rust test path `persistence_decay::restore_time_decay_is_applied_exactly_once`, confirms that the module index and test documentation cite this intent, and shows the coverage projection resolving the promise to that test.

The report shows the guarded non-cold prefix; the configured reciprocal covariance scale with unchanged means on the decay-only restore; a label made durable before the hour; equality of all three published Core blocks after live application and replay; one-label counter advancement; journal truncation to the captured header; matching public risk fields; and clean health.

The report shows the serde-enabled target and the existing restore-factor, label-clock, durable-replay, recovery-decay, and snapshot-clock witnesses passing under the package's prescribed test, formatting, clippy, documentation, all-feature, and featureless-library gates. The integration target is absent rather than failing when its serde prerequisite is disabled.

The report states the deliberately broken omitted-decay and repeated-decay behaviours and names the decay-only factor assertion and live-versus-replay block comparison that reject them. No executed mutation is required.
