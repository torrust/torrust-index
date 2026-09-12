// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`bootstrap_observe_updates_count`] | feature | Each observed vector advances the count by exactly one, starting from a freshly built accumulator at nothing. The count is what the completion test reads and what the variance is divided by, so it must track observations rather than, say, the number of positions written on each pass. |
//! | [`bootstrap_is_complete_at_target`] | feature | An accumulator declares itself complete the moment its count reaches the target, and not one observation earlier. Completion is what releases a slot into service, so the boundary decides whether the model starts from the sample size it was configured to require or from one short of it. |
//! | [`bootstrap_complete_returns_stats`] | feature | From one observation the statistics are that vector as the mean, zero variance throughout, and a count of one. A single point locates a distribution but says nothing about its width, and the accumulator reports exactly that — the count travelling alongside so a consumer can see how little the zero rests on. |
//! | [`bootstrap_complete_none_when_empty`] | feature | An accumulator that has seen nothing yields no statistics at all, rather than a vector of zero means and zero variances. Zeros would be indistinguishable from a genuinely measured constant stream, and a consumer that standardised against them would be dividing by a variance nothing supports; the absence is made explicit instead. |
//! | [`bootstrap_welford_variance`] | feature | The statistics an accumulator reports are the population mean and population variance of exactly the values it was shown — five observations spread evenly about five come back as a mean of five and a variance of two. The figures are built incrementally as observations arrive, so nothing needs to be retained but the running pair, and the answer is the same as a second pass over the stored sample would give. |
//! | [`batch_init_observe`] | feature | cites (´claim:feature:each-observed-vector-advances-the-accumulators-count-by-one´) |
//! | [`batch_init_complete`] | feature | cites (´claim:feature:the-accumulator-reports-the-population-mean-and-variance-of-what-it-was-shown´) |
//! | [`batch_init_reset`] | feature | A reset returns the count to nothing and re-sizes the running mean and second-moment vectors to the width it is given, discarding what was accumulated at the old width. A lifecycle event that changes the feature vector's dimension invalidates every position that came before it, so initialisation restarts rather than trying to carry statistics across a renumbering. |
//! | [`welford_numerical_stability`] | feature | A tiny spread riding on an enormous common offset is still recovered: three values ten billion apart from zero but one apart from each other give back both the large mean and the small variance. Computing the variance from a sum of squares would lose the difference entirely to cancellation here, whereas the incremental form works in deviations from the running mean and never forms the large square at all. |
//! | [`bootstrap_multi_feature_variance`] | feature | Every position keeps its own running pair: over the same five observations a constant position reports zero variance, a steadily rising one reports a modest spread, and one that stays flat then jumps enormously reports a huge spread — none of them influenced by the others. Features share only the vector they arrive in, so a single violent slot must not widen the variance the quiet slots are standardised against. |
//! | [`batch_init_not_complete_below_target`] | feature | cites (´claim:feature:an-accumulator-reports-complete-on-reaching-its-target-and-not-before´) |
//! | [`batch_init_completes_exactly_at_target`] | feature | cites (´claim:feature:an-accumulator-reports-complete-on-reaching-its-target-and-not-before´) |
//! | [`batch_init_stats_dimension`] | feature | The mean and variance vectors come back exactly as wide as the vectors that were observed. Standardisation indexes these two records by feature position, so a width that disagreed with the feature vector would silently standardise each slot against some other slot's statistics. |
//! | [`bootstrap_target_zero`] | feature | A target of zero is satisfied before anything is observed, yet the accumulator still refuses to produce statistics it has no observations for. Completion and having something to report are separate questions, so a configuration that asks for no warm-up cannot trick the accumulator into inventing a mean and a variance. |
//! | [`bootstrap_single_observation`] | feature | cites (´claim:feature:a-single-observation-yields-that-vector-as-the-mean-with-zero-variance´) |
//! | [`init_stats_are_finite`] | feature | A stream carrying a NaN at one position and an infinity at another, mid-accumulation, leaves every accumulated mean and variance finite and no variance negative: the offending entries are passed over one element at a time, so the positions beside them keep accumulating and the affected positions come back describing only the observations that were numbers. The alternative is not a locally wrong statistic but a permanently poisoned one — every later update is a difference against the running pair, so one bad entry keeps that position non-finite for the rest of the accumulation, and the statistics standardisation divides by are published from the first accepted observation onward, a share of the base mass at a time. |
//! | [`cold_ramp_phase_reads_off_the_count`] | feature | The phase is a reading of the accepted count and never a second piece of state: a ramp is waiting at zero, transitioning from the first accepted observation, still transitioning one short of the horizon, and in service exactly at it. The three values are what a host is told the coordinate system is doing, so a phase that could disagree with the count it accompanies would be two answers to one question — and the answer a host acted on would be whichever it happened to read. |
//! | [`cold_ramp_endpoint_is_exactly_empirical`] | feature | At the horizon the published moments are the empirical moments of the accepted sample bit for bit, not within a tolerance: the retired share is exactly one, so the base weight and the between-population term are both exactly zero and nothing of the priors survives the arithmetic. This is the property that makes the ramp a redistribution of the old wholesale replacement rather than a different destination — the same endpoint, reached in equal steps instead of one. |
//! | [`cold_ramp_variance_carries_the_between_population_term`] | feature | The variance published part way along the ramp is the variance of the mixture and not a mixture of the two variances: the term carrying how far the observed mean has travelled from the base mean is published with them. Here the base mean is zero and the sample's is four, so at the half-way point that term contributes a quarter of sixteen — and a variance that dropped it would report a spread narrower than either population's own, understating it worst exactly where the two disagree most and where a reader has least other evidence about which coordinate system a result came from. |
//! | [`cold_ramp_leaves_the_bias_position_alone`] | feature | The bias position takes no part in the ramp at any count: it is a constant the model reads directly and nothing standardises it, so a mean and a variance measured for it would describe a quantity that is never divided by them. Here the accepted sample carries a wildly varying value at the bias index and the published bias moments are still the ones the ramp started with, while its neighbours move. |
//! | [`cold_ramp_retires_one_equal_share_per_observation`] | feature | Each accepted observation retires exactly one horizon-th of the base mass and no more: against a constant sample the published mean advances in equal steps from the base to the sample's own value, and the largest single step is the same size as the smallest. The equal share is the whole point of the ramp — it is what replaced a coordinate system that moved its entire prior-to-empirical distance between two adjacent requests with one that discloses the same distance a share at a time. |
//! | [`cold_ramp_rebase_takes_published_moments`] | feature | A lifecycle event rebases the ramp on the moments it has just published and discards the sample gathered under the layout it replaced: the count returns to zero, the new base is what was published rather than the original priors, and the layout the ramp will accept observations under has moved on. Rolling the surviving positions back to their own priors instead would undo measurement the instance had already paid for, and carrying the old sample forward would attribute one position's distribution to another position's index. |
//! | [`cold_ramp_restore_recomputes_against_configured_horizon`] | feature | A restored ramp recomputes its position against the horizon the restoring instance is configured with, and a count at or past that horizon is complete on arrival: a sample of sixty against a horizon of fifty publishes the empirical moments and reports itself in service, while the same sixty against a horizon of a hundred is still transitioning with three fifths of the base mass retired. Carrying a horizon in the checkpoint instead would put a second authority beside the configuration, and a restore would then have two answers for how far along it was. |
//! | [`cold_ramp_publishes_finite_moments_through_non_finite_input`] | feature | cites (´claim:feature:a-non-finite-entry-is-skipped-per-element-so-it-cannot-poison-the-running-pair´) |
//! | [`batch_init_stats_are_finite`] | feature | cites (´claim:feature:a-non-finite-entry-is-skipped-per-element-so-it-cannot-poison-the-running-pair´) |

#![allow(dead_code)]
#![allow(clippy::doc_markdown, clippy::cast_precision_loss, clippy::suboptimal_flops)]

//! Bootstrap accumulator for Sentinel slot initialization.
//!
//! The `BootstrapAccumulator` collects observations during the bootstrap
//! phase of a Sentinel slot. Once sufficient observations have been
//! collected, it produces initial statistics for the model.
//!
//! The `BatchInitAccumulator` collects observations for full-vector
//! standardisation initialization before the model becomes operational.
//!
//! # Cross-References
//!
//! - (´def:extraction:slot´) — the per-Sentinel slot's structure: an
//!   occupancy indicator, then the extraction these accumulators average

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::feature::standardisation::FeatureClass;

// ═══════════════════════════════════════════════════════════════════════════════
// Init Stats
// ═══════════════════════════════════════════════════════════════════════════════

/// Statistics computed from bootstrap or batch init observations.
///
/// Contains mean and variance estimates for a set of features.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct InitStats {
    /// Running mean estimates per feature.
    pub mean: Vec<f64>,
    /// Running variance estimates per feature.
    pub variance: Vec<f64>,
    /// Number of observations used to compute the statistics.
    pub count: usize,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Bootstrap Accumulator
// ═══════════════════════════════════════════════════════════════════════════════

/// Accumulator for bootstrap phase observations.
///
/// Collects feature vectors during bootstrap to compute initial mean and
/// variance estimates. It is the second of the two acquisition mechanisms,
/// and no third exists (´dec:vector:two-acquisitions´).
///
/// Uses Welford's online algorithm for numerical stability.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BootstrapAccumulator {
    /// Running mean vector (Welford's algorithm).
    pub running_mean: Vec<f64>,
    /// Running M2 for variance computation (Welford's algorithm).
    ///
    /// M2 = Σ(x - mean_old)(x - mean_new). Variance = M2 / n.
    pub running_m2: Vec<f64>,
    /// Number of observations accumulated.
    pub count: usize,
    /// Target number of observations before completion.
    pub target: usize,
}

impl BootstrapAccumulator {
    /// Creates a new `BootstrapAccumulator`.
    ///
    /// # Arguments
    ///
    /// * `dimension` — Number of features to track
    /// * `target` — Number of observations before bootstrap completes
    #[must_use]
    pub fn new(dimension: usize, target: usize) -> Self {
        Self {
            running_mean: vec![0.0; dimension],
            running_m2: vec![0.0; dimension],
            count: 0,
            target,
        }
    }

    /// Returns `true` if bootstrap has collected enough observations.
    #[must_use]
    pub const fn is_complete(&self) -> bool {
        self.count >= self.target
    }

    /// Returns the current observation count.
    #[must_use]
    pub const fn count(&self) -> usize {
        self.count
    }

    /// Returns the target observation count.
    #[must_use]
    pub const fn target(&self) -> usize {
        self.target
    }

    /// Observes a feature vector and updates running statistics.
    ///
    /// Uses Welford's online algorithm for numerical stability:
    /// - mean(n) = mean(n-1) + (x - mean(n-1)) / n
    /// - M2(n) = M2(n-1) + (x - mean(n-1)) * (x - mean(n))
    ///
    /// If the feature dimension has changed (lifecycle event changed
    /// the slot width), the accumulator resets at the new dimension and
    /// re-starts from zero, and non-finite entries are skipped element by
    /// element — both as step 2 of
    /// (´alg:standardisation:sentinel-bootstrap´) requires.
    ///
    /// # Arguments
    ///
    /// * `features` — The feature vector to observe
    pub fn observe(&mut self, features: &[f64]) {
        if features.len() != self.running_mean.len() {
            *self = Self::new(features.len(), self.target);
        }

        self.count += 1;
        let n = self.count as f64;

        for (j, &x) in features.iter().enumerate() {
            if !x.is_finite() {
                continue;
            }
            let delta = x - self.running_mean[j];
            self.running_mean[j] += delta / n;
            let delta2 = x - self.running_mean[j];
            self.running_m2[j] += delta * delta2;
        }
    }

    /// Completes the bootstrap and returns the accumulated statistics.
    ///
    /// Computes variance from M2: variance = M2 / n.
    /// Returns `None` if no observations have been made.
    ///
    /// # Returns
    ///
    /// `Some(InitStats)` with mean and variance, or `None` if count is zero.
    #[must_use]
    pub fn complete(&self) -> Option<InitStats> {
        if self.count == 0 {
            return None;
        }

        let n = self.count as f64;
        let variance: Vec<f64> = self.running_m2.iter().map(|&m2| m2 / n).collect();

        Some(InitStats {
            mean: self.running_mean.clone(),
            variance,
            count: self.count,
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Batch Init Accumulator
// ═══════════════════════════════════════════════════════════════════════════════

/// Accumulator for batch initialisation of full feature vectors.
///
/// Collects observations of the complete φ vector during the initial
/// operation phase to compute standardisation statistics before the
/// model enters full service.
///
/// Uses Welford's online algorithm for numerical stability.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BatchInitAccumulator {
    /// Running mean vector (Welford's algorithm).
    pub running_mean: Vec<f64>,
    /// Running M2 for variance computation (Welford's algorithm).
    pub running_m2: Vec<f64>,
    /// Number of observations accumulated.
    pub count: usize,
    /// Target number of observations before completion.
    pub target: usize,
}

impl BatchInitAccumulator {
    /// Creates a new `BatchInitAccumulator`.
    ///
    /// # Arguments
    ///
    /// * `dimension` — Full feature vector dimension (p)
    /// * `target` — Number of observations before init completes
    #[must_use]
    pub fn new(dimension: usize, target: usize) -> Self {
        Self {
            running_mean: vec![0.0; dimension],
            running_m2: vec![0.0; dimension],
            count: 0,
            target,
        }
    }

    /// Returns `true` if batch init has collected enough observations.
    #[must_use]
    pub const fn is_complete(&self) -> bool {
        self.count >= self.target
    }

    /// Returns the current observation count.
    #[must_use]
    pub const fn count(&self) -> usize {
        self.count
    }

    /// Returns the target observation count.
    #[must_use]
    pub const fn target(&self) -> usize {
        self.target
    }

    /// Observes a full feature vector and updates running statistics.
    ///
    /// If the feature dimension has changed (lifecycle event changed p),
    /// the accumulator resets at the new dimension and re-starts from zero,
    /// and non-finite entries are skipped element by element — both as the
    /// accumulate step of
    /// (´alg:standardisation:batch-initialisation´) requires.
    ///
    /// # Arguments
    ///
    /// * `phi_raw` — The raw (unstandardised) feature vector
    pub fn observe(&mut self, phi_raw: &[f64]) {
        if phi_raw.len() != self.running_mean.len() {
            self.reset(phi_raw.len());
        }

        self.count += 1;
        let n = self.count as f64;

        for (j, &x) in phi_raw.iter().enumerate() {
            if !x.is_finite() {
                continue;
            }
            let delta = x - self.running_mean[j];
            self.running_mean[j] += delta / n;
            let delta2 = x - self.running_mean[j];
            self.running_m2[j] += delta * delta2;
        }
    }

    /// Completes batch init and returns the accumulated statistics.
    ///
    /// # Returns
    ///
    /// `Some(InitStats)` with mean and variance, or `None` if count is zero.
    #[must_use]
    pub fn complete(&self) -> Option<InitStats> {
        if self.count == 0 {
            return None;
        }

        let n = self.count as f64;
        let variance: Vec<f64> = self.running_m2.iter().map(|&m2| m2 / n).collect();

        Some(InitStats {
            mean: self.running_mean.clone(),
            variance,
            count: self.count,
        })
    }

    /// Resets the accumulator for a new dimension (after lifecycle event).
    ///
    /// # Arguments
    ///
    /// * `new_dimension` — New feature vector dimension
    pub fn reset(&mut self, new_dimension: usize) {
        self.running_mean = vec![0.0; new_dimension];
        self.running_m2 = vec![0.0; new_dimension];
        self.count = 0;
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Cold Prior-Mass Ramp
// ═══════════════════════════════════════════════════════════════════════════════

/// The cold full-vector standardisation ramp
/// (´alg:standardisation:batch-initialisation´).
///
/// The class priors carry the whole of the standardisation mass when an
/// instance is built, and each accepted raw assessment vector retires an equal
/// share of it. At the horizon the prior mass is zero and the published
/// moments are exactly the empirical moments of the accepted sample
/// (´dec:vector:prior-mass-ramp´).
///
/// The accepted count is the whole of the ramp's position: the phase is read
/// off it rather than stored beside it, and maturity is the count against the
/// horizon rather than a third field that could disagree with either
/// (´dec:health:standardisation-phase-reported´). What is stored is the base
/// the shares are retired against, the sufficient statistics of the accepted
/// sample, and the layout those two were gathered under.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ColdRamp {
    /// The moments each retired share is taken against — the class priors at
    /// cold start, and whatever a lifecycle event last published thereafter.
    base_means: Vec<f64>,
    /// The base variances, alongside `base_means`.
    base_variances: Vec<f64>,
    /// Sufficient statistics of the accepted sample, and the count.
    accumulator: BatchInitAccumulator,
    /// Which layout the base and the sample belong to. An observation
    /// assembled under a different one is refused rather than mixed
    /// (´req:standardisation:lifecycle-entries´).
    layout_generation: u64,
}

impl ColdRamp {
    /// Starts a ramp at the given base moments, with nothing accepted yet.
    ///
    /// # Arguments
    ///
    /// * `base_means` — Base moments, the class priors at cold start
    /// * `base_variances` — Base variances, alongside `base_means`
    /// * `horizon` — `N_init`, the accepted-observation horizon
    /// * `layout_generation` — The layout the base belongs to
    #[must_use]
    pub fn new(base_means: Vec<f64>, base_variances: Vec<f64>, horizon: usize, layout_generation: u64) -> Self {
        let dimension = base_means.len();
        Self {
            base_means,
            base_variances,
            accumulator: BatchInitAccumulator::new(dimension, horizon),
            layout_generation,
        }
    }

    /// Rebuilds a ramp from checkpointed parts.
    ///
    /// The horizon is the one the restoring instance is configured with, not
    /// one carried in the checkpoint: a restored ramp recomputes its position
    /// against the horizon in force, so there is never a second horizon
    /// authority to disagree with the configuration
    /// (´dec:durability:ramp-resumption´). A count at or past that horizon is
    /// therefore complete on arrival and publishes the empirical moments.
    #[must_use]
    pub const fn from_parts(
        base_means: Vec<f64>,
        base_variances: Vec<f64>,
        running_mean: Vec<f64>,
        running_m2: Vec<f64>,
        accepted: usize,
        horizon: usize,
        layout_generation: u64,
    ) -> Self {
        Self {
            base_means,
            base_variances,
            accumulator: BatchInitAccumulator {
                running_mean,
                running_m2,
                count: accepted,
                target: horizon,
            },
            layout_generation,
        }
    }

    /// Rebuilds a ramp that publishes the given moments and reports the given
    /// count, for a carrier that kept the published pair but not the parts
    /// behind it.
    ///
    /// The model snapshot carries what was published, the phase and the count,
    /// but not the base and the sample the mixture was formed from
    /// (´def:publication:model-snapshot´). Taking the published pair as both
    /// the base and the sample is the reconstruction that is self-consistent
    /// at every count: the mixture of a population with itself is itself, so
    /// the rebuilt ramp publishes exactly the moments it was given and reports
    /// exactly the count it was given. What it cannot recover is how those
    /// moments were reached, which is why a restore that must resume the ramp
    /// reads the checkpoint's own parts instead
    /// (´dec:durability:ramp-resumption´).
    #[must_use]
    pub fn from_published(means: &[f64], variances: &[f64], accepted: usize, horizon: usize, layout_generation: u64) -> Self {
        #[allow(clippy::cast_precision_loss)]
        let n = accepted.max(1) as f64;
        Self::from_parts(
            means.to_vec(),
            variances.to_vec(),
            means.to_vec(),
            variances.iter().map(|&v| v * n).collect(),
            accepted,
            horizon,
            layout_generation,
        )
    }

    /// The base moments each retired share is taken against.
    #[must_use]
    pub fn base_means(&self) -> &[f64] {
        &self.base_means
    }

    /// The base variances, alongside [`base_means`](Self::base_means).
    #[must_use]
    pub fn base_variances(&self) -> &[f64] {
        &self.base_variances
    }

    /// The sufficient statistics of the accepted sample.
    #[must_use]
    pub const fn accumulator(&self) -> &BatchInitAccumulator {
        &self.accumulator
    }

    /// Accepted observations so far.
    #[must_use]
    pub const fn accepted(&self) -> usize {
        self.accumulator.count
    }

    /// The ramp horizon `N_init`.
    #[must_use]
    pub const fn horizon(&self) -> usize {
        self.accumulator.target
    }

    /// The layout the base and the accepted sample were gathered under.
    #[must_use]
    pub const fn layout_generation(&self) -> u64 {
        self.layout_generation
    }

    /// The phase, read off the accepted count and the horizon.
    #[must_use]
    pub const fn phase(&self) -> crate::feature::standardisation::StandardisationPhase {
        crate::feature::standardisation::StandardisationPhase::from_progress(self.accepted(), self.horizon())
    }

    /// `true` once the horizon is reached and no prior mass remains.
    #[must_use]
    pub const fn is_complete(&self) -> bool {
        self.accepted() >= self.horizon()
    }

    /// The share of base mass retired so far, $a_n = n/N_\text{init}$.
    ///
    /// A horizon of zero has no mass to retire and reports the whole of it
    /// retired, which is the reading that agrees with the phase.
    #[must_use]
    pub fn retired_share(&self) -> f64 {
        if self.horizon() == 0 {
            return 1.0;
        }
        #[allow(clippy::cast_precision_loss)]
        let share = self.accepted() as f64 / self.horizon() as f64;
        share.min(1.0)
    }

    /// Takes one accepted observation into the sample.
    ///
    /// Step 3 of (´alg:standardisation:batch-initialisation´): the running
    /// mean and second moment advance by the numerically stable one-pass
    /// update, and non-finite entries are skipped element by element so one
    /// bad entry cannot poison a position for the rest of the ramp.
    pub fn observe(&mut self, phi_raw: &[f64]) {
        self.accumulator.observe(phi_raw);
    }

    /// Rebases the ramp on what a lifecycle event has just published.
    ///
    /// The published moments become the new base and the sample gathered
    /// under the layout being replaced is discarded, because a part-gathered
    /// vector at one width says nothing about a position that did not exist
    /// when gathering began. No published position is rolled back to its own
    /// prior (´req:standardisation:lifecycle-entries´).
    ///
    /// A ramp that has already reached its horizon stays there. A lifecycle
    /// event adds positions to a coordinate system; it does not un-reach a
    /// horizon, and returning an in-service instance to `WaitingForInit`
    /// would make every later Sentinel registration a fresh cold start. Under
    /// either branch the ramp publishes, immediately afterwards, exactly what
    /// the lifecycle published.
    pub fn rebase(&mut self, means: &[f64], variances: &[f64], layout_generation: u64) {
        let stays_in_service = self.is_complete();

        self.base_means = means.to_vec();
        self.base_variances = variances.to_vec();
        self.accumulator.reset(means.len());
        self.layout_generation = layout_generation;

        if stays_in_service {
            // Hold the horizon, and carry the published moments as the sample
            // so that the mixture at full weight reproduces them rather than
            // collapsing every variance onto the floor.
            #[allow(clippy::cast_precision_loss)]
            let n = self.accumulator.target as f64;
            self.accumulator.count = self.accumulator.target;
            self.accumulator.running_mean = means.to_vec();
            self.accumulator.running_m2 = variances.iter().map(|&v| v * n).collect();
        }
    }

    /// The moments to publish at the current count.
    ///
    /// Step 4 of (´alg:standardisation:batch-initialisation´). At count $n$,
    /// with $a_n$ the retired share, every non-bias position publishes
    ///
    /// ```text
    /// mu = (1 - a) * mu_0 + a * mu_e
    /// v  = max(v_floor, (1 - a) * v_0 + a * v_e + a * (1 - a) * (mu_e - mu_0)^2)
    /// ```
    ///
    /// The third variance term is the between-population variance and is
    /// published rather than dropped: a variance without it would be neither
    /// the base's nor the sample's, and would understate the spread worst in
    /// the middle of the ramp, where $a(1-a)$ is largest and a reader has
    /// least other evidence about which coordinate system a result came from.
    ///
    /// At the horizon $a$ is one, so both the base weight and the
    /// between-population term are exactly zero and what is published is the
    /// empirical moments themselves — not a close approximation of them, and
    /// with no rate left running underneath them. Before the first accepted
    /// observation the base is published unchanged.
    ///
    /// The bias position takes no part at any count.
    #[must_use]
    pub fn publish(&self, classes: &[FeatureClass], v_floor: f64) -> (Vec<f64>, Vec<f64>) {
        let mut means = self.base_means.clone();
        let mut variances = self.base_variances.clone();

        let accepted = self.accepted();
        if accepted == 0 {
            return (means, variances);
        }

        let a = self.retired_share();
        #[allow(clippy::cast_precision_loss)]
        let n = accepted as f64;

        for j in 0..means.len() {
            if matches!(classes.get(j), Some(FeatureClass::Bias)) {
                continue;
            }
            let (Some(&mu_e), Some(&m2)) = (self.accumulator.running_mean.get(j), self.accumulator.running_m2.get(j)) else {
                continue;
            };
            let v_e = m2 / n;
            let mu_0 = means[j];
            let v_0 = variances[j];
            let gap = mu_e - mu_0;

            means[j] = (1.0 - a) * mu_0 + a * mu_e;
            variances[j] = ((1.0 - a) * v_0 + a * v_e + a * (1.0 - a) * gap * gap).max(v_floor);
        }

        (means, variances)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {

    use super::*;

    /// Each observed vector advances the count by exactly one, starting from a
    /// freshly built accumulator at nothing. The count is what the completion
    /// test reads and what the variance is divided by, so it must track
    /// observations rather than, say, the number of positions written on each
    /// pass.
    ///
    /// ´claim:feature:each-observed-vector-advances-the-accumulators-count-by-one´
    /// ´test:unit:bootstrap-observe-updates-count´
    #[test]
    fn bootstrap_observe_updates_count() {
        let mut acc = BootstrapAccumulator::new(5, 10);
        assert_eq!(acc.count(), 0);

        acc.observe(&[1.0; 5]);
        assert_eq!(acc.count(), 1);

        acc.observe(&[2.0; 5]);
        assert_eq!(acc.count(), 2);
    }

    /// An accumulator declares itself complete the moment its count reaches the
    /// target, and not one observation earlier. Completion is what releases a
    /// slot into service, so the boundary decides whether the model starts from
    /// the sample size it was configured to require or from one short of it.
    ///
    /// ´claim:feature:an-accumulator-reports-complete-on-reaching-its-target-and-not-before´
    /// ´test:unit:bootstrap-is-complete-at-target´
    #[test]
    fn bootstrap_is_complete_at_target() {
        let mut acc = BootstrapAccumulator::new(3, 2);
        assert!(!acc.is_complete());

        acc.observe(&[1.0; 3]);
        assert!(!acc.is_complete());

        acc.observe(&[2.0; 3]);
        assert!(acc.is_complete());
    }

    /// From one observation the statistics are that vector as the mean, zero
    /// variance throughout, and a count of one. A single point locates a
    /// distribution but says nothing about its width, and the accumulator
    /// reports exactly that — the count travelling alongside so a consumer can
    /// see how little the zero rests on.
    ///
    /// ´claim:feature:a-single-observation-yields-that-vector-as-the-mean-with-zero-variance´
    /// ´test:unit:bootstrap-complete-returns-stats´
    #[test]
    fn bootstrap_complete_returns_stats() {
        let mut acc = BootstrapAccumulator::new(2, 1);
        acc.observe(&[3.0, 5.0]);

        let stats = acc.complete().expect("should have stats");
        assert_eq!(stats.mean, vec![3.0, 5.0]);
        assert_eq!(stats.variance, vec![0.0, 0.0]); // Single observation → zero variance
        assert_eq!(stats.count, 1);
    }

    /// An accumulator that has seen nothing yields no statistics at all, rather
    /// than a vector of zero means and zero variances. Zeros would be
    /// indistinguishable from a genuinely measured constant stream, and a
    /// consumer that standardised against them would be dividing by a variance
    /// nothing supports; the absence is made explicit instead.
    ///
    /// ´claim:feature:an-accumulator-that-has-seen-nothing-yields-no-statistics-rather-than-zeros´
    /// ´test:unit:bootstrap-complete-none-when-empty´
    #[test]
    fn bootstrap_complete_none_when_empty() {
        let acc = BootstrapAccumulator::new(5, 10);
        assert!(acc.complete().is_none());
    }

    /// The statistics an accumulator reports are the population mean and
    /// population variance of exactly the values it was shown — five
    /// observations spread evenly about five come back as a mean of five and a
    /// variance of two. The figures are built incrementally as observations
    /// arrive, so nothing needs to be retained but the running pair, and the
    /// answer is the same as a second pass over the stored sample would give.
    ///
    /// ´claim:feature:the-accumulator-reports-the-population-mean-and-variance-of-what-it-was-shown´
    /// ´test:unit:bootstrap-welford-variance´
    #[test]
    fn bootstrap_welford_variance() {
        // Test with known mean=5, variance=2
        // Observations: [3, 4, 5, 6, 7] → mean=5, var=2
        let mut acc = BootstrapAccumulator::new(1, 5);
        for &x in &[3.0, 4.0, 5.0, 6.0, 7.0] {
            acc.observe(&[x]);
        }

        let stats = acc.complete().unwrap();
        assert!((stats.mean[0] - 5.0).abs() < 1e-10);
        assert!((stats.variance[0] - 2.0).abs() < 1e-10);
    }

    /// The batch-init accumulator counts the same way over full feature
    /// vectors: fifty observations of a ten-wide vector leave it at fifty. It
    /// tracks the whole standardisation vector rather than one slot's features,
    /// but the accounting that governs when initialisation ends is identical.
    ///
    /// (´claim:feature:each-observed-vector-advances-the-accumulators-count-by-one´)
    /// ´test:unit:batch-init-observe´
    #[test]
    fn batch_init_observe() {
        let mut acc = BatchInitAccumulator::new(10, 100);
        assert_eq!(acc.count(), 0);

        for _ in 0..50 {
            acc.observe(&[1.0; 10]);
        }
        assert_eq!(acc.count(), 50);
        assert!(!acc.is_complete());
    }

    /// Batch initialisation reports the same population statistics across a
    /// whole vector at once: three observations that each step up by one leave
    /// every position with its own mean and the common variance of a
    /// three-point ramp. The statistics that standardisation will start from
    /// are therefore measured from real traffic rather than assumed.
    ///
    /// (´claim:feature:the-accumulator-reports-the-population-mean-and-variance-of-what-it-was-shown´)
    /// ´test:unit:batch-init-complete´
    #[test]
    fn batch_init_complete() {
        let mut acc = BatchInitAccumulator::new(3, 3);
        acc.observe(&[1.0, 2.0, 3.0]);
        acc.observe(&[2.0, 3.0, 4.0]);
        acc.observe(&[3.0, 4.0, 5.0]);

        assert!(acc.is_complete());
        let stats = acc.complete().unwrap();

        // Mean: [2, 3, 4]
        assert!((stats.mean[0] - 2.0).abs() < 1e-10);
        assert!((stats.mean[1] - 3.0).abs() < 1e-10);
        assert!((stats.mean[2] - 4.0).abs() < 1e-10);

        // Variance: 2/3 for each (population variance of [1,2,3], [2,3,4], [3,4,5])
        let expected_var = 2.0 / 3.0;
        assert!((stats.variance[0] - expected_var).abs() < 1e-10);
        assert!((stats.variance[1] - expected_var).abs() < 1e-10);
        assert!((stats.variance[2] - expected_var).abs() < 1e-10);
    }

    /// A reset returns the count to nothing and re-sizes the running mean and
    /// second-moment vectors to the width it is given, discarding what was
    /// accumulated at the old width. A lifecycle event that changes the feature
    /// vector's dimension invalidates every position that came before it, so
    /// initialisation restarts rather than trying to carry statistics across a
    /// renumbering.
    ///
    /// ´claim:feature:a-reset-clears-the-count-and-re-sizes-the-running-vectors-to-the-new-width´
    /// ´test:unit:batch-init-reset´
    #[test]
    fn batch_init_reset() {
        let mut acc = BatchInitAccumulator::new(5, 10);
        acc.observe(&[1.0; 5]);
        acc.observe(&[2.0; 5]);
        assert_eq!(acc.count(), 2);

        acc.reset(10);
        assert_eq!(acc.count(), 0);
        assert_eq!(acc.running_mean.len(), 10);
        assert_eq!(acc.running_m2.len(), 10);
    }

    /// A tiny spread riding on an enormous common offset is still recovered:
    /// three values ten billion apart from zero but one apart from each other
    /// give back both the large mean and the small variance. Computing the
    /// variance from a sum of squares would lose the difference entirely to
    /// cancellation here, whereas the incremental form works in deviations from
    /// the running mean and never forms the large square at all.
    ///
    /// ´claim:feature:a-small-variance-survives-a-huge-common-offset-without-cancellation´
    /// ´test:unit:welford-numerical-stability´
    #[test]
    fn welford_numerical_stability() {
        // Test with large values that could cause numerical issues
        let mut acc = BootstrapAccumulator::new(1, 3);
        acc.observe(&[1e10 + 1.0]);
        acc.observe(&[1e10 + 2.0]);
        acc.observe(&[1e10 + 3.0]);

        let stats = acc.complete().unwrap();
        // Mean should be 1e10 + 2
        assert!((stats.mean[0] - (1e10 + 2.0)).abs() < 1e-5);
        // Variance should be 2/3
        assert!((stats.variance[0] - 2.0 / 3.0).abs() < 1e-5);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Additional bootstrap tests
    // ─────────────────────────────────────────────────────────────────────────

    /// Every position keeps its own running pair: over the same five
    /// observations a constant position reports zero variance, a steadily
    /// rising one reports a modest spread, and one that stays flat then jumps
    /// enormously reports a huge spread — none of them influenced by the
    /// others. Features share only the vector they arrive in, so a single
    /// violent slot must not widen the variance the quiet slots are
    /// standardised against.
    ///
    /// ´claim:feature:each-position-accumulates-its-own-mean-and-variance-independently-of-its-neighbours´
    /// ´test:unit:bootstrap-multi-feature-variance´
    #[test]
    fn bootstrap_multi_feature_variance() {
        // Test variance computation with multiple independent features.
        let mut acc = BootstrapAccumulator::new(3, 5);

        // Feature 0: constant 1.0 → variance = 0
        // Feature 1: [1, 2, 3, 4, 5] → mean = 3, variance = 2
        // Feature 2: [0, 0, 0, 0, 100] → mean = 20, variance = 1600
        let observations = [
            [1.0, 1.0, 0.0],
            [1.0, 2.0, 0.0],
            [1.0, 3.0, 0.0],
            [1.0, 4.0, 0.0],
            [1.0, 5.0, 100.0],
        ];

        for obs in &observations {
            acc.observe(obs);
        }

        let stats = acc.complete().unwrap();

        // Feature 0: constant
        assert_eq!(stats.mean[0].to_bits(), 1.0f64.to_bits());
        assert!((stats.variance[0]).abs() < 1e-10, "constant feature variance");

        // Feature 1: mean=3, var=2
        assert!((stats.mean[1] - 3.0).abs() < 1e-10);
        assert!((stats.variance[1] - 2.0).abs() < 1e-10);

        // Feature 2: mean=20, var=1600
        assert!((stats.mean[2] - 20.0).abs() < 1e-10);
        assert!((stats.variance[2] - 1600.0).abs() < 1e-10);
    }

    /// Half way to a target of a hundred, batch initialisation still reports
    /// itself unfinished. Nothing about having collected a substantial sample
    /// short-circuits the requirement, so the model does not slip into service
    /// on statistics gathered from half the traffic the operator asked for.
    ///
    /// (´claim:feature:an-accumulator-reports-complete-on-reaching-its-target-and-not-before´)
    /// ´test:unit:batch-init-not-complete-below-target´
    #[test]
    fn batch_init_not_complete_below_target() {
        let mut acc = BatchInitAccumulator::new(5, 100);

        for _ in 0..50 {
            acc.observe(&[1.0; 5]);
        }

        assert!(!acc.is_complete());
        assert_eq!(acc.count(), 50);
    }

    /// Checked after every single observation on the way up, batch
    /// initialisation stays unfinished through the fourth and flips on the
    /// fifth against a target of five. The transition happens once, at the
    /// stated count — never a step early on a rounding of the comparison, and
    /// never late.
    ///
    /// (´claim:feature:an-accumulator-reports-complete-on-reaching-its-target-and-not-before´)
    /// ´test:unit:batch-init-completes-exactly-at-target´
    #[test]
    fn batch_init_completes_exactly_at_target() {
        let mut acc = BatchInitAccumulator::new(3, 5);

        for i in 0..4 {
            acc.observe(&[f64::from(i); 3]);
            assert!(!acc.is_complete());
        }

        acc.observe(&[4.0; 3]);
        assert!(acc.is_complete());
        assert_eq!(acc.count(), 5);
    }

    /// The mean and variance vectors come back exactly as wide as the vectors
    /// that were observed. Standardisation indexes these two records by feature
    /// position, so a width that disagreed with the feature vector would silently
    /// standardise each slot against some other slot's statistics.
    ///
    /// ´claim:feature:the-reported-statistics-are-exactly-as-wide-as-the-vectors-observed´
    /// ´test:unit:batch-init-stats-dimension´
    #[test]
    fn batch_init_stats_dimension() {
        let mut acc = BatchInitAccumulator::new(10, 1);
        acc.observe(&[0.0; 10]);

        let stats = acc.complete().unwrap();
        assert_eq!(stats.mean.len(), 10);
        assert_eq!(stats.variance.len(), 10);
    }

    /// A target of zero is satisfied before anything is observed, yet the
    /// accumulator still refuses to produce statistics it has no observations
    /// for. Completion and having something to report are separate questions,
    /// so a configuration that asks for no warm-up cannot trick the accumulator
    /// into inventing a mean and a variance.
    ///
    /// ´claim:feature:completion-and-having-statistics-to-report-are-separate-questions´
    /// ´test:unit:bootstrap-target-zero´
    #[test]
    fn bootstrap_target_zero() {
        // Edge case: target = 0 means complete immediately.
        let acc = BootstrapAccumulator::new(5, 0);
        assert!(acc.is_complete());
        // Complete with no observations returns None.
        assert!(acc.complete().is_none());
    }

    /// One observation across two positions gives back both values as the means
    /// and zero as each variance, whatever the two values were. The zero is not
    /// a placeholder standing in for an unknown spread but the true population
    /// variance of a sample of one, and the accompanying count of one is what
    /// tells a consumer to treat it as such.
    ///
    /// (´claim:feature:a-single-observation-yields-that-vector-as-the-mean-with-zero-variance´)
    /// ´test:unit:bootstrap-single-observation´
    #[test]
    fn bootstrap_single_observation() {
        let mut acc = BootstrapAccumulator::new(2, 1);
        acc.observe(&[3.0, 7.0]);

        let stats = acc.complete().unwrap();
        assert_eq!(stats.mean, vec![3.0, 7.0]);
        // Single observation → variance = 0
        assert_eq!(stats.variance, vec![0.0, 0.0]);
        assert_eq!(stats.count, 1);
    }

    /// A stream carrying a NaN at one position and an infinity at another,
    /// mid-accumulation, leaves every accumulated mean and variance finite and
    /// no variance negative: the offending entries are passed over one element
    /// at a time, so the positions beside them keep accumulating and the
    /// affected positions come back describing only the observations that were
    /// numbers. The alternative is not a locally wrong statistic but a
    /// permanently poisoned one — every later update is a difference against
    /// the running pair, so one bad entry keeps that position non-finite for
    /// the rest of the accumulation, and the statistics standardisation
    /// divides by are published from the first accepted observation onward, a
    /// share of the base mass at a time.
    ///
    /// ´claim:feature:a-non-finite-entry-is-skipped-per-element-so-it-cannot-poison-the-running-pair´
    /// ´test:unit:init-stats-are-finite´
    #[test]
    fn init_stats_are_finite() {
        let mut acc = BootstrapAccumulator::new(3, 10);

        // Mix of small and large values, with two non-finite entries partway in.
        for i in 0..10 {
            let third = if i == 4 { f64::NAN } else { f64::from(i).sin() };
            let second = if i == 7 { f64::INFINITY } else { f64::from(i) * 10.0 };
            acc.observe(&[f64::from(i) * 0.1, second, third]);
        }

        let stats = acc.complete().unwrap();

        for (j, (&mean, &var)) in stats.mean.iter().zip(stats.variance.iter()).enumerate() {
            assert!(mean.is_finite(), "mean[{j}] is not finite: {mean}");
            assert!(var.is_finite(), "variance[{j}] is not finite: {var}");
            assert!(var >= 0.0, "variance[{j}] is negative: {var}");
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Cold prior-mass ramp
    // ─────────────────────────────────────────────────────────────────────────

    /// A four-wide test layout whose leading position is the bias.
    fn ramp_classes() -> Vec<FeatureClass> {
        vec![
            FeatureClass::Bias,
            FeatureClass::ZScore,
            FeatureClass::Binary,
            FeatureClass::Cusum,
        ]
    }

    /// A ramp over that layout, based on those classes' own priors.
    fn ramp(horizon: usize) -> ColdRamp {
        let classes = ramp_classes();
        let (means, variances) = crate::feature::standardisation::init_from_priors(&classes);
        ColdRamp::new(means, variances, horizon, 0)
    }

    /// The phase is a reading of the accepted count and never a second piece of
    /// state: a ramp is waiting at zero, transitioning from the first accepted
    /// observation, still transitioning one short of the horizon, and in
    /// service exactly at it. The three values are what a host is told the
    /// coordinate system is doing, so a phase that could disagree with the
    /// count it accompanies would be two answers to one question — and the
    /// answer a host acted on would be whichever it happened to read.
    ///
    /// ´claim:feature:the-standardisation-phase-is-read-off-the-accepted-count-rather-than-stored-beside-it´
    /// ´test:unit:cold-ramp-phase-reads-off-the-count´
    #[test]
    fn cold_ramp_phase_reads_off_the_count() {
        use crate::feature::standardisation::StandardisationPhase;

        let mut r = ramp(5);
        assert_eq!(r.phase(), StandardisationPhase::WaitingForInit);
        assert_eq!(r.accepted(), 0);

        r.observe(&[1.0, 0.0, 0.0, 0.0]);
        assert_eq!(r.phase(), StandardisationPhase::Transitioning);

        for _ in 0..3 {
            r.observe(&[1.0, 0.0, 0.0, 0.0]);
        }
        assert_eq!(r.accepted(), 4, "one short of the horizon");
        assert_eq!(r.phase(), StandardisationPhase::Transitioning);

        r.observe(&[1.0, 0.0, 0.0, 0.0]);
        assert_eq!(r.accepted(), 5);
        assert_eq!(r.phase(), StandardisationPhase::InService);
        assert!(r.is_complete());
    }

    /// At the horizon the published moments are the empirical moments of the
    /// accepted sample bit for bit, not within a tolerance: the retired share
    /// is exactly one, so the base weight and the between-population term are
    /// both exactly zero and nothing of the priors survives the arithmetic.
    /// This is the property that makes the ramp a redistribution of the old
    /// wholesale replacement rather than a different destination — the same
    /// endpoint, reached in equal steps instead of one.
    ///
    /// ´claim:feature:the-ramps-endpoint-is-the-empirical-moments-exactly-rather-than-within-a-tolerance´
    /// ´test:unit:cold-ramp-endpoint-is-exactly-empirical´
    #[test]
    fn cold_ramp_endpoint_is_exactly_empirical() {
        let classes = ramp_classes();
        let mut r = ramp(4);
        let sample = [
            [1.0, 2.0, 0.0, 7.0],
            [1.0, 4.0, 1.0, 9.0],
            [1.0, 6.0, 0.0, 11.0],
            [1.0, 8.0, 1.0, 13.0],
        ];
        for row in &sample {
            r.observe(row);
        }

        let empirical = r.accumulator().complete().expect("four observations");
        let (means, variances) = r.publish(&classes, 1e-4);

        for j in 1..4 {
            assert_eq!(
                means[j].to_bits(),
                empirical.mean[j].to_bits(),
                "position {j} should publish the empirical mean exactly at the horizon"
            );
            assert_eq!(
                variances[j].to_bits(),
                empirical.variance[j].max(1e-4).to_bits(),
                "position {j} should publish the empirical variance at the horizon"
            );
        }
    }

    /// The variance published part way along the ramp is the variance of the
    /// mixture and not a mixture of the two variances: the term carrying how
    /// far the observed mean has travelled from the base mean is published
    /// with them. Here the base mean is zero and the sample's is four, so at
    /// the half-way point that term contributes a quarter of sixteen — and a
    /// variance that dropped it would report a spread narrower than either
    /// population's own, understating it worst exactly where the two disagree
    /// most and where a reader has least other evidence about which coordinate
    /// system a result came from.
    ///
    /// ´claim:feature:the-mixed-variance-carries-the-between-population-term-rather-than-blending-two-variances´
    /// ´test:unit:cold-ramp-variance-carries-the-between-population-term´
    #[test]
    fn cold_ramp_variance_carries_the_between_population_term() {
        let classes = ramp_classes();
        let mut r = ramp(4);
        // Position 1 is a z-score: base moments (0, 1). Two observations at
        // exactly four leave the sample's mean at four and its variance at
        // nothing, so every part of the published spread is attributable.
        r.observe(&[1.0, 4.0, 0.0, 0.0]);
        r.observe(&[1.0, 4.0, 0.0, 0.0]);
        assert_eq!(r.accepted(), 2, "half way to the horizon");

        let (means, variances) = r.publish(&classes, 1e-4);

        // a = 0.5: the mean is half way between zero and four.
        assert!((means[1] - 2.0).abs() < 1e-12, "mixed mean was {}", means[1]);

        // v = 0.5*1 + 0.5*0 + 0.5*0.5*(4 - 0)^2 = 0.5 + 4.0.
        let two_term = 0.5 * 1.0 + 0.5 * 0.0;
        let between = 0.5 * 0.5 * 16.0;
        assert!(
            (variances[1] - (two_term + between)).abs() < 1e-12,
            "mixed variance was {}, expected {}",
            variances[1],
            two_term + between,
        );
        assert!(
            variances[1] > two_term,
            "dropping the between-population term would publish {two_term}, narrower than either population",
        );
    }

    /// The bias position takes no part in the ramp at any count: it is a
    /// constant the model reads directly and nothing standardises it, so a
    /// mean and a variance measured for it would describe a quantity that is
    /// never divided by them. Here the accepted sample carries a wildly
    /// varying value at the bias index and the published bias moments are
    /// still the ones the ramp started with, while its neighbours move.
    ///
    /// ´claim:feature:the-bias-position-takes-no-part-in-the-ramp-at-any-count´
    /// ´test:unit:cold-ramp-leaves-the-bias-position-alone´
    #[test]
    fn cold_ramp_leaves_the_bias_position_alone() {
        let classes = ramp_classes();
        let (base_means, base_variances) = crate::feature::standardisation::init_from_priors(&classes);
        let mut r = ramp(4);

        r.observe(&[-50.0, 1.0, 1.0, 1.0]);
        r.observe(&[900.0, 1.0, 1.0, 1.0]);

        let (means, variances) = r.publish(&classes, 1e-4);
        assert_eq!(means[0].to_bits(), base_means[0].to_bits(), "the bias mean must not move");
        assert_eq!(
            variances[0].to_bits(),
            base_variances[0].to_bits(),
            "the bias variance must not move"
        );
        assert!(
            (means[1] - base_means[1]).abs() > 0.0,
            "a neighbouring position must move, else the test proves nothing",
        );
    }

    /// Each accepted observation retires exactly one horizon-th of the base
    /// mass and no more: against a constant sample the published mean advances
    /// in equal steps from the base to the sample's own value, and the largest
    /// single step is the same size as the smallest. The equal share is the
    /// whole point of the ramp — it is what replaced a coordinate system that
    /// moved its entire prior-to-empirical distance between two adjacent
    /// requests with one that discloses the same distance a share at a time.
    ///
    /// ´claim:feature:each-accepted-observation-retires-exactly-one-horizon-th-of-the-base-mass´
    /// ´test:unit:cold-ramp-retires-one-equal-share-per-observation´
    #[test]
    fn cold_ramp_retires_one_equal_share_per_observation() {
        let classes = ramp_classes();
        let mut r = ramp(10);
        let mut published = vec![r.publish(&classes, 1e-4).0[1]];

        for _ in 0..10 {
            r.observe(&[1.0, 5.0, 0.0, 0.0]);
            published.push(r.publish(&classes, 1e-4).0[1]);
        }

        // Base mean zero, sample mean five, ten equal shares of one half.
        let steps: Vec<f64> = published.windows(2).map(|w| w[1] - w[0]).collect();
        assert_eq!(steps.len(), 10);
        for (i, step) in steps.iter().enumerate() {
            assert!(
                (step - 0.5).abs() < 1e-12,
                "step {i} was {step}, and every step should retire one tenth of the distance",
            );
        }
        assert!((published[10] - 5.0).abs() < 1e-12, "the endpoint is the sample's own mean");
    }

    /// A lifecycle event rebases the ramp on the moments it has just published
    /// and discards the sample gathered under the layout it replaced: the
    /// count returns to zero, the new base is what was published rather than
    /// the original priors, and the layout the ramp will accept observations
    /// under has moved on. Rolling the surviving positions back to their own
    /// priors instead would undo measurement the instance had already paid
    /// for, and carrying the old sample forward would attribute one position's
    /// distribution to another position's index.
    ///
    /// ´claim:feature:a-lifecycle-rebase-takes-the-published-moments-as-the-new-base-and-discards-the-sample´
    /// ´test:unit:cold-ramp-rebase-takes-published-moments´
    #[test]
    fn cold_ramp_rebase_takes_published_moments() {
        let classes = ramp_classes();
        let mut r = ramp(10);
        for _ in 0..4 {
            r.observe(&[1.0, 5.0, 0.0, 0.0]);
        }
        let (published_means, published_variances) = r.publish(&classes, 1e-4);
        assert_eq!(r.accepted(), 4);

        r.rebase(&published_means, &published_variances, 7);

        assert_eq!(r.accepted(), 0, "the incompatible sample is discarded");
        assert_eq!(r.layout_generation(), 7);
        assert_eq!(
            r.base_means(),
            published_means.as_slice(),
            "the new base is what was published"
        );
        let (after, _) = r.publish(&classes, 1e-4);
        assert_eq!(
            after, published_means,
            "a rebased ramp publishes what the lifecycle published, not the original priors",
        );
    }

    /// A restored ramp recomputes its position against the horizon the
    /// restoring instance is configured with, and a count at or past that
    /// horizon is complete on arrival: a sample of sixty against a horizon of
    /// fifty publishes the empirical moments and reports itself in service,
    /// while the same sixty against a horizon of a hundred is still
    /// transitioning with three fifths of the base mass retired. Carrying a
    /// horizon in the checkpoint instead would put a second authority beside
    /// the configuration, and a restore would then have two answers for how
    /// far along it was.
    ///
    /// ´claim:feature:a-restored-ramp-recomputes-against-the-configured-horizon-rather-than-a-carried-one´
    /// ´test:unit:cold-ramp-restore-recomputes-against-configured-horizon´
    #[test]
    fn cold_ramp_restore_recomputes_against_configured_horizon() {
        use crate::feature::standardisation::StandardisationPhase;

        let classes = ramp_classes();
        let (base_means, base_variances) = crate::feature::standardisation::init_from_priors(&classes);
        let running_mean = vec![1.0, 5.0, 0.0, 0.0];
        let running_m2 = vec![0.0; 4];

        let shrunk = ColdRamp::from_parts(
            base_means.clone(),
            base_variances.clone(),
            running_mean.clone(),
            running_m2.clone(),
            60,
            50,
            0,
        );
        assert_eq!(shrunk.phase(), StandardisationPhase::InService);
        assert!(shrunk.is_complete());
        assert!((shrunk.retired_share() - 1.0).abs() < 1e-12);
        let (means, _) = shrunk.publish(&classes, 1e-4);
        assert_eq!(
            means[1].to_bits(),
            5.0f64.to_bits(),
            "a count past the horizon publishes the empirical moments"
        );

        let widened = ColdRamp::from_parts(base_means, base_variances, running_mean, running_m2, 60, 100, 0);
        assert_eq!(widened.phase(), StandardisationPhase::Transitioning);
        assert!((widened.retired_share() - 0.6).abs() < 1e-12);
        let (means, _) = widened.publish(&classes, 1e-4);
        assert!((means[1] - 3.0).abs() < 1e-12, "three fifths of the way from zero to five");
    }

    /// A non-finite entry in an accepted observation cannot reach the
    /// published coordinate system: the entry is skipped where it arrives, and
    /// every mean and variance the ramp publishes is finite with no variance
    /// negative. The guard matters more here than at the accumulator, because
    /// what these moments are divided into is every feature every model reads
    /// — a single poisoned position would not produce one wrong answer but
    /// would put a NaN through the quadratic forms of every assessment after
    /// it.
    ///
    /// (´claim:feature:a-non-finite-entry-is-skipped-per-element-so-it-cannot-poison-the-running-pair´)
    /// ´test:unit:cold-ramp-publishes-finite-moments-through-non-finite-input´
    #[test]
    fn cold_ramp_publishes_finite_moments_through_non_finite_input() {
        let classes = ramp_classes();
        let mut r = ramp(6);
        for i in 0..6 {
            let second = if i == 2 { f64::NAN } else { f64::from(i) };
            let third = if i == 4 { f64::INFINITY } else { f64::from(i) * 0.5 };
            r.observe(&[1.0, second, third, f64::from(i)]);
        }

        let (means, variances) = r.publish(&classes, 1e-4);
        for (j, (&mu, &v)) in means.iter().zip(variances.iter()).enumerate() {
            assert!(mu.is_finite(), "published mean[{j}] is not finite: {mu}");
            assert!(v.is_finite(), "published variance[{j}] is not finite: {v}");
            assert!(v >= 0.0, "published variance[{j}] is negative: {v}");
        }
    }

    /// The batch-initialisation accumulator carries the same guard as the
    /// per-Sentinel one, presented the same way: a NaN and an infinity at two
    /// positions leave every published statistic finite. The two accumulators
    /// are separate loops over separate state, so the guard holds in each only
    /// because it is written in each — and this one feeds the standardisation
    /// every model reads rather than one slot's.
    ///
    /// (´claim:feature:a-non-finite-entry-is-skipped-per-element-so-it-cannot-poison-the-running-pair´)
    /// ´test:unit:batch-init-stats-are-finite´
    #[test]
    fn batch_init_stats_are_finite() {
        let mut acc = BatchInitAccumulator::new(3, 10);

        for i in 0..10 {
            let third = if i == 4 { f64::NAN } else { f64::from(i).sin() };
            let second = if i == 7 { f64::NEG_INFINITY } else { f64::from(i) * 10.0 };
            acc.observe(&[f64::from(i) * 0.1, second, third]);
        }

        let stats = acc.complete().unwrap();

        for (j, (&mean, &var)) in stats.mean.iter().zip(stats.variance.iter()).enumerate() {
            assert!(mean.is_finite(), "mean[{j}] is not finite: {mean}");
            assert!(var.is_finite(), "variance[{j}] is not finite: {var}");
            assert!(var >= 0.0, "variance[{j}] is negative: {var}");
        }
    }
}
