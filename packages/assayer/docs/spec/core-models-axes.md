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
