// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`new_initialises_to_prior`] | risk | A freshly constructed channel state holds exactly the priors it was given and nothing else: the live counts start equal to the prior counts, which are kept separately so the state always knows how much of itself is belief rather than observation. |
//! | [`update_fail_increments_alpha`] | risk | A challenge the entity failed is one count of evidence that challenges do catch adverse actors, and it moves only that side of the Beta state — the pass count is left exactly where it was. Recording an outcome therefore shifts the estimate without inventing evidence for the other conclusion. |
//! | [`update_pass_increments_beta`] | risk | cites (´claim:risk:a-challenge-outcome-adds-one-count-to-its-own-side-and-leaves-the-other-untouched´) |
//! | [`estimate_returns_posterior_mean`] | risk | The challenge effectiveness a host reads back is the Beta posterior mean — the catch count as a share of all counts — so the prior and the observations are combined by the same arithmetic rather than the prior being a separate special case that has to be blended in later. |
//! | [`estimate_variance_correct`] | risk | The estimate travels with the Beta posterior variance, which is strictly positive and strictly below the variance of a single coin flip at the same mean. That gap is the whole point of accumulating counts: a host reading the pair can tell a well-evidenced catch rate from a barely-evidenced one, which a point estimate alone would hide. |
//! | [`effective_sample_size_computation`] | risk | Effective sample size counts observations and not belief: a state that has seen nothing reports zero however heavy its prior, and fifty recorded outcomes report fifty. Subtracting the prior is what keeps a confident-looking prior from passing itself off as evidence. |
//! | [`sufficient_evidence_threshold`] | risk | Sufficiency is a verdict on the observed count against a host-chosen bar: a state fresh from its prior fails it, and the same state passes once enough real outcomes have accumulated. A host can therefore decline to act on an estimate that is still mostly prior. |
//! | [`time_decay_reduces_effective_sample`] | risk | Time pulls the accumulated counts back toward the prior, so a run of observations left to age is worth less than it was — but never nothing: the effective sample falls and stays positive. Evidence fades rather than being discarded, which is what lets an estimate follow a changing challenge mechanism instead of being pinned by however busy the channel once was. |
//! | [`time_decay_half_life`] | risk | The fade has a calibrated rate rather than an arbitrary one: at the default hourly factor, waiting the algebraic half-life leaves an accumulation worth half what it was. The decay constant a host configures in hours can therefore be reasoned about in months, which is the timescale challenge mechanisms actually change on. |
//! | [`serde_roundtrip`] | risk | Companion state is the host's to persist, so it survives a trip through serialisation with both pseudo-counts and its decay rate intact. Losing the decay rate on restore would be the quiet failure: the counts would look right and would then age at the wrong speed. |
//! | [`tracker_unknown_channel_returns_default_estimate`] | risk | A channel the tracker has never heard of still answers, with the default estimate rather than an absence. Derivation can therefore run from the first request on a new channel, before any challenge has been observed, instead of the host having to seed every channel it might route to. |
//! | [`tracker_update_records_channel_result`] | risk | Recording an outcome on an untracked channel brings that channel's state into existence from the defaults and hands back the estimate the outcome produced, already moved above the neutral half. The host never has to register a channel before reporting on it. |
//! | [`tracker_health_report_contains_channel`] | risk | The Companion reports on itself: every tracked channel appears in the health report with a finite estimate, a finite variance, the count of evidence behind them and the sufficiency verdict at the host's threshold. An operator can see which channels are speaking from evidence and which are still repeating the prior back. |
//! | [`time_decay_preserves_mean_increases_variance_reduces_n_eff`] | risk | Ageing costs confidence, not opinion. Both counts shrink toward their priors at the same rate, so a balanced posterior stays exactly balanced while its variance grows and its effective sample falls — and the variance still stops short of the untouched prior's. Decay is therefore the right shape for a stale channel: the estimate is not silently rewritten, it is merely believed less. |
//! | [`estimate_decays_on_read_without_mutating`] | risk | A read is the posterior at the moment of reading: asked about an idle channel six months on, the estimate comes back with the counts decayed to the present — the same numbers a mutating decay would produce — while the stored state itself has not moved by a bit. An undecayed read is unconservative in exactly one direction, a variance too tight for evidence that has aged; and a read that wrote would let a diagnostic pass erode the evidence it was only supposed to look at. |
//! | [`p_plus_tracker_update_positive`] | risk | The positive-class rate is an exponential average of the label stream: each label moves it toward that label's own indicator by one minus the smoothing factor. An adverse label nudges it up by a fixed fraction of the distance to one, so no single label can redefine what "rare" means. |
//! | [`p_plus_tracker_update_negative`] | risk | cites (´claim:risk:the-positive-class-rate-moves-toward-each-labels-indicator-by-one-minus-the-smoothing-factor´) |
//! | [`p_plus_tracker_clamped`] | risk | However long a run of one class goes on, the rate is held off both endpoints. A thousand consecutive adverse labels leave it just short of one rather than at it — which matters because the importance weight divides by this rate and by its complement, and an endpoint would make one of those a division by zero. |
//! | [`importance_weight_balanced`] | risk | When the two classes arrive equally often there is no imbalance to correct, and both classes come out at unit weight. The correction is therefore inert on balanced data rather than being a constant tax applied regardless. |
//! | [`importance_weight_rare_positive`] | risk | Each weight is the reciprocal of twice its class rate, and the point of that form is what it buys: at any class rate, each class contributes exactly half the total gradient weight — rate times weight is one half on both sides. At a one-in-ten positive rate the adverse label carries 5 and the benign one 5/9, a ratio of nine; the odds ratio would carry 9 against 1/9, squaring the ratio and abandoning the equal-halves property at every rate but one half. The gradient balance, not the ratio, is the quantity the models are owed. |
//! | [`importance_weight_ceiling_applied`] | risk | The upweighting is bounded: at an extreme class rate the raw balancing weight would make one label worth dozens, and the ceiling cuts it to the caller's limit instead. The correction can lift the rare class without letting a handful of labels take over the fit entirely. |
//! | [`eligibility_allow_always_eligible`] | risk | A request that was allowed through runs its course, so whatever happened next is an unconfounded observation of what that entity does — eligible whether or not anyone verified it. This is the population the model can learn from without correction. |
//! | [`eligibility_block_requires_ground_truth`] | risk | Where the index intervened, nobody knows what would have happened: a blocked request produces no outcome to learn from unless the outcome was established independently. Only the ground-truth flag makes such a label eligible. Training on the rest would teach the model that its own interventions work. |
//! | [`eligibility_slow_requires_ground_truth`] | risk | cites (´claim:risk:a-label-from-an-intervened-request-trains-the-model-only-when-the-outcome-was-independently-verified´) |
//! | [`eligibility_challenge_follows_policy`] | risk | A challenge is not a block: the request proceeded, and by default its label trains the models, because the default policy asserts that failing a challenge is a property of the request rather than an effect of having been challenged. A deployment that disputes that sets the policy false, and then a non-ground-truth challenge label is excluded — the predicate defers to the declared policy rather than hard-coding either judgement. |
//! | [`challenge_risk_target_positive_valence`] | risk | The risk target the model trains against is a sign, not a magnitude: a strongly adverse observation and a mildly adverse one both become the same positive target. How bad an outcome was is the host's business; whether it was adverse is all the model is asked to learn. |
//! | [`challenge_risk_target_negative_valence`] | risk | The benign side collapses the same way: any negative valence, slight or severe, becomes the single negative target. The mapping is a two-way split of the real line, so the model's training signal has exactly two values. |
//! | [`risk_target_zero_valence`] | risk | cites (´claim:risk:every-non-adverse-valence-collapses-to-the-single-negative-target´) |
//! | [`tracker_override_replaces_estimate_until_cleared`] | risk | A host override stands in front of the posterior rather than replacing it: while set, every read returns the override, and clearing it hands the override back and reveals the Beta estimate that was accumulating underneath all along. An operator who knows a challenge has been defeated can say so immediately and then step back without having discarded the channel's evidence. |
//! | [`tracker_inject_evidence_clamps_to_ceiling`] | risk | Evidence a host imports from outside is capped at the channel's ceiling before it reaches the Beta state: an injection of five against a ceiling of two adds two, and the resulting estimate and effective sample both reflect the capped amount. One import cannot claim more authority than the channel's owner allowed it, however large the number handed in. |
//! | [`tracker_inject_evidence_default_ceiling_is_one_thousand`] | risk | A channel created on demand by an injection carries a default ceiling of a thousand pseudo-counts, so five thousand offered lands as a thousand recorded. The cap is a property of the default state and not something a host must remember to configure before the first import. |
//! | [`tracker_inject_evidence_rejects_negative_counts`] | risk | A count that is not a finite non-negative number is refused, and refused early: the channel it named is not brought into existence by the attempt. A rejected import leaves the tracker exactly as it was, rather than seeding an empty channel that a later read would answer from. |
//! | [`replacement_trait_default_posterior_moment_matches`] | risk | A provider of a scalar and a variance participates in the posterior machinery without implementing anything further: the surface's default posterior is a moment-matched Beta that reproduces the provider's mean and variance exactly, and a pair no Beta can carry falls back to the uniform prior rather than to an invented shape. Without this default, every scalar provider would have to fork the shipped tracker to be heard at all. |
//! | [`shipped_tracker_implements_the_replacement_surface`] | risk | The shipped tracker implements the replacement surface, and as the holder of a genuine conjugate posterior it answers with its exact decayed counts rather than a moment match: after one observed failure on uniform priors the posterior is Beta(2, 1), the trait's estimate agrees with the tracker's own read, and an unknown channel answers from the uniform prior. This is what makes the tracker substitutable — a host implementing against the trait gets the same answers the concrete type gives. |

//! Host-owned Companion Tracker for challenge effectiveness.
//!
//! The Companion Tracker estimates challenge effectiveness per channel using a
//! conjugate Beta-Binomial model with time-indexed pseudo-count decay. It is
//! independent of the Core model snapshot and label pipeline: hosts feed it
//! observed challenge outcomes, optional external evidence, or overrides, then
//! pass its [`ChallengeEstimate`] output to `derive_reckoning()` explicitly.
//!
//! # Model
//!
//! The model estimates challenge effectiveness `q_c` — the probability that
//! a challenge correctly identifies adverse actors. Uses a Beta prior
//! `Beta(α_0, β_0)` with conjugate updating:
//!
//! - Challenge failed → α += 1 (adverse actor correctly caught)
//! - Challenge passed → β += 1 (benign actor correctly released)
//!
//! Time decay blends the posterior toward the prior, preventing stale
//! data from dominating current estimates.
//!
//! # Cross-References
//!
//! - (´tab:eligibility:training´) — which observation trains which model, the
//!   contract that stands without a challenge result
//! - (´chap:spec:challenge-effectiveness´) — the Companion Tracker
//! - (´sig:companion:posterior´) — the `ChallengeEstimate` that crosses the
//!   derivation boundary

#![allow(clippy::doc_markdown)]

use std::collections::HashMap;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::types::{ChallengeEstimate, ChallengeResult, ChannelId, PersistentTimestamp};

// ═══════════════════════════════════════════════════════════════════════════════
// Challenge Effectiveness State
// ═══════════════════════════════════════════════════════════════════════════════

/// Host-owned state for tracking challenge effectiveness for one channel.
///
/// Uses a Beta-Binomial conjugate model with time-indexed pseudo-count decay.
/// The model estimates `q_c` — the probability that a challenge correctly
/// identifies adverse actors. The Core never stores this state; hosts own its
/// persistence and decide whether multiple channels share one state.
///
/// # Time Decay
///
/// Pseudo-counts decay toward the prior over time:
/// ```text
/// α ← α_0 + (α - α_0) × decay_factor(Δt)
/// β ← β_0 + (β - β_0) × decay_factor(Δt)
/// ```
///
/// With `γ_qt ≈ 0.9998` (~145 day half-life), old observations fade
/// while the prior remains stable.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ChallengeEffectivenessState {
    /// Current alpha (failure count + prior: adverse actors correctly caught).
    pub alpha: f64,
    /// Current beta (pass count + prior: benign actors correctly released).
    pub beta: f64,
    /// Timestamp of last update (for decay computation).
    pub t_last: PersistentTimestamp,
    /// Prior alpha (initial pseudo-count for failures).
    pub alpha_0: f64,
    /// Prior beta (initial pseudo-count for passes).
    pub beta_0: f64,
    /// Hourly decay factor γ_qt (´def:companion:challenge-decay´).
    pub gamma_qt: f64,
    /// Maximum pseudo-count magnitude accepted in one evidence injection.
    pub injection_ceiling: f64,
    /// Active host override, if any.
    pub override_active: Option<ChallengeOverride>,
}

/// Host-provided challenge-effectiveness override.
///
/// While present, [`ChallengeEffectivenessState::estimate`] returns this value
/// instead of the Beta-Binomial posterior. The underlying Beta state continues
/// to accept observed and injected evidence (´alg:companion:override´).
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ChallengeOverride {
    /// Overridden challenge catch-rate estimate `q̂_c`.
    pub q_c: f64,
    /// Overridden posterior variance `σ²(q̂_c)`.
    pub variance: f64,
}

impl ChallengeOverride {
    /// Creates a host override, using the spec default variance when omitted.
    #[must_use]
    pub fn new(q_c: f64, variance: Option<f64>) -> Self {
        let q_c = if q_c.is_finite() { q_c.clamp(0.0, 1.0) } else { 0.5 };
        let variance = variance
            .filter(|value| value.is_finite() && *value >= 0.0)
            .unwrap_or_else(|| q_c * (1.0 - q_c) / 101.0);

        Self { q_c, variance }
    }
}

impl From<ChallengeOverride> for ChallengeEstimate {
    fn from(value: ChallengeOverride) -> Self {
        // In domain by construction: `ChallengeOverride::new` clamps the
        // mean into [0, 1] and refuses a negative or non-finite variance.
        Self::new_unchecked(value.q_c, value.variance)
    }
}

/// Error returned when injected challenge evidence is invalid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ChallengeEvidenceError {
    /// Injection pseudo-counts must be finite and non-negative (´alg:companion:injection´).
    InvalidInjection,
}

impl std::fmt::Display for ChallengeEvidenceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInjection => formatter.write_str("challenge evidence injection counts must be finite and non-negative"),
        }
    }
}

impl std::error::Error for ChallengeEvidenceError {}

impl ChallengeEffectivenessState {
    /// Default prior alpha, the uniform Beta(1,1) prior
    /// (´def:companion:prior´).
    ///
    /// ´const:assayer:uniform-prior-alpha´ (´alg:const:scalar´)
    /// ´const:assayer:uniform-prior-alpha-scalar-1p0´
    pub const DEFAULT_ALPHA_0: f64 = 1.0;

    /// Default prior beta, the uniform Beta(1,1) prior
    /// (´def:companion:prior´).
    ///
    /// ´const:assayer:uniform-prior-beta´ (´alg:const:scalar´)
    /// ´const:assayer:uniform-prior-beta-scalar-1p0´
    pub const DEFAULT_BETA_0: f64 = 1.0;

    /// Default hourly decay factor (~145 day half-life).
    ///
    /// Computed as: `0.9998^(24*145) ≈ 0.5`
    ///
    /// The rate is the Companion's own and is independent of every Core
    /// forgetting rate (´alg:companion:decay´).
    ///
    /// ´const:assayer:challenge-evidence-decay-rate´ (´alg:const:scalar´)
    /// ´const:assayer:challenge-evidence-decay-rate-scalar-0p9998´
    pub const DEFAULT_GAMMA_QT: f64 = 0.9998;

    /// Default maximum pseudo-count magnitude for one evidence injection
    /// (´alg:companion:injection´).
    ///
    /// ´const:assayer:evidence-injection-ceiling´ (´alg:const:scalar´)
    /// ´const:assayer:evidence-injection-ceiling-scalar-1000p0´
    pub const DEFAULT_INJECTION_CEILING: f64 = 1000.0;

    /// Creates a new challenge effectiveness state with specified priors.
    ///
    /// # Arguments
    ///
    /// * `alpha_0` — Prior alpha (success pseudo-count)
    /// * `beta_0` — Prior beta (failure pseudo-count)
    /// * `gamma_qt` — Hourly decay factor
    #[must_use]
    pub fn new(alpha_0: f64, beta_0: f64, gamma_qt: f64) -> Self {
        Self {
            alpha: alpha_0,
            beta: beta_0,
            t_last: PersistentTimestamp::now(),
            alpha_0,
            beta_0,
            gamma_qt,
            injection_ceiling: Self::DEFAULT_INJECTION_CEILING,
            override_active: None,
        }
    }

    /// Creates a new challenge effectiveness state with default priors.
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(Self::DEFAULT_ALPHA_0, Self::DEFAULT_BETA_0, Self::DEFAULT_GAMMA_QT)
    }

    /// Applies time decay, blending toward the prior.
    ///
    /// # Arguments
    ///
    /// * `now` — Current timestamp
    ///
    /// # Time Decay Formula
    ///
    /// ```text
    /// Δh = hours_elapsed(t_last, now)
    /// decay = γ_qt^Δh
    /// α ← α_0 + (α - α_0) × decay
    /// β ← β_0 + (β - β_0) × decay
    /// ```
    pub fn apply_time_decay(&mut self, now: &PersistentTimestamp) {
        let decay = crate::numerics::decay_factor_since(self.gamma_qt, &self.t_last, now);
        if (1.0 - decay).abs() < f64::EPSILON {
            return;
        }

        // Blend toward prior (´dec:companion:decay-form´)
        self.alpha = (self.alpha - self.alpha_0).mul_add(decay, self.alpha_0);
        self.beta = (self.beta - self.beta_0).mul_add(decay, self.beta_0);
        self.t_last = *now;
    }

    /// Updates the state with a host-observed challenge result
    /// (´alg:companion:update´).
    ///
    /// Applies time decay first, then performs the conjugate update:
    /// - `Fail` → α += 1 (challenge correctly identified adverse actor)
    /// - `Pass` → β += 1 (challenge correctly released benign actor)
    pub fn update(&mut self, result: ChallengeResult, now: &PersistentTimestamp) {
        self.apply_time_decay(now);

        match result {
            ChallengeResult::Fail => self.alpha += 1.0,
            ChallengeResult::Pass => self.beta += 1.0,
        }
    }

    /// Sets or replaces the host override (´alg:companion:override´).
    ///
    /// The Beta-Binomial state remains live while the override is active.
    /// Future [`update`](Self::update) and [`inject_evidence`](Self::inject_evidence)
    /// calls still mutate the underlying pseudo-counts.
    pub fn set_override(&mut self, q_c: f64, variance: Option<f64>) -> ChallengeEstimate {
        let override_value = ChallengeOverride::new(q_c, variance);
        self.override_active = Some(override_value);
        override_value.into()
    }

    /// Clears the active host override, returning it when present (´alg:companion:override´).
    pub const fn clear_override(&mut self) -> Option<ChallengeOverride> {
        self.override_active.take()
    }

    /// Injects external evidence as pseudo-counts (´alg:companion:injection´).
    ///
    /// Both counts must be finite and non-negative. Each count is clamped to
    /// [`Self::injection_ceiling`] before it is added to the Beta state.
    ///
    /// # Errors
    ///
    /// Returns [`ChallengeEvidenceError::InvalidInjection`] when either count
    /// is negative, infinite, or NaN.
    pub fn inject_evidence(
        &mut self,
        failures: f64,
        passes: f64,
        now: &PersistentTimestamp,
    ) -> Result<ChallengeEstimate, ChallengeEvidenceError> {
        if !failures.is_finite() || !passes.is_finite() || failures < 0.0 || passes < 0.0 {
            return Err(ChallengeEvidenceError::InvalidInjection);
        }

        self.apply_time_decay(now);
        let ceiling = self.injection_ceiling;
        self.alpha += Self::clamp_injected_count("failures", failures, ceiling);
        self.beta += Self::clamp_injected_count("passes", passes, ceiling);

        let (q_c, variance) = self.estimate(now);
        // In domain by construction: a Beta posterior's mean and variance,
        // or a sanitised override.
        Ok(ChallengeEstimate::new_unchecked(q_c, variance))
    }

    /// Clamps one injected pseudo-count and warns when the host exceeds the ceiling.
    fn clamp_injected_count(kind: &'static str, value: f64, ceiling: f64) -> f64 {
        if value <= ceiling {
            return value;
        }

        tracing::warn!(kind, value, ceiling, "challenge evidence injection clamped to ceiling");
        ceiling
    }

    /// Returns the pseudo-counts decayed to `now` as a pure function of
    /// elapsed time, mutating nothing — a read must not be a write
    /// (´alg:companion:inference´).
    #[must_use]
    fn decayed_counts(&self, now: &PersistentTimestamp) -> (f64, f64) {
        let decay = crate::numerics::decay_factor_since(self.gamma_qt, &self.t_last, now);
        (
            (self.alpha - self.alpha_0).mul_add(decay, self.alpha_0),
            (self.beta - self.beta_0).mul_add(decay, self.beta_0),
        )
    }

    /// Returns the estimate `q̂_c` and variance, the posterior at the
    /// moment of reading (´alg:companion:inference´).
    ///
    /// Active host overrides take precedence over the Beta posterior. Use
    /// [`Self::beta_estimate`] when the raw pseudo-count estimate is needed.
    ///
    /// # Returns
    ///
    /// `(q̂_c, σ²_q̂c)` where:
    /// - `q̂_c = α / (α + β)` — posterior mean
    /// - `σ²_q̂c = αβ / ((α+β)²(α+β+1))` — posterior variance
    #[must_use]
    pub fn estimate(&self, now: &PersistentTimestamp) -> (f64, f64) {
        if let Some(override_value) = self.override_active {
            return (override_value.q_c, override_value.variance);
        }

        self.beta_estimate(now)
    }

    /// Returns the raw Beta posterior mean and variance at the moment of
    /// reading, ignoring overrides.
    #[must_use]
    // Justified: σ² = αβ / ((α+β)²(α+β+1)) — the squared sum is the formula,
    // not a mistyped pairing.
    #[allow(clippy::suspicious_operation_groupings)]
    pub fn beta_estimate(&self, now: &PersistentTimestamp) -> (f64, f64) {
        let (alpha, beta) = self.decayed_counts(now);
        let sum = alpha + beta;
        if sum <= 0.0 {
            return (0.5, 0.25); // Fallback for degenerate case
        }

        let mean = alpha / sum;
        let variance = (alpha * beta) / (sum * sum * (sum + 1.0));

        (mean, variance)
    }

    /// Returns the effective sample size (observations beyond prior) at
    /// the moment of reading.
    ///
    /// ```text
    /// n_eff = (α + β) - (α_0 + β_0)
    /// ```
    #[must_use]
    pub fn effective_sample_size(&self, now: &PersistentTimestamp) -> f64 {
        let (alpha, beta) = self.decayed_counts(now);
        (alpha + beta) - (self.alpha_0 + self.beta_0)
    }

    /// Returns `true` if there is sufficient evidence for estimates.
    ///
    /// # Arguments
    ///
    /// * `threshold` — Minimum effective sample size required
    /// * `now` — The moment of reading
    #[must_use]
    pub fn sufficient_evidence(&self, threshold: f64, now: &PersistentTimestamp) -> bool {
        self.effective_sample_size(now) >= threshold
    }
}

impl Default for ChallengeEffectivenessState {
    fn default() -> Self {
        Self::with_defaults()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Standalone Companion Tracker
// ═══════════════════════════════════════════════════════════════════════════════

/// Host-owned Companion Tracker for per-channel challenge effectiveness.
///
/// This component sits outside the Core model snapshot and label pipeline.
/// Hosts update it from observed challenge outcomes, inject external evidence
/// or set overrides, then pass the resulting [`ChallengeEstimate`] to
/// `derive_reckoning()` as an explicit derivation input
/// (´sig:companion:posterior´). The Companion's own boundary is what keeps it
/// outside the Core (´inv:companion:boundary´), and durable Companion state
/// belongs to host-managed persistence.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ChallengeEffectivenessTracker {
    /// Per-channel Beta-Binomial states.
    states: HashMap<ChannelId, ChallengeEffectivenessState>,
}

impl ChallengeEffectivenessTracker {
    /// Default effective-sample floor at which a channel's estimate is
    /// reported as sufficiently evidenced, in effective samples beyond
    /// the prior, matching what [`Self::health_report`] expects.
    ///
    /// This is the host's figure and no part of Core. It once stood in
    /// the Core convergence construction configuration, where its own
    /// source's dated notice had asked since 2026-05-17 that it not be,
    /// and it moved here at the fifty it shipped with while the value
    /// question stayed open. That question is closed at twenty, the
    /// figure this corpus already names twice — the specification's own
    /// Companion convergence target and the warm-up ladder's
    /// practically-useful-variance milestone — where fifty answered to
    /// no anchor at all (´cav:challenge:sufficiency-threshold-open´).
    ///
    /// Twenty is a floor the shipped forgetting rate lets a weakly,
    /// moderately or well targeted channel hold. An untargeted channel,
    /// at the specified eight hundredths of a contributing label a day,
    /// settles at about 17.17 effective samples and never reaches it.
    /// That residue is accepted: the floor reports thin evidence
    /// honestly rather than being lowered to flatter it
    /// (´alg:companion:decay´).
    ///
    /// ´const:assayer:companion-sufficiency-floor´ (´alg:const:scalar´)
    /// ´const:assayer:companion-sufficiency-floor-scalar-20p0´
    pub const DEFAULT_SUFFICIENT_EVIDENCE_THRESHOLD: f64 = 20.0;

    // TODO ´todo:code:only-tests-construct-the-challenge-effectiveness´: Only tests construct the challenge effectiveness tracker. Every caller
    // of this constructor in the workspace is a test or a test-support fixture:
    // nothing on the assessment or label path builds one or reads its estimate, so
    // the Companion model this type implements reaches no decision the package
    // makes, and the whole module is exercised only against itself.
    /// Creates an empty challenge-effectiveness tracker.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the number of tracked channels.
    #[must_use]
    pub fn len(&self) -> usize {
        self.states.len()
    }

    /// Returns `true` when no channels are tracked.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }

    /// Registers or replaces the state for a channel.
    ///
    /// Returns the previous state if the channel was already present.
    pub fn insert_channel(
        &mut self,
        channel_id: ChannelId,
        state: ChallengeEffectivenessState,
    ) -> Option<ChallengeEffectivenessState> {
        self.states.insert(channel_id, state)
    }

    /// Removes the state for a channel.
    ///
    /// Returns the removed state when one existed.
    pub fn remove_channel(&mut self, channel_id: ChannelId) -> Option<ChallengeEffectivenessState> {
        self.states.remove(&channel_id)
    }

    /// Returns the derivation estimate for a channel at the moment of
    /// reading (´sig:companion:posterior´): the pseudo-counts are decayed to `now`
    /// into locals without mutating anything (´alg:companion:inference´).
    ///
    /// Unknown channels return [`ChallengeEstimate::default`], allowing hosts
    /// to derive assessments before any challenge outcomes have been observed.
    #[must_use]
    pub fn estimate(&self, channel_id: ChannelId, now: &PersistentTimestamp) -> ChallengeEstimate {
        self.states
            .get(&channel_id)
            .map_or_else(ChallengeEstimate::default, |state| Self::estimate_from_state(state, now))
    }

    /// Updates a channel from an observed challenge outcome and returns its new estimate.
    ///
    /// Unknown channels are initialised with [`ChallengeEffectivenessState::default`].
    pub fn update(&mut self, channel_id: ChannelId, result: ChallengeResult, now: &PersistentTimestamp) -> ChallengeEstimate {
        let state = self.states.entry(channel_id).or_default();
        state.update(result, now);
        Self::estimate_from_state(state, now)
    }

    /// Sets or replaces a channel's host override (´alg:companion:override´).
    pub fn set_override(&mut self, channel_id: ChannelId, q_c: f64, variance: Option<f64>) -> ChallengeEstimate {
        self.states.entry(channel_id).or_default().set_override(q_c, variance)
    }

    /// Clears a channel's host override, returning it when present (´alg:companion:override´).
    pub fn clear_override(&mut self, channel_id: ChannelId) -> Option<ChallengeOverride> {
        self.states
            .get_mut(&channel_id)
            .and_then(ChallengeEffectivenessState::clear_override)
    }

    /// Injects external evidence into a channel's Beta state (´alg:companion:injection´).
    ///
    /// # Errors
    ///
    /// Returns [`ChallengeEvidenceError::InvalidInjection`] when either count
    /// is negative, infinite, or NaN.
    pub fn inject_evidence(
        &mut self,
        channel_id: ChannelId,
        failures: f64,
        passes: f64,
        now: &PersistentTimestamp,
    ) -> Result<ChallengeEstimate, ChallengeEvidenceError> {
        if !failures.is_finite() || !passes.is_finite() || failures < 0.0 || passes < 0.0 {
            return Err(ChallengeEvidenceError::InvalidInjection);
        }

        self.states
            .entry(channel_id)
            .or_default()
            .inject_evidence(failures, passes, now)
    }

    /// Builds a standalone Companion health report at the moment of
    /// reading (´tab:companion:health´), through the same clocked read path as
    /// [`Self::estimate`].
    ///
    /// `sufficient_evidence_threshold` is expressed in effective samples beyond
    /// the prior, matching [`ChallengeEffectivenessState::sufficient_evidence`].
    #[must_use]
    pub fn health_report(&self, sufficient_evidence_threshold: f64, now: &PersistentTimestamp) -> ChallengeHealthReport {
        let channels = self
            .states
            .iter()
            .map(|(&channel_id, state)| {
                let (q_hat, variance) = state.estimate(now);
                let override_values = state.override_active.map(ChallengeEstimate::from);
                (
                    channel_id,
                    ChallengeHealthDetail {
                        q_hat,
                        variance,
                        effective_sample_size: state.effective_sample_size(now),
                        sufficient_evidence: state.sufficient_evidence(sufficient_evidence_threshold, now),
                        override_active: state.override_active.is_some(),
                        override_values,
                    },
                )
            })
            .collect();

        ChallengeHealthReport { channels }
    }

    /// Converts a Beta-Binomial state into the derivation estimate type.
    fn estimate_from_state(state: &ChallengeEffectivenessState, now: &PersistentTimestamp) -> ChallengeEstimate {
        let (q_c, variance) = state.estimate(now);
        // In domain by construction: a Beta posterior's mean and variance,
        // or a sanitised override.
        ChallengeEstimate::new_unchecked(q_c, variance)
    }
}

/// Standalone health report for a host-owned Companion Tracker (´tab:companion:health´).
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ChallengeHealthReport {
    /// Per-channel challenge health details.
    pub channels: HashMap<ChannelId, ChallengeHealthDetail>,
}

/// Health details for one challenge channel.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ChallengeHealthDetail {
    /// Challenge effectiveness point estimate `q̂_c` (´sig:companion:posterior´).
    pub q_hat: f64,
    /// Posterior variance `σ²(q̂_c)` (´sig:companion:posterior´).
    pub variance: f64,
    /// Effective observations beyond the prior.
    pub effective_sample_size: f64,
    /// Whether the channel has enough effective observations for operational use.
    pub sufficient_evidence: bool,
    /// Whether a host override is currently active (´alg:companion:override´).
    pub override_active: bool,
    /// Active override values, when present (´alg:companion:override´).
    pub override_values: Option<ChallengeEstimate>,
}

impl Default for ChallengeHealthDetail {
    fn default() -> Self {
        let estimate = ChallengeEstimate::default();
        Self {
            q_hat: estimate.q_c,
            variance: estimate.variance,
            effective_sample_size: 0.0,
            sufficient_evidence: false,
            override_active: false,
            override_values: None,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// The Replacement Surface
// ═══════════════════════════════════════════════════════════════════════════════

/// A challenge-effectiveness posterior as its two pseudo-counts.
///
/// The carrier the inference algorithm speaks of
/// (´alg:companion:inference´): a Beta posterior is its failure count α
/// and its pass count β, and a consumer holding the counts can evaluate
/// exact quantiles where a moment pair supports only first-order reads.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ChallengePosterior {
    /// Failure pseudo-count α (adverse actors correctly caught).
    pub alpha: f64,
    /// Pass pseudo-count β (benign actors correctly released).
    pub beta: f64,
}

impl ChallengePosterior {
    /// The uniform prior Beta(1, 1) (´def:companion:prior´).
    #[must_use]
    pub const fn uniform_prior() -> Self {
        Self { alpha: 1.0, beta: 1.0 }
    }

    /// Creates a posterior from its two pseudo-counts.
    #[must_use]
    pub const fn new(alpha: f64, beta: f64) -> Self {
        Self { alpha, beta }
    }

    /// Moment-matches a Beta posterior to a scalar estimate.
    ///
    /// For a mean `m` strictly inside the unit interval and a variance
    /// `v` strictly positive with `v` below `m(1 − m)` — the domain the
    /// replacement surface requires of a provider
    /// (´req:companion:replacement-trait´) — the matched Beta has
    /// `ν = m(1 − m)/v − 1`, `α = mν`, `β = (1 − m)ν`, and reproduces
    /// the estimate's mean and variance exactly. A pair outside that
    /// domain has no matching Beta, and the match falls back to the
    /// uniform prior (´def:companion:prior´); a provider holding such a
    /// pair should override the posterior method instead.
    #[must_use]
    pub fn moment_matched(estimate: ChallengeEstimate) -> Self {
        let m = estimate.q_c();
        let v = estimate.variance();
        if m <= 0.0 || m >= 1.0 || v <= 0.0 {
            return Self::uniform_prior();
        }
        let nu = m * (1.0 - m) / v - 1.0;
        if nu <= 0.0 || !nu.is_finite() {
            return Self::uniform_prior();
        }
        Self {
            alpha: m * nu,
            beta: (1.0 - m) * nu,
        }
    }

    /// The posterior mean `α / (α + β)`.
    #[must_use]
    pub fn mean(&self) -> f64 {
        self.alpha / (self.alpha + self.beta)
    }

    /// The posterior variance `αβ / ((α + β)²(α + β + 1))`.
    #[must_use]
    // Justified: σ² = αβ / ((α+β)²(α+β+1)) — the squared sum is the
    // formula, not a mistyped pairing.
    pub fn variance(&self) -> f64 {
        let sum = self.alpha + self.beta;
        (self.alpha * self.beta) / (sum * sum * (sum + 1.0))
    }
}

impl From<ChallengePosterior> for crate::types::ChallengePosteriorInput {
    /// The Companion's conjugate posterior crosses the derivation
    /// boundary as itself (´sig:companion:posterior´): the pseudo-count
    /// pair survives the crossing, so credible quantiles and dominance
    /// probabilities are evaluated on the exact shape.
    fn from(posterior: ChallengePosterior) -> Self {
        Self::Conjugate {
            alpha: posterior.alpha,
            beta: posterior.beta,
        }
    }
}

/// The replacement surface (´req:companion:replacement-trait´).
///
/// This public trait is exported from the crate root, and the shipped
/// [`ChallengeEffectivenessTracker`] implements it below.
///
/// (´claim:risk:the-shipped-tracker-answers-the-replacement-surface-with-its-exact-posterior-counts´)
///
/// The Beta-Binomial tracker is one estimator of challenge
/// effectiveness, and a replacement owes no more than this interface: a
/// method returning the scalar estimate, and a posterior method
/// defaulting to a moment-matched Beta built from that estimate. A
/// provider of a scalar and a variance participates at first-order
/// fidelity without implementing anything further; a provider holding a
/// genuine conjugate posterior overrides the second method and gets
/// exact quantiles. Neither the Core nor the derivation is affected by
/// which provider is in place (´inv:companion:boundary´).
pub trait ChallengeEffectivenessProvider {
    /// The scalar estimate for a channel at the moment of reading.
    fn challenge_estimate(&self, channel_id: ChannelId, now: &PersistentTimestamp) -> ChallengeEstimate;

    /// The posterior for a channel, defaulting to a moment-matched Beta
    /// built from the scalar estimate.
    fn challenge_posterior(&self, channel_id: ChannelId, now: &PersistentTimestamp) -> ChallengePosterior {
        ChallengePosterior::moment_matched(self.challenge_estimate(channel_id, now))
    }
}

impl ChallengeEffectivenessProvider for ChallengeEffectivenessTracker {
    fn challenge_estimate(&self, channel_id: ChannelId, now: &PersistentTimestamp) -> ChallengeEstimate {
        self.estimate(channel_id, now)
    }

    /// The shipped tracker holds a genuine conjugate posterior, so it
    /// overrides the default with its exact decayed counts. An active
    /// override is a scalar pair and moment-matches; an unknown channel
    /// answers from the uniform prior, agreeing with
    /// [`Self::estimate`]'s default.
    fn challenge_posterior(&self, channel_id: ChannelId, now: &PersistentTimestamp) -> ChallengePosterior {
        self.states
            .get(&channel_id)
            .map_or_else(ChallengePosterior::uniform_prior, |state| {
                if state.override_active.is_some() {
                    ChallengePosterior::moment_matched(Self::estimate_from_state(state, now))
                } else {
                    let (alpha, beta) = state.decayed_counts(now);
                    ChallengePosterior::new(alpha, beta)
                }
            })
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Importance Weighting Helpers
// ═══════════════════════════════════════════════════════════════════════════════

/// Updates a P+ (positive-class prior) tracker with exponential smoothing.
///
/// # Arguments
///
/// * `tracker` — Current P+ estimate (modified in place)
/// * `gamma` — Smoothing factor (0 < γ < 1; higher = slower adaptation)
/// * `is_positive` — Whether this label is positive (adverse)
///
/// # Formula
///
/// ```text
/// P+_new = γ × P+_old + (1 - γ) × I(positive)
/// ```
///
/// # Cross-References
///
/// - (´alg:weighting:tracker-update´) — the class-rate tracker update
pub fn update_p_plus_tracker(tracker: &mut f64, gamma: f64, is_positive: bool) {
    let indicator = if is_positive { 1.0 } else { 0.0 };
    *tracker = (1.0 - gamma).mul_add(indicator, gamma * *tracker);

    // Clamp to valid range (prevents drift to exactly 0 or 1)
    *tracker = tracker.clamp(0.001, 0.999);
}

/// Computes the balancing importance weight for a label.
///
/// Each weight is the reciprocal of twice the relevant class rate, capped
/// at the ceiling (´def:weighting:balancing-weights´):
///
/// ```text
/// w = min(ceiling,
///         if positive: 1 / (2·P+)
///         else:        1 / (2·(1 − P+)))
/// ```
///
/// The defining property of this form is that each class contributes
/// exactly half the total gradient weight at any class rate, until the
/// ceiling binds.
///
/// # Arguments
///
/// * `p_plus` — Current estimate of positive-class frequency
/// * `is_positive` — Whether this label is positive (adverse)
/// * `ceiling` — Maximum importance weight (for stability)
#[must_use]
pub fn compute_importance_weight(p_plus: f64, is_positive: bool, ceiling: f64) -> f64 {
    // The clamp bounds both denominators away from zero, standing in for
    // the definition's epsilon guard.
    let p = p_plus.clamp(0.001, 0.999);

    let weight = if is_positive {
        1.0 / (2.0 * p)
    } else {
        1.0 / (2.0 * (1.0 - p))
    };

    weight.min(ceiling)
}

/// Determines label eligibility for sister and anchor training
/// (´tab:eligibility:training´).
///
/// The operational model trains on every labelled outcome; this predicate
/// decides which labels also train the inherent-risk models.
///
/// # Arguments
///
/// * `action` — The action taken (Allow, Challenge, Slow, Block)
/// * `policy` — The host's declared eligibility policy
///   (´req:host:eligibility-policy´)
/// * `ground_truth` — Whether the outcome was established by investigation
///
/// # Returns
///
/// `true` if the label is eligible for sister and anchor updates.
#[must_use]
pub const fn determine_eligibility(
    action: crate::types::Action,
    policy: crate::config::types::EligibilityPolicy,
    ground_truth: bool,
) -> bool {
    use crate::types::Action;

    // A host-investigated outcome is ground truth whatever the action
    // (´conv:eligibility:ground-truth´).
    if ground_truth {
        return true;
    }

    match action {
        // Unconfounded: the request ran its course.
        Action::Allow => true,
        // Policy-governed. Pass or fail is not visible to the Core, so the
        // policy's failed-challenge row applies to every non-ground-truth
        // challenge label.
        Action::Challenge => policy.challenge_fail_is_unconfounded,
        // Causally confounded: the host intervened.
        Action::Slow | Action::Block => false,
    }
}

/// Computes the risk target `r_ρ` from valence.
///
/// The risk target is `r_ρ = 2·𝟙[v > 0] − 1 ∈ {−1, +1}`:
/// - Positive valence (adverse) → r_ρ = +1
/// - Non-positive valence (benign or zero) → r_ρ = −1
///
/// # Cross-References
///
/// - (´def:risk:target´) — the risk target this computes
#[must_use]
pub fn compute_risk_target(valence: f64) -> f64 {
    if valence > 0.0 { 1.0 } else { -1.0 }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ─────────────────────────────────────────────────────────────────────────
    // Challenge Effectiveness Tests
    // ─────────────────────────────────────────────────────────────────────────

    /// A freshly constructed channel state holds exactly the priors it was given
    /// and nothing else: the live counts start equal to the prior counts, which
    /// are kept separately so the state always knows how much of itself is belief
    /// rather than observation.
    ///
    /// ´claim:risk:a-fresh-challenge-state-holds-only-the-prior-pseudo-counts-it-was-given´
    /// ´test:unit:new-initialises-to-prior´
    #[test]
    fn new_initialises_to_prior() {
        let state = ChallengeEffectivenessState::new(3.0, 2.0, 0.999);
        assert!((state.alpha - 3.0).abs() < f64::EPSILON);
        assert!((state.beta - 2.0).abs() < f64::EPSILON);
        assert!((state.alpha_0 - 3.0).abs() < f64::EPSILON);
        assert!((state.beta_0 - 2.0).abs() < f64::EPSILON);
    }

    /// A challenge the entity failed is one count of evidence that challenges do
    /// catch adverse actors, and it moves only that side of the Beta state — the
    /// pass count is left exactly where it was. Recording an outcome therefore
    /// shifts the estimate without inventing evidence for the other conclusion.
    ///
    /// ´claim:risk:a-challenge-outcome-adds-one-count-to-its-own-side-and-leaves-the-other-untouched´
    /// ´test:unit:update-fail-increments-alpha´
    #[test]
    fn update_fail_increments_alpha() {
        let mut state = ChallengeEffectivenessState::new(2.0, 2.0, 0.9998);
        let now = state.t_last; // Same time, no decay
        state.update(ChallengeResult::Fail, &now);
        assert!((state.alpha - 3.0).abs() < f64::EPSILON);
        assert!((state.beta - 2.0).abs() < f64::EPSILON);
    }

    /// The mirror case: a challenge the entity passed lands entirely on the
    /// benign-release count, and the adverse-catch count is unchanged. The two
    /// outcomes are symmetric bookkeeping on one shared state.
    ///
    /// (´claim:risk:a-challenge-outcome-adds-one-count-to-its-own-side-and-leaves-the-other-untouched´)
    /// ´test:unit:update-pass-increments-beta´
    #[test]
    fn update_pass_increments_beta() {
        let mut state = ChallengeEffectivenessState::new(2.0, 2.0, 0.9998);
        let now = state.t_last;
        state.update(ChallengeResult::Pass, &now);
        assert!((state.alpha - 2.0).abs() < f64::EPSILON);
        assert!((state.beta - 3.0).abs() < f64::EPSILON);
    }

    /// The challenge effectiveness a host reads back is the Beta posterior mean —
    /// the catch count as a share of all counts — so the prior and the observations
    /// are combined by the same arithmetic rather than the prior being a separate
    /// special case that has to be blended in later.
    ///
    /// ´claim:risk:the-challenge-estimate-is-the-beta-posterior-mean-of-the-two-pseudo-counts´
    /// ´test:unit:estimate-returns-posterior-mean´
    #[test]
    fn estimate_returns_posterior_mean() {
        let state = ChallengeEffectivenessState::new(4.0, 2.0, 0.9998);
        let (mean, _variance) = state.estimate(&state.t_last);
        // α/(α+β) = 4/6 = 0.666...
        assert!((mean - 4.0 / 6.0).abs() < 1e-10);
    }

    /// The estimate travels with the Beta posterior variance, which is strictly
    /// positive and strictly below the variance of a single coin flip at the same
    /// mean. That gap is the whole point of accumulating counts: a host reading the
    /// pair can tell a well-evidenced catch rate from a barely-evidenced one, which
    /// a point estimate alone would hide.
    ///
    /// ´claim:risk:the-estimates-uncertainty-is-the-beta-posterior-variance-and-sits-below-the-single-trial-variance´
    /// ´test:unit:estimate-variance-correct´
    #[test]
    fn estimate_variance_correct() {
        let state = ChallengeEffectivenessState::new(4.0, 2.0, 0.9998);
        let (mean, variance) = state.estimate(&state.t_last);

        // σ² = αβ / ((α+β)²(α+β+1)) = 4*2 / (36 * 7) = 8/252 ≈ 0.0317
        let expected_variance = (4.0 * 2.0) / (6.0 * 6.0 * 7.0);
        assert!((variance - expected_variance).abs() < 1e-10);
        // Sanity: variance should be positive and small
        assert!(variance > 0.0);
        assert!(variance < mean * (1.0 - mean)); // Less than Bernoulli variance
    }

    /// Effective sample size counts observations and not belief: a state that has
    /// seen nothing reports zero however heavy its prior, and fifty recorded
    /// outcomes report fifty. Subtracting the prior is what keeps a confident-looking
    /// prior from passing itself off as evidence.
    ///
    /// ´claim:risk:effective-sample-size-counts-only-the-evidence-observed-beyond-the-prior´
    /// ´test:unit:effective-sample-size-computation´
    #[test]
    fn effective_sample_size_computation() {
        let mut state = ChallengeEffectivenessState::new(2.0, 2.0, 0.9998);
        assert!(state.effective_sample_size(&state.t_last).abs() < f64::EPSILON);

        // Simulate 50 observations (25 fail, 25 pass) without decay
        let now = state.t_last;
        for _ in 0..25 {
            state.update(ChallengeResult::Fail, &now);
            state.update(ChallengeResult::Pass, &now);
        }
        assert!((state.effective_sample_size(&state.t_last) - 50.0).abs() < f64::EPSILON);
    }

    /// Sufficiency is a verdict on the observed count against a host-chosen bar: a
    /// state fresh from its prior fails it, and the same state passes once enough
    /// real outcomes have accumulated. A host can therefore decline to act on an
    /// estimate that is still mostly prior.
    ///
    /// ´claim:risk:a-channel-counts-as-sufficiently-evidenced-only-once-its-observed-count-reaches-the-hosts-threshold´
    /// ´test:unit:sufficient-evidence-threshold´
    #[test]
    fn sufficient_evidence_threshold() {
        let mut state = ChallengeEffectivenessState::new(2.0, 2.0, 0.9998);
        assert!(!state.sufficient_evidence(10.0, &state.t_last));

        let now = state.t_last;
        for _ in 0..5 {
            state.update(ChallengeResult::Fail, &now);
            state.update(ChallengeResult::Pass, &now);
        }
        assert!(state.sufficient_evidence(10.0, &state.t_last));
    }

    /// Time pulls the accumulated counts back toward the prior, so a run of
    /// observations left to age is worth less than it was — but never nothing: the
    /// effective sample falls and stays positive. Evidence fades rather than being
    /// discarded, which is what lets an estimate follow a changing challenge
    /// mechanism instead of being pinned by however busy the channel once was.
    ///
    /// ´claim:risk:ageing-blends-the-counts-back-toward-the-prior-so-old-evidence-fades-without-vanishing´
    /// ´test:unit:time-decay-reduces-effective-sample´
    #[test]
    fn time_decay_reduces_effective_sample() {
        let mut state = ChallengeEffectivenessState::new(2.0, 2.0, 0.9998);
        let start = state.t_last;

        // Add 50 observations
        for _ in 0..25 {
            state.update(ChallengeResult::Fail, &start);
            state.update(ChallengeResult::Pass, &start);
        }
        let n_eff_before = state.effective_sample_size(&start);
        assert!((n_eff_before - 50.0).abs() < f64::EPSILON);

        // Advance time by 1000 hours
        let future = PersistentTimestamp::new(start.seconds + 1000 * 3600, 0);
        state.apply_time_decay(&future);

        let n_eff_after = state.effective_sample_size(&future);
        // After 1000 hours with γ=0.9998: decay = 0.9998^1000 ≈ 0.8187
        // n_eff should be reduced
        assert!(n_eff_after < n_eff_before, "n_eff should decrease with decay");
        assert!(n_eff_after > 0.0, "n_eff should remain positive");
    }

    /// The fade has a calibrated rate rather than an arbitrary one: at the default
    /// hourly factor, waiting the algebraic half-life leaves an accumulation worth
    /// half what it was. The decay constant a host configures in hours can therefore
    /// be reasoned about in months, which is the timescale challenge mechanisms
    /// actually change on.
    ///
    /// ´claim:risk:the-hourly-decay-factor-halves-accumulated-challenge-evidence-over-its-algebraic-half-life´
    /// ´test:unit:time-decay-half-life´
    #[test]
    fn time_decay_half_life() {
        // At γ=0.9998, half-life in hours: log(0.5) / log(0.9998) ≈ 3465 hours ≈ 144 days
        let mut state = ChallengeEffectivenessState::new(2.0, 2.0, 0.9998);
        let start = state.t_last;

        // Add 100 observations
        for _ in 0..50 {
            state.update(ChallengeResult::Fail, &start);
            state.update(ChallengeResult::Pass, &start);
        }
        let n_eff_initial = state.effective_sample_size(&start);

        // Advance by half-life (~3465 hours)
        #[allow(clippy::cast_possible_truncation)]
        let half_life_hours = 0.5_f64.log(0.9998_f64).round() as i64;
        let future = PersistentTimestamp::new(start.seconds + half_life_hours * 3600, 0);
        state.apply_time_decay(&future);

        let n_eff_after = state.effective_sample_size(&future);
        // Should be approximately half
        let ratio = n_eff_after / n_eff_initial;
        assert!(
            (ratio - 0.5).abs() < 0.05,
            "After half-life, n_eff should be ~50% of initial, got {ratio:.3}"
        );
    }

    /// Companion state is the host's to persist, so it survives a trip through
    /// serialisation with both pseudo-counts and its decay rate intact. Losing the
    /// decay rate on restore would be the quiet failure: the counts would look
    /// right and would then age at the wrong speed.
    ///
    /// ´claim:risk:a-challenge-state-survives-serialisation-with-its-counts-and-its-decay-rate-intact´
    /// ´test:unit:serde-roundtrip´
    #[cfg(feature = "serde")]
    #[test]
    fn serde_roundtrip() {
        let state = ChallengeEffectivenessState::new(5.5, 3.3, 0.9995);
        let json = serde_json::to_string(&state).expect("serialise");
        let restored: ChallengeEffectivenessState = serde_json::from_str(&json).expect("deserialise");
        assert!((restored.alpha - 5.5).abs() < f64::EPSILON);
        assert!((restored.beta - 3.3).abs() < f64::EPSILON);
        assert!((restored.gamma_qt - 0.9995).abs() < f64::EPSILON);
    }

    /// A channel the tracker has never heard of still answers, with the default
    /// estimate rather than an absence. Derivation can therefore run from the first
    /// request on a new channel, before any challenge has been observed, instead of
    /// the host having to seed every channel it might route to.
    ///
    /// ´claim:risk:a-channel-with-no-recorded-challenges-answers-with-the-default-estimate-rather-than-nothing´
    /// ´test:unit:tracker-unknown-channel-returns-default-estimate´
    #[test]
    fn tracker_unknown_channel_returns_default_estimate() {
        let tracker = ChallengeEffectivenessTracker::new();

        let estimate = tracker.estimate(ChannelId(7), &PersistentTimestamp::now());

        assert!((estimate.q_c - ChallengeEstimate::default().q_c).abs() < f64::EPSILON);
        assert!((estimate.variance - ChallengeEstimate::default().variance).abs() < f64::EPSILON);
    }

    /// Recording an outcome on an untracked channel brings that channel's state into
    /// existence from the defaults and hands back the estimate the outcome produced,
    /// already moved above the neutral half. The host never has to register a channel
    /// before reporting on it.
    ///
    /// ´claim:risk:recording-an-outcome-creates-the-channels-state-on-demand-and-returns-the-updated-estimate´
    /// ´test:unit:tracker-update-records-channel-result´
    #[test]
    fn tracker_update_records_channel_result() {
        let mut tracker = ChallengeEffectivenessTracker::new();
        let now = PersistentTimestamp::now();

        let estimate = tracker.update(ChannelId(3), ChallengeResult::Fail, &now);

        assert_eq!(tracker.len(), 1);
        assert!(estimate.q_c > 0.5, "Fail increments alpha and raises q̂_c");
    }

    /// The Companion reports on itself: every tracked channel appears in the health
    /// report with a finite estimate, a finite variance, the count of evidence behind
    /// them and the sufficiency verdict at the host's threshold. An operator can see
    /// which channels are speaking from evidence and which are still repeating the
    /// prior back.
    ///
    /// ´claim:risk:the-companion-health-report-carries-each-channels-estimate-evidence-count-and-sufficiency-verdict´
    /// ´test:unit:tracker-health-report-contains-channel´
    #[test]
    fn tracker_health_report_contains_channel() {
        let mut tracker = ChallengeEffectivenessTracker::new();
        let now = PersistentTimestamp::now();
        tracker.update(ChannelId(3), ChallengeResult::Fail, &now);

        let report = tracker.health_report(1.0, &now);
        let detail = report.channels.get(&ChannelId(3)).expect("channel health exists");

        assert!(detail.q_hat.is_finite());
        assert!(detail.variance.is_finite());
        assert!(detail.effective_sample_size >= 1.0);
        assert!(detail.sufficient_evidence);
    }

    /// Ageing costs confidence, not opinion. Both counts shrink toward their priors
    /// at the same rate, so a balanced posterior stays exactly balanced while its
    /// variance grows and its effective sample falls — and the variance still stops
    /// short of the untouched prior's. Decay is therefore the right shape for a stale
    /// channel: the estimate is not silently rewritten, it is merely believed less.
    ///
    /// ´claim:risk:decay-preserves-the-point-estimate-while-widening-its-uncertainty-and-shrinking-the-evidence-behind-it´
    /// ´test:unit:time-decay-preserves-mean-increases-variance-reduces-n-eff´
    #[test]
    fn time_decay_preserves_mean_increases_variance_reduces_n_eff() {
        // Blend-toward-prior decay composite identity
        // (´dec:companion:decay-form´).
        //
        // Scenario: start from Beta(1, 1), inject 100 failures and 100 passes
        // (q̂_c = 0.5, tight posterior), then age by 6 months.
        //
        //   decay = γ^Δh with γ = 0.9998, Δh = 24 · 182 = 4368 h
        //   decay ≈ 0.4173
        //   α  ← 1 + (101 − 1) · decay ≈ 42.73
        //   β  ← 1 + (101 − 1) · decay ≈ 42.73
        //   q̂_c preserved at 0.5 (ratio is scale-invariant under blend).
        //   n_eff = (α + β) − (α_0 + β_0) ≈ 83.5
        //   σ²(q̂_c) strictly greater than pre-decay (accumulation lost).
        let mut state = ChallengeEffectivenessState::new(1.0, 1.0, 0.9998);
        let start = state.t_last;
        for _ in 0..100 {
            state.update(ChallengeResult::Fail, &start);
            state.update(ChallengeResult::Pass, &start);
        }
        let (mean_before, var_before) = state.estimate(&start);
        let n_eff_before = state.effective_sample_size(&start);
        assert!((mean_before - 0.5).abs() < 1e-12, "q̂_c should start at 0.5");
        assert!((n_eff_before - 200.0).abs() < 1e-12, "n_eff should start at 200");

        // Advance 6 months ≈ 182 days ≈ 4368 hours.
        let six_months_hours: i64 = 24 * 182;
        let future = PersistentTimestamp::new(start.seconds + six_months_hours * 3600, 0);
        state.apply_time_decay(&future);

        let (mean_after, var_after) = state.estimate(&future);
        let n_eff_after = state.effective_sample_size(&future);

        // Mean preserved — the (α − α_0) and (β − β_0) deltas decay at the
        // same rate so α / (α+β) is fixed.
        assert!(
            (mean_after - 0.5).abs() < 1e-12,
            "q̂_c should remain 0.5 under blend-toward-prior decay, got {mean_after}"
        );

        // Variance grows back toward the prior as n_eff shrinks.
        assert!(
            var_after > var_before,
            "σ²(q̂_c) should increase as evidence decays: before={var_before}, after={var_after}"
        );
        // Hard upper bound: still less than Beta(1,1) prior variance (1/12).
        assert!(var_after < 1.0 / 12.0, "σ²(q̂_c) < prior variance");

        // Effective sample size ≈ 100 · 2 · decay ≈ 83.5.
        assert!(
            (n_eff_after - 83.5).abs() < 1.0,
            "n_eff ≈ 83.5 after 6-month decay, got {n_eff_after}"
        );
    }

    /// A read is the posterior at the moment of reading: asked about an idle
    /// channel six months on, the estimate comes back with the counts decayed
    /// to the present — the same numbers a mutating decay would produce — while
    /// the stored state itself has not moved by a bit. An undecayed read is
    /// unconservative in exactly one direction, a variance too tight for
    /// evidence that has aged; and a read that wrote would let a diagnostic
    /// pass erode the evidence it was only supposed to look at.
    ///
    /// ´claim:risk:a-read-returns-the-posterior-decayed-to-the-moment-of-reading-and-mutates-nothing´
    /// ´test:unit:estimate-decays-on-read-without-mutating´
    #[test]
    fn estimate_decays_on_read_without_mutating() {
        let mut state = ChallengeEffectivenessState::new(1.0, 1.0, 0.9998);
        let start = state.t_last;
        for _ in 0..100 {
            state.update(ChallengeResult::Fail, &start);
            state.update(ChallengeResult::Pass, &start);
        }
        let (alpha_stored, beta_stored, t_stored) = (state.alpha, state.beta, state.t_last);
        let (_, var_at_start) = state.estimate(&start);

        // Read six months later without touching the state.
        let future = PersistentTimestamp::new(start.seconds + 24 * 182 * 3600, 0);
        let (mean_read, var_read) = state.estimate(&future);
        let n_eff_read = state.effective_sample_size(&future);

        // The read decayed: same balanced mean, wider variance, less evidence.
        assert!((mean_read - 0.5).abs() < 1e-12, "balanced mean survives read-time decay");
        assert!(var_read > var_at_start, "an aged read must widen, not stay tight");
        assert!(n_eff_read < 200.0, "an aged read reports less evidence than was stored");

        // The read mutated nothing.
        assert!((state.alpha - alpha_stored).abs() < f64::EPSILON, "read must not move alpha");
        assert!((state.beta - beta_stored).abs() < f64::EPSILON, "read must not move beta");
        assert_eq!(state.t_last, t_stored, "read must not restamp t_last");

        // The pure read agrees exactly with the mutating decay's answer.
        let mut mutated = state.clone();
        mutated.apply_time_decay(&future);
        let (mean_mut, var_mut) = mutated.estimate(&future);
        assert!((mean_read - mean_mut).abs() < 1e-12);
        assert!((var_read - var_mut).abs() < 1e-12);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Importance Weighting Tests
    // ─────────────────────────────────────────────────────────────────────────

    /// The positive-class rate is an exponential average of the label stream: each
    /// label moves it toward that label's own indicator by one minus the smoothing
    /// factor. An adverse label nudges it up by a fixed fraction of the distance to
    /// one, so no single label can redefine what "rare" means.
    ///
    /// ´claim:risk:the-positive-class-rate-moves-toward-each-labels-indicator-by-one-minus-the-smoothing-factor´
    /// ´test:unit:p-plus-tracker-update-positive´
    #[test]
    fn p_plus_tracker_update_positive() {
        let mut p_plus = 0.5;
        update_p_plus_tracker(&mut p_plus, 0.99, true);
        // 0.99 * 0.5 + 0.01 * 1.0 = 0.505
        assert!((p_plus - 0.505).abs() < 1e-10);
    }

    /// The same step downward for a benign label: the rate falls toward zero by the
    /// identical fraction. The two directions are one formula with the indicator
    /// swapped, so the tracker cannot drift asymmetrically towards either class.
    ///
    /// (´claim:risk:the-positive-class-rate-moves-toward-each-labels-indicator-by-one-minus-the-smoothing-factor´)
    /// ´test:unit:p-plus-tracker-update-negative´
    #[test]
    fn p_plus_tracker_update_negative() {
        let mut p_plus = 0.5;
        update_p_plus_tracker(&mut p_plus, 0.99, false);
        // 0.99 * 0.5 + 0.01 * 0.0 = 0.495
        assert!((p_plus - 0.495).abs() < 1e-10);
    }

    /// However long a run of one class goes on, the rate is held off both endpoints.
    /// A thousand consecutive adverse labels leave it just short of one rather than
    /// at it — which matters because the importance weight divides by this rate and
    /// by its complement, and an endpoint would make one of those a division by zero.
    ///
    /// ´claim:risk:the-positive-class-rate-is-held-off-both-endpoints-so-the-importance-weight-never-divides-by-zero´
    /// ´test:unit:p-plus-tracker-clamped´
    #[test]
    fn p_plus_tracker_clamped() {
        let mut p_plus = 0.999;
        for _ in 0..1000 {
            update_p_plus_tracker(&mut p_plus, 0.99, true);
        }
        assert!(p_plus <= 0.999);
        assert!(p_plus >= 0.001);
    }

    /// When the two classes arrive equally often there is no imbalance to correct,
    /// and both classes come out at unit weight. The correction is therefore inert
    /// on balanced data rather than being a constant tax applied regardless.
    ///
    /// ´claim:risk:a-balanced-class-rate-leaves-both-classes-at-unit-importance´
    /// ´test:unit:importance-weight-balanced´
    #[test]
    fn importance_weight_balanced() {
        // At P+ = 0.5, weights should be 1.0 for both classes
        let w_pos = compute_importance_weight(0.5, true, 10.0);
        let w_neg = compute_importance_weight(0.5, false, 10.0);
        assert!((w_pos - 1.0).abs() < 1e-10);
        assert!((w_neg - 1.0).abs() < 1e-10);
    }

    /// Each weight is the reciprocal of twice its class rate, and the point of
    /// that form is what it buys: at any class rate, each class contributes
    /// exactly half the total gradient weight — rate times weight is one half
    /// on both sides. At a one-in-ten positive rate the adverse label carries
    /// 5 and the benign one 5/9, a ratio of nine; the odds ratio would carry 9
    /// against 1/9, squaring the ratio and abandoning the equal-halves
    /// property at every rate but one half. The gradient balance, not the
    /// ratio, is the quantity the models are owed.
    ///
    /// ´claim:risk:each-class-contributes-exactly-half-the-gradient-weight-at-any-class-rate´
    /// ´test:unit:importance-weight-rare-positive´
    #[test]
    fn importance_weight_rare_positive() {
        // At P+ = 0.1 (´def:weighting:balancing-weights´):
        // w+ = 1 / (2·0.1) = 5.0, w− = 1 / (2·0.9) = 5/9.
        let w_pos = compute_importance_weight(0.1, true, 10.0);
        let w_neg = compute_importance_weight(0.1, false, 10.0);
        assert!((w_pos - 5.0).abs() < 1e-10);
        assert!((w_neg - 5.0 / 9.0).abs() < 1e-10);

        // The defining property: each class's expected share of the
        // gradient weight is exactly one half.
        assert!(0.1_f64.mul_add(w_pos, -0.5).abs() < 1e-10);
        assert!(0.9_f64.mul_add(w_neg, -0.5).abs() < 1e-10);
    }

    /// The upweighting is bounded: at an extreme class rate the raw balancing
    /// weight would make one label worth dozens, and the ceiling cuts it to
    /// the caller's limit instead. The correction can lift the rare class
    /// without letting a handful of labels take over the fit entirely.
    ///
    /// ´claim:risk:the-ceiling-caps-what-a-single-rare-label-can-be-worth´
    /// ´test:unit:importance-weight-ceiling-applied´
    #[test]
    fn importance_weight_ceiling_applied() {
        // At P+ = 0.01 the balancing weight would be 1/(2·0.01) = 50,
        // but the ceiling limits it.
        let w_pos = compute_importance_weight(0.01, true, 10.0);
        assert!((w_pos - 10.0).abs() < 1e-10);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Eligibility Tests
    // ─────────────────────────────────────────────────────────────────────────

    /// A request that was allowed through runs its course, so whatever happened next
    /// is an unconfounded observation of what that entity does — eligible whether
    /// or not anyone verified it. This is the population the model can learn from
    /// without correction.
    ///
    /// ´claim:risk:an-allowed-request-yields-an-unconfounded-outcome-so-its-label-always-trains-the-model´
    /// ´test:unit:eligibility-allow-always-eligible´
    #[test]
    fn eligibility_allow_always_eligible() {
        use crate::types::Action;
        let policy = crate::config::types::EligibilityPolicy::default();
        assert!(determine_eligibility(Action::Allow, policy, false));
        assert!(determine_eligibility(Action::Allow, policy, true));
    }

    /// Where the index intervened, nobody knows what would have happened: a blocked
    /// request produces no outcome to learn from unless the outcome was established
    /// independently. Only the ground-truth flag makes such a label eligible.
    /// Training on the rest would teach the model that its own interventions work.
    ///
    /// ´claim:risk:a-label-from-an-intervened-request-trains-the-model-only-when-the-outcome-was-independently-verified´
    /// ´test:unit:eligibility-block-requires-ground-truth´
    #[test]
    fn eligibility_block_requires_ground_truth() {
        use crate::types::Action;
        let policy = crate::config::types::EligibilityPolicy::default();
        assert!(!determine_eligibility(Action::Block, policy, false));
        assert!(determine_eligibility(Action::Block, policy, true));
    }

    /// A slowed request is an intervention on the same footing as a block: the
    /// outcome that followed is the outcome of a slowed request, and admitting
    /// it would teach the inherent-risk estimate about the host's own behaviour.
    /// So a slow label trains the sister and anchor only under the ground-truth
    /// flag — this is the causal-inference contract's confounded row, and the
    /// predicate that once admitted it unconditionally was training the
    /// deployment's inherent-risk estimate on the host's interventions.
    ///
    /// (´claim:risk:a-label-from-an-intervened-request-trains-the-model-only-when-the-outcome-was-independently-verified´)
    /// ´test:unit:eligibility-slow-requires-ground-truth´
    #[test]
    fn eligibility_slow_requires_ground_truth() {
        use crate::types::Action;
        let policy = crate::config::types::EligibilityPolicy::default();
        assert!(!determine_eligibility(Action::Slow, policy, false));
        assert!(determine_eligibility(Action::Slow, policy, true));
    }

    /// A challenge is not a block: the request proceeded, and by default its
    /// label trains the models, because the default policy asserts that failing
    /// a challenge is a property of the request rather than an effect of having
    /// been challenged. A deployment that disputes that sets the policy false,
    /// and then a non-ground-truth challenge label is excluded — the predicate
    /// defers to the declared policy rather than hard-coding either judgement.
    ///
    /// ´claim:risk:a-challenge-label-trains-by-default-and-the-declared-policy-can-withdraw-it´
    /// ´test:unit:eligibility-challenge-follows-policy´
    #[test]
    fn eligibility_challenge_follows_policy() {
        use crate::types::Action;

        // Default policy: included (´tab:eligibility:training´).
        let default_policy = crate::config::types::EligibilityPolicy::default();
        assert!(default_policy.challenge_fail_is_unconfounded);
        assert!(determine_eligibility(Action::Challenge, default_policy, false));

        // A deployment that disputes the default withdraws the row.
        let strict = crate::config::types::EligibilityPolicy {
            challenge_fail_is_unconfounded: false,
        };
        assert!(!determine_eligibility(Action::Challenge, strict, false));

        // Ground truth restores eligibility under either policy.
        assert!(determine_eligibility(Action::Challenge, strict, true));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Risk Target Tests
    // ─────────────────────────────────────────────────────────────────────────

    /// The risk target the model trains against is a sign, not a magnitude: a
    /// strongly adverse observation and a mildly adverse one both become the same
    /// positive target. How bad an outcome was is the host's business; whether it
    /// was adverse is all the model is asked to learn.
    ///
    /// ´claim:risk:the-risk-target-is-a-sign-not-a-magnitude-so-any-adverse-valence-maps-to-the-same-target´
    /// ´test:unit:challenge-risk-target-positive-valence´
    #[test]
    fn challenge_risk_target_positive_valence() {
        assert!((compute_risk_target(1.0) - 1.0).abs() < f64::EPSILON);
        assert!((compute_risk_target(0.5) - 1.0).abs() < f64::EPSILON);
    }

    /// The benign side collapses the same way: any negative valence, slight or
    /// severe, becomes the single negative target. The mapping is a two-way split of
    /// the real line, so the model's training signal has exactly two values.
    ///
    /// ´claim:risk:every-non-adverse-valence-collapses-to-the-single-negative-target´
    /// ´test:unit:challenge-risk-target-negative-valence´
    #[test]
    fn challenge_risk_target_negative_valence() {
        assert!((compute_risk_target(-1.0) - (-1.0)).abs() < f64::EPSILON);
        assert!((compute_risk_target(-0.5) - (-1.0)).abs() < f64::EPSILON);
    }

    /// The split is strict, and zero falls on the benign side. An outcome that
    /// registered no harm is not treated as evidence of harm, so a stream of
    /// nothing-happened labels pushes the model toward benign rather than sitting
    /// ambiguously between the two targets.
    ///
    /// (´claim:risk:every-non-adverse-valence-collapses-to-the-single-negative-target´)
    /// ´test:unit:risk-target-zero-valence´
    #[test]
    fn risk_target_zero_valence() {
        assert!((compute_risk_target(0.0) - (-1.0)).abs() < f64::EPSILON);
    }

    /// A host override stands in front of the posterior rather than replacing it:
    /// while set, every read returns the override, and clearing it hands the override
    /// back and reveals the Beta estimate that was accumulating underneath all along.
    /// An operator who knows a challenge has been defeated can say so immediately and
    /// then step back without having discarded the channel's evidence.
    ///
    /// ´claim:risk:a-host-override-stands-in-front-of-the-posterior-until-cleared-and-the-beta-estimate-resumes-underneath´
    /// ´test:unit:tracker-override-replaces-estimate-until-cleared´
    #[test]
    fn tracker_override_replaces_estimate_until_cleared() {
        let mut tracker = ChallengeEffectivenessTracker::new();
        let channel_id = ChannelId(3);
        let now = PersistentTimestamp::now();
        tracker.update(channel_id, ChallengeResult::Fail, &now);

        let overridden = tracker.set_override(channel_id, 0.9, None);
        assert!((overridden.q_c - 0.9).abs() < f64::EPSILON);
        assert!((tracker.estimate(channel_id, &now).q_c - 0.9).abs() < f64::EPSILON);

        let cleared = tracker.clear_override(channel_id).expect("override was active");
        assert!((cleared.q_c - 0.9).abs() < f64::EPSILON);
        assert!(
            tracker.estimate(channel_id, &now).q_c < 0.9,
            "beta estimate should resume after override clears"
        );
    }

    /// Evidence a host imports from outside is capped at the channel's ceiling before
    /// it reaches the Beta state: an injection of five against a ceiling of two adds
    /// two, and the resulting estimate and effective sample both reflect the capped
    /// amount. One import cannot claim more authority than the channel's owner
    /// allowed it, however large the number handed in.
    ///
    /// ´claim:risk:injected-evidence-is-capped-at-the-channels-ceiling-before-it-reaches-the-beta-state´
    /// ´test:unit:tracker-inject-evidence-clamps-to-ceiling´
    #[test]
    fn tracker_inject_evidence_clamps_to_ceiling() {
        let mut tracker = ChallengeEffectivenessTracker::new();
        let channel_id = ChannelId(3);
        let state = ChallengeEffectivenessState {
            injection_ceiling: 2.0,
            ..ChallengeEffectivenessState::default()
        };
        tracker.insert_channel(channel_id, state);
        let now = PersistentTimestamp::now();

        let estimate = tracker
            .inject_evidence(channel_id, 5.0, 0.0, &now)
            .expect("valid evidence injection");

        assert!((estimate.q_c - 3.0 / 4.0).abs() < f64::EPSILON);
        let health = tracker.health_report(1.0, &now);
        assert!((health.channels[&channel_id].effective_sample_size - 2.0).abs() < f64::EPSILON);
    }

    /// A channel created on demand by an injection carries a default ceiling of a
    /// thousand pseudo-counts, so five thousand offered lands as a thousand recorded.
    /// The cap is a property of the default state and not something a host must
    /// remember to configure before the first import.
    ///
    /// ´claim:risk:a-channel-created-on-demand-caps-injections-at-a-thousand-pseudo-counts-by-default´
    /// ´test:unit:tracker-inject-evidence-default-ceiling-is-one-thousand´
    #[test]
    fn tracker_inject_evidence_default_ceiling_is_one_thousand() {
        let mut tracker = ChallengeEffectivenessTracker::new();
        let channel_id = ChannelId(3);
        let now = PersistentTimestamp::now();

        let estimate = tracker
            .inject_evidence(channel_id, 5000.0, 0.0, &now)
            .expect("valid evidence injection");

        assert!((estimate.q_c - 1001.0 / 1002.0).abs() < f64::EPSILON);
        let health = tracker.health_report(1.0, &now);
        assert!((health.channels[&channel_id].effective_sample_size - 1000.0).abs() < f64::EPSILON);
    }

    /// A count that is not a finite non-negative number is refused, and refused
    /// early: the channel it named is not brought into existence by the attempt. A
    /// rejected import leaves the tracker exactly as it was, rather than seeding an
    /// empty channel that a later read would answer from.
    ///
    /// ´claim:risk:an-invalid-injection-is-refused-before-any-channel-state-is-created´
    /// ´test:unit:tracker-inject-evidence-rejects-negative-counts´
    #[test]
    fn tracker_inject_evidence_rejects_negative_counts() {
        let mut tracker = ChallengeEffectivenessTracker::new();
        let result = tracker.inject_evidence(ChannelId(3), -1.0, 0.0, &PersistentTimestamp::now());

        assert!(matches!(result, Err(ChallengeEvidenceError::InvalidInjection)));
        assert!(tracker.is_empty(), "invalid injection should not create channel state");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // The replacement surface
    // ─────────────────────────────────────────────────────────────────────────

    /// A provider that owns only a scalar estimate — the analytics-pipeline
    /// or lookup-table case the replacement surface exists for.
    struct ScalarOnlyProvider {
        estimate: ChallengeEstimate,
    }

    impl ChallengeEffectivenessProvider for ScalarOnlyProvider {
        fn challenge_estimate(&self, _channel_id: ChannelId, _now: &PersistentTimestamp) -> ChallengeEstimate {
            self.estimate
        }
    }

    /// A provider of a scalar and a variance participates in the posterior
    /// machinery without implementing anything further: the surface's
    /// default posterior is a moment-matched Beta that reproduces the
    /// provider's mean and variance exactly, and a pair no Beta can carry
    /// falls back to the uniform prior rather than to an invented shape.
    /// Without this default, every scalar provider would have to fork the
    /// shipped tracker to be heard at all.
    ///
    /// ´claim:risk:a-scalar-provider-gets-a-moment-matched-posterior-without-implementing-one´
    /// ´test:unit:replacement-trait-default-posterior-moment-matches´
    #[test]
    fn replacement_trait_default_posterior_moment_matches() {
        let now = PersistentTimestamp::now();
        let provider = ScalarOnlyProvider {
            estimate: ChallengeEstimate::new(0.7, 0.01).expect("in-domain pair"),
        };

        let posterior = provider.challenge_posterior(ChannelId(1), &now);
        assert!((posterior.mean() - 0.7).abs() < 1e-12, "the match reproduces the mean");
        assert!(
            (posterior.variance() - 0.01).abs() < 1e-12,
            "the match reproduces the variance"
        );

        // A degenerate pair — variance at or beyond m(1 − m) — has no
        // matching Beta and answers from the uniform prior.
        let degenerate = ScalarOnlyProvider {
            estimate: ChallengeEstimate::new(0.5, 0.5).expect("in [0,1] with non-negative variance"),
        };
        assert_eq!(
            degenerate.challenge_posterior(ChannelId(1), &now),
            ChallengePosterior::uniform_prior()
        );
    }

    /// The shipped tracker implements the replacement surface, and as the
    /// holder of a genuine conjugate posterior it answers with its exact
    /// decayed counts rather than a moment match: after one observed
    /// failure on uniform priors the posterior is Beta(2, 1), the trait's
    /// estimate agrees with the tracker's own read, and an unknown channel
    /// answers from the uniform prior. This is what makes the tracker
    /// substitutable — a host implementing against the trait gets the same
    /// answers the concrete type gives.
    ///
    /// ´claim:risk:the-shipped-tracker-answers-the-replacement-surface-with-its-exact-posterior-counts´
    /// ´test:unit:shipped-tracker-implements-the-replacement-surface´
    #[test]
    fn shipped_tracker_implements_the_replacement_surface() {
        let now = PersistentTimestamp::now();
        let mut tracker = ChallengeEffectivenessTracker::new();
        tracker.update(ChannelId(1), ChallengeResult::Fail, &now);

        let provider: &dyn ChallengeEffectivenessProvider = &tracker;

        // The trait's estimate is the tracker's estimate.
        let via_trait = provider.challenge_estimate(ChannelId(1), &now);
        let direct = tracker.estimate(ChannelId(1), &now);
        assert!((via_trait.q_c() - direct.q_c()).abs() < f64::EPSILON);
        assert!((via_trait.variance() - direct.variance()).abs() < f64::EPSILON);

        // The posterior is the exact conjugate counts, not a moment match.
        let posterior = provider.challenge_posterior(ChannelId(1), &now);
        assert!(
            (posterior.alpha - 2.0).abs() < 1e-9,
            "one failure on Beta(1, 1) gives alpha 2"
        );
        assert!((posterior.beta - 1.0).abs() < 1e-9, "beta stays at its prior count");

        // An unknown channel answers from the uniform prior, agreeing with
        // the estimate's own default.
        let unknown = provider.challenge_posterior(ChannelId(9), &now);
        assert_eq!(unknown, ChallengePosterior::uniform_prior());
    }
}
