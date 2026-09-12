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
