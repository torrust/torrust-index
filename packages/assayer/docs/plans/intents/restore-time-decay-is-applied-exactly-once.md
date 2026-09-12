# Keeping Restore-Time Decay to One Application · `plan:assayer:intent-restore-time-decay-is-applied-exactly-once`

This plan keeps the promise that a restore discounts checkpointed state for the outage once while a journalled label replayed immediately afterwards carries no second charge for the same interval (´claim:persistence:restore-time-decay-is-applied-exactly-once´).

## What the promise says, precisely · `sec:assayer:intent-restore-time-decay-is-applied-exactly-once-promise`

Let a checkpoint be captured at persistent time $t_0$, restored at $t_1$, and let $\Delta t=(t_1-t_0)$ hours. For the configured Core time rate $r=\gamma_{t,\mathrm{core}}$, the restore factor is $f=r^{\Delta t}$, independently of the label-indexed factors that later updates apply (´def:temporal:two-mechanisms´).

Immediately after restore and before replay, every Core model must satisfy $\mu_R=\mu_C$, $B_R=fB_C$, and $\Sigma_R=f^{-1}\Sigma_C$: elapsed decay widens confidence without moving the posterior mean (´alg:temporal:lazy-application´). The checkpoint-to-restore interval is read once in the persistent domain and applied once per model family (´rem:clock:persistence-inheritance´), (´dec:durability:decay-once´).

For a journalled label $\ell$ replayed after that restore, the time-indexed factor at the head of the label path is unity because the restored process's monotonic label clock starts at its current instant. The label still receives its configured label-indexed factor; only the already-accounted outage interval is absent from its update (´dec:ordering:decay-at-head´).

The measurable equivalence is $\operatorname{replay}_{t_1}(\operatorname{decay}_{t_0\rightarrow t_1}(C),\ell)=\operatorname{live}_{t_1}(\operatorname{decay}_{t_0\rightarrow t_1}(C),\ell)$ for the model state published after either label. This compares the same checkpoint, elapsed factor, reconstructed feature vector, label data, and label-indexed factor, leaving replay's elapsed-time treatment as the only variable.

The fixture sets `temporal.gamma_t_core` to $0.81$ and uses $\Delta t=1$ hour, both admitted by the temporal configuration surface (´tab:config:temporal´). One restore therefore multiplies covariance by $1/0.81$, while a second charge would multiply it by $1/0.81^2$; the alternatives are separated by roughly $0.29$ as scale factors rather than by scheduler-sized noise.

The decay-only branch compares covariance-diagonal ratios with $1/0.81$ under `World::tol().default`, and the replay branch compares its complete published Core means and covariances with the live-label branch under the same one-shot tolerance (´tab:assayer:harness-scenario-tolerances´). Non-vacuity guards require each captured covariance diagonal to be finite and positive, at least one trained covariance entry to differ materially from its cold value, and the once-only and twice-applied scale factors to differ by more than one hundred tolerances.

This is an orchestration promise rather than another proof of exponential arithmetic. The shared decay functions and zero-interval identity already have focused witnesses (´dec:clock:shared-functions´), (´test:integration:decay-factor-since-same-time´); the missing fact is which interval the restore and replay paths hand to them.

## What the code offers today · `sec:assayer:intent-restore-time-decay-is-applied-exactly-once-current-code`

The host-facing path is `WorldBuilder::persistence_dirs(...)` into `AssayerBuilder::build()`, followed by `World::derive_default`, `World::label`, and `World::flush_labels`. Construction chooses restore when the checkpoint is usable (´dec:construction:two-starts´), and the flush is an acknowledged command-and-label barrier that also produces a checkpoint when persistence is enabled.

The recovery path reads the checkpoint timestamp against the builder's injected persistent clock, computes one Core, Ledger, and identity factor, mutates the restored payloads, and filters journal entries beyond the checkpoint high-water mark. Construction then sends each retained entry through the ordinary owner label pipeline as replay context, which is the arrangement behind exact replay (´dec:durability:checkpoint-journal´), (´cor:durability:replay-exactness´).

Working-copy reconstruction installs a fresh `last_label_time`, and the owner label path measures the next time factor from that mark before combining it with the label factor. The virtual clock can create one clock at $t_0$ and separate fresh clocks at $t_1$ from its declared epoch (´const:assayer:harness-clock-epoch´), keeping the persistent readings comparable while restarting the monotonic domain as required by the type-level separation (´dec:clock:two-domains´).

The completed harness skeleton supplies deterministic seeds, configurable builders, `World::clock`, owner shutdown, `LabelSpec`, label and observation barriers, named tolerances, and the liveness bounds `ACK_DEADLINE` and `STATE_DEADLINE` (´entry:assayer:harness-stage-skeleton´). No real sleep or wall-clock assertion is needed.

The nearest public error-handling witness creates an acknowledged checkpoint, leaves a label durable after owner shutdown, rebuilds from the same directories, waits for replay, and observes journal truncation (´test:integration:label-reports-model-owner-shutdown-journal-replays-on-restart´). It proves that replay happens but holds the clock fixed and observes only the label count and file length.

The nearest crate persistence witness restores a day-old checkpoint and proves that precision falls while covariance rises (´test:crate:recovery-decay-applied-to-precision´). It has no replayed label and asserts direction rather than the configured factor. The snapshot test proves that reconstruction restarts the label clock (´test:crate:from-snapshot-resets-last-label-time´), but it does not cross the checkpoint-plus-journal construction path.

The public risk basis exposes effective uncertainty and therefore reflects model decay (´schema:risk:basis´), with a read-time correction for time since publication (´def:runtime:time-correction´). That nonlinear blend is a poor oracle for an exact multiplicity: the immutable published snapshot already contains the model means and covariances, while the excluded precision matrix has no assessment reader (´dec:retention:precision-excluded´).

The standing testing plan directly assigns this promise to the unfinished persistent-clock stage (´entry:assayer:harness-stage-clock´). Its open persistence-fixture and snapshot-diffing stage also supplies the natural home for the narrow probe below (´entry:assayer:harness-stage-tapes´); no separate standing gap entry covers this promise.

## The witness · `sec:assayer:intent-restore-time-decay-is-applied-exactly-once-witness`

The witness belongs in a new persistence integration-test module, whose module index states the same promise and whose test documentation cites the intent. It remains a public-path integration test: the only inspection addition projects the immutable snapshot the assessment path already reads and does not expose or mutate the working copy.

- Setup: build a seeded world at $t_0$ with persistence, one default channel, no runtime registrations, `gamma_t_core = 0.81`, and a fresh virtual clock anchored at the harness epoch; settle the cold standardisation ramp on one fixed bias-only request.

- Learned-state stimulus: submit a balanced alternating ground-truth stream for that same request, settle labels and observations, and use the acknowledged flush to capture checkpoint $C$. Record the published Core-model probe $P_C$ and require it to be finite, positive on every covariance diagonal, and materially different from the cold probe.

- Branch point: copy the acknowledged checkpoint and empty journal into a second temporary persistence root. In the original world, create one more assessment for the fixed request, stop the model owner, and submit its label so the durable append succeeds while the live enqueue reports owner shutdown, reproducing the established journal fixture.

- New-process clocks: drop the source world and create two distinct `VirtualClock` instances anchored exactly one hour after the source epoch. Each future clock begins a new monotonic domain at zero offset while reporting the same persistent $t_1$; sharing the source clock across the restart would model a monotonic instant surviving a process and would invalidate the witness.

- Decay observation: rebuild the copied, empty-journal branch at $t_1$ and take probe $P_D$ before any label. For every operational, sister, and anchor covariance diagonal, assert $P_D/P_C=1/0.81$ within the generic one-shot tolerance; require the observed ratio to be materially separated from both unity and $1/0.81^2$, and require every mean to equal its checkpoint value.

- Live reference: on the decay-only branch assess the same fixed request at $t_1$, submit a label identical in valence, provenance, eligibility, and action to the durable label, flush, and capture probe $P_L$. The bias-only feature vector and completed ramp make the replayed pending context and the new pending context model-equivalent.

- Replay stimulus: rebuild the original checkpoint-plus-journal branch at $t_1$, call `flush_labels` to wait for replay and its checkpoint, and capture probe $P_R$. Assert that the restored health count includes exactly one label beyond $C$ and that the journal has returned to its header-only form.

- Exact-once assertion: compare the operational, sister, and anchor means and full covariance payloads in $P_R$ and $P_L$ under `World::tol().default`, together with dimensions and floor masses. The decay-only ratio proves the outage was charged; the branch equality proves replay did not charge it again.

- Completion observation: assess the fixed request once on both branches, require their public `rho_eff`, `sigma_eff`, `anchor_weight`, and component probabilities to agree within the same tolerance, and require clean health so matching model probes cannot conceal a degraded recovery path.

The fails-before is a restore that leaves the checkpoint timestamp or its elapsed factor live for replay. The replay branch then scales old precision by another $0.81$ and covariance by another $1/0.81$ before applying the journalled evidence, while the live branch applies the evidence at zero elapsed time; $P_R$ moves toward the $1/0.81^2$ signature and fails the full-state comparison. Omitting restore decay fails the $P_D/P_C$ oracle, and applying the outage after replay ages the new evidence and fails the same branch equality.

## What is missing · `sec:assayer:intent-restore-time-decay-is-applied-exactly-once-missing`

**Entry (Published Core-model probe)** · `entry:assayer:intent-restore-time-decay-is-applied-exactly-once-published-model-probe`

Roughly sixty to ninety lines extend the test-support surface with a `#[doc(hidden)]` `PublishedModelProbe` and `World::published_model_probe()`. The probe takes one immutable published-snapshot load and clones the version, dimensions, means, packed covariances, and floor masses of the operational, sister, and anchor models; it carries no precision matrix, owner handle, clock read, or mutation path. This is the narrow snapshot-diffing slice of (´entry:assayer:harness-stage-tapes´), depends on the existing skeleton, and has no dependency on another intent lane.

**Entry (The once-only restore integration witness)** · `entry:assayer:intent-restore-time-decay-is-applied-exactly-once-integration-witness`

Roughly one hundred and thirty to one hundred and eighty lines in the new persistence integration-test module add the module index, two persistence roots, separate process clocks, learned-state and durable-label setup, decay oracle, live-versus-replay comparison, non-vacuity guards, public-output confirmation, and clean-health assertion. It depends on (´entry:assayer:intent-restore-time-decay-is-applied-exactly-once-published-model-probe´) and the existing `WorldBuilder`, `VirtualClock`, owner-shutdown, barrier, and liveness facilities; it needs neither a general label tape nor another intent lane.

## Risks and open questions · `sec:assayer:intent-restore-time-decay-is-applied-exactly-once-risks`

**Observation (A restart creates a new monotonic epoch)** · `obs:assayer:intent-restore-time-decay-is-applied-exactly-once-monotonic-epoch`

Reusing the source `VirtualClock` after advancing it would carry its monotonic offset across the simulated restart and make the first replayed label appear one hour old even if recovery reset its mark correctly. Two freshly constructed future clocks preserve the persistent instant and discard the source monotonic epoch, which is the process boundary the promise describes.

**Observation (The oracle separates restore from replay)** · `obs:assayer:intent-restore-time-decay-is-applied-exactly-once-two-part-oracle`

Live-versus-replay equality alone would pass if neither branch applied restore decay. The pre-label covariance-ratio assertion makes zero applications fail, while post-label branch equality makes a second application fail; neither assertion substitutes for the other.

**Observation (Only model-equivalent context belongs in the branch comparison)** · `obs:assayer:intent-restore-time-decay-is-applied-exactly-once-context-equivalence`

The replayed pending context was captured at $t_0$ and the live reference context at $t_1$. A completed ramp, no registered Sentinels or identity dimensions, and one bias-only request make their reconstructed model feature vectors identical; the probe compares model state rather than calibration, latency, or pending-buffer diagnostics whose inputs legitimately record different moments.

**Observation (Acknowledgements define every file boundary)** · `obs:assayer:intent-restore-time-decay-is-applied-exactly-once-file-boundaries`

The checkpoint is copied only after `flush_labels` returns, and replay is observed only after the restored branch's flush returns. File polling and sleeps add no evidence: the acknowledgements already order the checkpoint write and journal truncation, while the named liveness deadlines turn a stopped owner into a failed test.

**Observation (One conspicuous configured hour is a discriminating fixture)** · `obs:assayer:intent-restore-time-decay-is-applied-exactly-once-configured-factor`

The shipped rate would move covariance by only a small fraction over one hour and invite a long outage or a loose comparison. The configuration contract permits $0.81$, structural compatibility permits numerical retuning across restore, and the witness names the chosen value as stimulus rather than presenting it as a production default.

No maintainer decision remains open: the records fix the ordering and timestamp domains, the specification fixes the equations and configuration range, and the existing harness fixes the public lifecycle used to create both branches.

## Acceptance · `sec:assayer:intent-restore-time-decay-is-applied-exactly-once-acceptance`

The implementing lane's report identifies the new persistence integration witness, confirms its module index and test documentation cite the intent, and shows the coverage report resolving the intent to that integration test.

The report identifies the test-support probe and confirms that it reads one immutable publication, exposes no precision or mutation path, is documented as hidden, and is absent when test support is disabled.

The reported assertions show a non-cold checkpoint, the configured $1/0.81$ covariance scale and unchanged means on the decay-only restore, a durable label past the checkpoint mark, equality of all three Core model probes after live application and replay, one-label counter advancement, journal truncation, matching public risk fields, and clean health.

The report shows the targeted serde-enabled integration witness and the nearest persistence, snapshot-clock, and decay-arithmetic tests passing under the package's prescribed debug, release, all-feature, formatting, clippy, documentation, and no-default-feature gates; the serde-gated witness is expected to be absent rather than fail in the no-default-feature run.

The report states the deliberately broken second-decay behavior and names the live-versus-replay model comparison that rejects it, together with the decay-only ratio that rejects no restore decay. No executed mutation is required.
