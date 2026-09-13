// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Cholesky recomputation strategy, firing on a counter or on conditioning
//! (´dec:posterior:recomputation-trigger´).
//!
//! Implements the Cholesky recomputation the cadence schedules
//! (´dec:posterior:repair-cascade´) with adaptive interval halving and
//! recovery. There is one attempt: plain Cholesky of the matrix the model
//! will hold, floored where the spectrum asked for it
//! (´dec:posterior:spectral-floor´). A refusal offers nothing, and the
//! terminus retains the maintained covariance and flags it.
//!
//! # Cross-References
//!
//! - (´dec:posterior:adaptive-cadence´) — the interval halving and the
//!   slow recovery this file is the source of truth for
//! - (´alg:gaussian:synchronisation-monitor´) — the periodic
//!   precision/covariance synchronisation

use std::time::Instant;

use faer::Mat;

use crate::linalg::bridge::{CholeskyCallSite, CholeskyInverseResult, cholesky_inverse};
use crate::linalg::symmetric::SymmetricMatrix;

// ═══════════════════════════════════════════════════════════════════════════════
// Constants
// ═══════════════════════════════════════════════════════════════════════════════

/// Floor for the effective recomputation interval.
///
/// Below this, the O(p³) Cholesky cost (~80 ms) amortised over few
/// labels competes with the SM update cost itself. The floor the
/// adaptive cadence halves against (´dec:posterior:adaptive-cadence´),
/// at the bound the configuration surface fixes (´tab:config:risk-model´).
///
/// ´const:assayer:recomputation-cadence-floor´ (´alg:const:count´)
/// ´const:assayer:recomputation-cadence-floor-count-100´
pub const N_RECOMPUTE_FLOOR: u32 = 100;

/// Consecutive clean recomputes required for interval recovery.
///
/// The run of clean outcomes that restores the configured interval, the
/// slow half of down-fast-up-slowly (´dec:posterior:adaptive-cadence´).
///
/// ´const:assayer:cadence-recovery-run´ (´alg:const:count´)
/// ´const:assayer:cadence-recovery-run-count-3´
pub const N_RECOVERY_CLEAN: u32 = 3;

/// Default growth factor the diagonal ratio must clear to call for a rebuild.
///
/// The fast check tests the ratio's *growth since the last rebuild* rather
/// than its absolute value, so the figure is a multiple and not a threshold
/// (´alg:gaussian:condition-adaptive-recompute´). Two is the default and the
/// reason is a rate: the arm can fire at most once per doubling of the ratio,
/// because a rebuild resets the baseline to what it has just measured, so its
/// firing rate is logarithmic in how far the matrix has grown rather than
/// proportional to the labels absorbed. Over the twelve hundred labels the
/// drift study drove, with the largest diagonal entry climbing from the prior
/// precision to a few hundred, that is about a dozen firings.
///
/// A smaller factor would fire on the ordinary breathing of the diagonal
/// under decay and replenishment; a much larger one would let the proxy climb
/// through an order of magnitude before the counter's own interval elapsed,
/// which makes the arm redundant with the counter it exists to anticipate.
///
/// ´const:assayer:conditioning-growth-trigger´ (´alg:const:scalar´)
/// ´const:assayer:conditioning-growth-trigger-scalar-2p0´
pub const DEFAULT_KAPPA_GROWTH_FACTOR: f64 = 2.0;

/// Synchronisation-error threshold per model dimension.
///
/// The full threshold is `10⁻⁶ × p`, where `p` is that model's current
/// width (´def:config:synchronisation-threshold´).
///
/// ´const:assayer:synchronisation-error-threshold-per-dimension´ (´alg:const:scalar´)
/// ´const:assayer:synchronisation-error-threshold-per-dimension-scalar-1en6´
pub const SYNC_ERROR_THRESHOLD_PER_DIMENSION: f64 = 1e-6;

/// Returns the synchronisation-error threshold for one model width.
#[must_use]
#[allow(clippy::cast_precision_loss)] // Model dimensions are far below f64's exact integer range.
pub fn synchronisation_error_threshold(p: usize) -> f64 {
    SYNC_ERROR_THRESHOLD_PER_DIMENSION * p as f64
}

/// The growth factor in the standard error bound for a width-`p` inner
/// product, `γ_p = p·u / (1 − p·u)` at the unit roundoff `u = ε/2`.
///
/// This is the factor the standard bound for a floating-point matrix product
/// carries: for `P = fl(AB)` at width `p`, `|P − AB| ≤ γ_p · |A||B|`
/// entrywise (Higham, *Accuracy and Stability of Numerical Algorithms*, 2nd
/// ed., section 3.5). It is the only quantity the resolution of a
/// synchronisation reading needs beyond the operands themselves, and like the
/// spectral floor's own constant it is derived from the arithmetic rather than
/// chosen (´dec:posterior:spectral-floor´).
///
/// Returns infinity where `p·u` reaches one. That is a width no deployment
/// carries and the width at which the bound stops saying anything, so the
/// saturating answer is the honest one: every residual is then at the
/// resolution of the reading it came out of.
#[must_use]
#[allow(clippy::cast_precision_loss)] // Model dimensions are far below f64's exact integer range.
pub fn product_error_growth(p: usize) -> f64 {
    let unit_roundoff = f64::EPSILON / 2.0;
    let width = p as f64;
    let denominator = width.mul_add(-unit_roundoff, 1.0);
    if denominator <= 0.0 {
        f64::INFINITY
    } else {
        width * unit_roundoff / denominator
    }
}

/// The constant in the Cholesky success condition, from which the spectral
/// floor is derived.
///
/// Higham's sufficient condition for a floating-point Cholesky factorisation
/// to run to completion on a symmetric positive definite matrix is
/// `c · p^(3/2) · u · κ₂(A) ≤ 1` at a width `p` and a unit roundoff `u`, with
/// `c = 20` (Higham, *Accuracy and Stability of Numerical Algorithms*, 2nd
/// ed., chapter 10). The constant is carried here rather than folded into the
/// derivation so that the figure the floor rests on is one named quantity
/// (´dec:posterior:spectral-floor´).
///
/// ´const:assayer:cholesky-success-constant´ (´alg:const:scalar´)
/// ´const:assayer:cholesky-success-constant-scalar-20p0´
pub const CHOLESKY_SUCCESS_CONSTANT: f64 = 20.0;

/// The least eigenvalue that keeps a width-`p` matrix factorisable, given its
/// largest.
///
/// # The derivation
///
/// A floating-point Cholesky factorisation of a symmetric positive definite
/// `A` runs to completion when `c · p^(3/2) · u · κ₂(A) ≤ 1`, where `u` is the
/// unit roundoff `ε/2 = 2⁻⁵³` and `c` is [`CHOLESKY_SUCCESS_CONSTANT`]. Since
/// `κ₂(A) = λ_max / λ_min` for a symmetric definite matrix, the condition
/// rearranges to a bound on the least eigenvalue alone:
///
/// ```text
/// λ_min  ≥  c · p^(3/2) · u · λ_max
/// ```
///
/// That right-hand side is what this function returns, and it is the *least*
/// such bound: any smaller floor admits a condition number the factorisation
/// is not guaranteed to survive, and any larger one claims confidence the
/// arithmetic does not require. The floor is therefore derived from evidence
/// — `λ_max` is measured at each rebuild — rather than configured, which is
/// what keeps it out of the class of free parameters
/// (´dec:posterior:spectral-floor´).
///
/// The magnitude is small: at a width near a thousand and a largest
/// eigenvalue near three hundred the floor is of order `10⁻⁸`, five orders
/// below the coordinatewise replenishment floor
/// (´req:gaussian:prior-replenishment-floor´). The two are not
/// interchangeable in either direction, and the replenishment floor's
/// magnitude applied here would settle the least eigenvalue far above the
/// prior precision rather than at the edge of factorisability.
///
/// Returns zero for a non-finite or non-positive `λ_max`, and for an empty
/// width: there is then no spectrum to bound and no floor to hold.
#[must_use]
#[allow(clippy::cast_precision_loss)] // Model dimensions are far below f64's exact integer range.
pub fn spectral_floor(lambda_max: f64, p: usize) -> f64 {
    if p == 0 || !lambda_max.is_finite() || lambda_max <= 0.0 {
        return 0.0;
    }
    let unit_roundoff = f64::EPSILON / 2.0;
    let width = p as f64;
    CHOLESKY_SUCCESS_CONSTANT * width * width.sqrt() * unit_roundoff * lambda_max
}

/// How much of the least-confident direction is held up by the floor rather
/// than by evidence, as a share in `[0, 1]`.
///
/// # What the number says
///
/// The floor holds the least eigenvalue at `λ_floor` under decay alone
/// (´dec:posterior:spectral-floor´), so `λ_floor` is the part of a measured
/// `λ_min` that the model is supplying and `λ_min − λ_floor` is the part the
/// evidence is supplying. The share the floor carries is therefore
/// `λ_floor / λ_min`, which is what this returns.
///
/// Three properties make it readable without its units. It is dimensionless,
/// because both quantities are precisions and the ratio cancels them. It
/// tends to zero as evidence accumulates in the weakest direction, because a
/// `λ_min` far above the floor leaves the floor carrying an ever smaller part
/// of it — a model whose weakest direction is well populated reads near zero,
/// and a model whose weakest direction is entirely the floor's doing reads
/// one. And it is monotone in the degradation: for a fixed floor it rises as
/// `λ_min` falls, and for a fixed `λ_min` it rises as the floor rises.
///
/// The reading is exactly zero only where no floor is in force, which is the
/// honest answer rather than a defect: a model carrying a floor is carrying
/// some share of its weakest direction on the floor's account, and rounding
/// that share to zero below some binding threshold would report a discrete
/// verdict where the quantity is continuous.
///
/// A non-positive `λ_min` reads one, and that test comes first. The reading
/// answers how much of the direction the evidence is *not* holding up, and
/// outside the definite cone the evidence is holding up none of it — so the
/// answer saturates rather than reporting the floor's own contribution, which
/// on a matrix nobody has floored would be zero and would publish a broken
/// model as an undegraded one. The reading is monotone in the degradation, and
/// a matrix outside the cone is the worst degradation there is.
#[must_use]
pub fn floor_share(lambda_floor: f64, lambda_min: f64) -> f64 {
    if !lambda_min.is_finite() || lambda_min <= 0.0 {
        return 1.0;
    }
    if !lambda_floor.is_finite() || lambda_floor <= 0.0 {
        return 0.0;
    }
    (lambda_floor / lambda_min).clamp(0.0, 1.0)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Types
// ═══════════════════════════════════════════════════════════════════════════════

/// The spectrum a rebuild read, and the floor it derived from it.
///
/// The pair `(λ_max, λ_min)` is the true spectrum rather than the diagonal
/// pair the fast check reads (´def:monitoring:condition-number´), so the
/// condition number carried here is the matrix's own and not a lower bound on
/// it. It costs a decomposition, which is why it is taken at the rebuild and
/// nowhere else (´dec:posterior:spectral-floor´).
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SpectralReading {
    /// The largest eigenvalue of the precision matrix as the rebuild found it.
    pub lambda_max: f64,
    /// The least eigenvalue of the precision matrix as the rebuild found it,
    /// before the floor was applied.
    pub lambda_min: f64,
    /// The spectral floor this reading puts in force, from [`spectral_floor`].
    pub lambda_floor: f64,
    /// The amount the floor asks to be added to reach it, `max(0, λ_floor −
    /// λ_min)`, and zero where the evidence already clears the floor.
    ///
    /// Adding `deficit · I` shifts every eigenvalue by it, so the least
    /// becomes exactly the floor and no eigenvalue is raised further than the
    /// least one needed (´dec:posterior:spectral-floor´).
    pub deficit: f64,
    /// The share of the least eigenvalue the floor is holding up, from
    /// [`floor_share`], read after the deficit is applied.
    pub floor_share: f64,
}

impl SpectralReading {
    /// Reads a precision matrix's spectrum and derives the floor it implies.
    ///
    /// Returns `None` where there is no verdict to take: an empty matrix, or
    /// an eigensolver that did not converge. A caller with no reading holds
    /// no floor, which is the conservative disposition — the increment is not
    /// applied on a spectrum nobody measured.
    #[must_use]
    pub fn measure(precision: &SymmetricMatrix, standing_mass: f64) -> Option<Self> {
        let p = precision.dim();
        if p == 0 {
            return None;
        }
        let (lambda_max, lambda_min) = crate::linalg::bridge::eigenvalue_min_max(precision).ok()?;
        let lambda_floor = spectral_floor(lambda_max, p);
        // The deficit is derived from the largest eigenvalue, and applying it
        // raises the largest by the same amount. The floor is orders below the
        // largest at any width a deployment reaches, so the derivation is not
        // sensitive to that and is taken once rather than iterated to a fixed
        // point that would differ in its last bits.
        //
        // A non-positive least eigenvalue asks for nothing. The floor bounds a
        // definite matrix away from the edge of its own arithmetic; it does not
        // carry an indefinite matrix into the definite cone, and the amount
        // that would take is not the least value the arithmetic requires but a
        // repair of whatever went wrong. A matrix outside the cone is a verdict
        // rather than a condition to floor, and what happens to it is decided
        // by the factorisation and by the measurement of its answer
        // (´dec:posterior:measured-adoption´).
        let deficit = if lambda_min > 0.0 && lambda_floor.is_finite() && lambda_floor > lambda_min {
            lambda_floor - lambda_min
        } else {
            0.0
        };
        // The share is read after the floor is applied, against the mass the
        // prior has standing in the matrix by then: what has survived decay
        // from earlier rebuilds, plus what this one is about to add.
        let held = if standing_mass.is_finite() && standing_mass > 0.0 {
            standing_mass + deficit
        } else {
            deficit
        };
        Some(Self {
            lambda_max,
            lambda_min,
            lambda_floor,
            deficit,
            floor_share: floor_share(held, lambda_min + deficit),
        })
    }

    /// The true condition number `λ_max / λ_min`.
    ///
    /// Infinite for a non-positive least eigenvalue, which is the reading a
    /// matrix outside the definite cone earns.
    #[must_use]
    pub const fn kappa(&self) -> f64 {
        if self.lambda_min > 0.0 {
            self.lambda_max / self.lambda_min
        } else {
            f64::INFINITY
        }
    }

    /// The least eigenvalue the matrix carries once the deficit is applied.
    #[must_use]
    pub const fn lambda_min_floored(&self) -> f64 {
        self.lambda_min + self.deficit
    }
}

/// The pair a rebuild offers, and the pair a model installs together.
///
/// The precision matrix travels with the covariance because the rebuild may
/// have added the spectral floor's deficit to it
/// (´dec:posterior:spectral-floor´). Installing one without the other is the
/// arrangement this design exists to end: a covariance that is the inverse of
/// a matrix the model does not hold is a number no consumer can distinguish
/// from an exact one, and the monitor reads the difference as a drift it can
/// never recompute away.
#[derive(Clone, Debug)]
pub struct RebuiltPair {
    /// The precision matrix the model is to hold, floored where the rebuild
    /// found the spectrum below the floor and unchanged where it did not.
    pub precision: SymmetricMatrix,
    /// Its inverse, as the factorisation produced it.
    pub covariance: SymmetricMatrix,
}

/// What the factorisation answered at a rebuild.
///
/// The verdict is the cascade's own, recorded on the baseline so that the
/// health surface can report which device produced the covariance that was
/// offered — separately from whether it was adopted, which is a measurement
/// rather than a verdict (´alg:gaussian:synchronisation-monitor´).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RebuildVerdict {
    /// The factorisation succeeded.
    Clean,
    /// The factorisation was refused and the rebuild offered nothing.
    Refused,
}

/// What one rebuild answered about itself.
///
/// The three readings are three readings of one event, so they share one
/// absence: a model whose cadence has only ever taken measurements has no
/// rebuild to report and carries none of them
/// (´dec:posterior:measured-adoption´). A measurement carries the record its
/// predecessor left forward unchanged, because the spectrum it names and the
/// floor derived from it are still the ones in force — a measurement reads no
/// spectrum and applies no floor, so it has nothing of its own to put here.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RebuildRecord {
    /// The spectrum read at the rebuild, absent where none could be taken.
    pub spectrum: Option<SpectralReading>,
    /// The drift measured after the rebuild, against the same precision
    /// matrix the model would hold.
    pub sync_error_after: f64,
    /// What the factorisation answered.
    pub verdict: RebuildVerdict,
    /// Whether the offered covariance was adopted.
    pub adopted: bool,
    /// The rebuild's own wall time, in nanoseconds.
    pub wall_nanos: u64,
}

/// The record one visit writes, and the baseline the fast check reads.
///
/// Everything the fast check tests is relative to this record, so a visit
/// resets every quantity the check can fire on
/// (´dec:posterior:recomputation-trigger´). Both kinds of visit write one: a
/// measurement writes the readings it took and carries the last rebuild's
/// record forward, and a rebuild writes its own.
///
/// **The separation is the design.** The readings above the rebuild record
/// stand for the pair the model holds and are taken at every visit; the
/// rebuild record stands for the last time a covariance was actually
/// recomputed, and a model whose measurements keep coming back good never
/// writes another one (´dec:posterior:measured-adoption´).
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RecomputeBaseline {
    /// Labels the model had absorbed since the previous visit.
    pub labels_at_record: u32,
    /// The drift the visit measured on the pair the model holds
    /// (´def:monitoring:synchronisation-error´).
    ///
    /// The whole reading, both components together, and the one a reader
    /// comparing against the specification's definition should look at.
    pub sync_error_before: f64,
    /// The part of that reading the prior put there rather than the
    /// arithmetic — the replenishment clamp's contribution since the last
    /// rebuild, seen through the covariance
    /// (´req:gaussian:prior-replenishment-floor´).
    ///
    /// Absent in a baseline written before the components were separated, and
    /// restored as nothing, which reads as a model attributing none of its
    /// drift to the prior.
    #[cfg_attr(feature = "serde", serde(default))]
    pub sync_error_prior_induced: f64,
    /// What is left once that part is taken out, floored at zero — the drift
    /// the arithmetic accumulated, and the quantity the cadence acts on
    /// (´dec:posterior:adaptive-cadence´).
    #[cfg_attr(feature = "serde", serde(default))]
    pub sync_error_residual: f64,
    /// The rounding level of the reading the residual was taken out of, from
    /// [`synchronisation_reading`].
    ///
    /// Published beside the residual rather than folded into it, so that a
    /// reader can see how far above its own resolution a residual stands
    /// instead of taking the verdict on trust.
    #[cfg_attr(feature = "serde", serde(default))]
    pub sync_error_resolution: f64,
    /// Whether the residual is at or below that level — a difference of two
    /// figures that agree to the accuracy the arithmetic could deliver, and
    /// therefore not evidence of drift.
    #[cfg_attr(feature = "serde", serde(default))]
    pub at_resolution: bool,
    /// The cheap diagonal ratio at the visit — the fast check's baseline
    /// for conditioning growth (´def:monitoring:condition-number´).
    pub diagonal_ratio: f64,
    /// Dimensions at the replenishment floor at the visit — the fast
    /// check's baseline for a change in the floored count
    /// (´req:monitoring:replenishment-floor´).
    pub dimensions_at_floor: usize,
    /// The visit's own wall time, in nanoseconds.
    pub wall_nanos: u64,
    /// What the last rebuild answered, absent on a model that has never
    /// rebuilt and carried forward across measurements.
    pub rebuild: Option<RebuildRecord>,
}

impl RecomputeBaseline {
    /// The spectrum the last rebuild read, where one has run and took it.
    #[must_use]
    pub fn spectrum(&self) -> Option<SpectralReading> {
        self.rebuild.and_then(|r| r.spectrum)
    }

    /// The drift the last rebuild measured after itself, where one has run.
    #[must_use]
    pub fn sync_error_after(&self) -> Option<f64> {
        self.rebuild.map(|r| r.sync_error_after)
    }

    /// What the last rebuild's factorisation answered, where one has run.
    #[must_use]
    pub fn verdict(&self) -> Option<RebuildVerdict> {
        self.rebuild.map(|r| r.verdict)
    }

    /// Whether the last rebuild's offered covariance was adopted, where one
    /// has run.
    #[must_use]
    pub fn adopted(&self) -> Option<bool> {
        self.rebuild.map(|r| r.adopted)
    }

    /// The true condition number the last rebuild measured, where it took a
    /// spectrum.
    #[must_use]
    pub fn kappa_true(&self) -> Option<f64> {
        self.spectrum().map(|s| s.kappa())
    }

    /// The spectral floor in force since that rebuild.
    #[must_use]
    pub fn lambda_floor(&self) -> f64 {
        self.spectrum().map_or(0.0, |s| s.lambda_floor)
    }

    /// The share of the least-confident direction the floor is holding up.
    #[must_use]
    pub fn floor_share(&self) -> Option<f64> {
        self.spectrum().map(|s| s.floor_share)
    }

    /// Assembles the record a visit writes from the two halves that produced
    /// it.
    ///
    /// One constructor for both kinds of visit, so that a measurement and a
    /// rebuild cannot disagree about what a baseline is: the readings and the
    /// fast check's own quantities come from the measurement and the inputs,
    /// and `rebuild` is what the visit did about them — its own record where a
    /// rebuild ran, and the record the previous visit left where none did.
    #[must_use]
    pub const fn from_visit(
        measurement: &SynchronisationMeasurement,
        inputs: &VisitInputs<'_>,
        wall_nanos: u64,
        rebuild: Option<RebuildRecord>,
    ) -> Self {
        Self {
            labels_at_record: inputs.labels_since_recompute,
            sync_error_before: measurement.reading.measured,
            sync_error_prior_induced: measurement.prior_induced,
            sync_error_residual: measurement.residual,
            sync_error_resolution: measurement.reading.resolution,
            at_resolution: measurement.at_resolution,
            diagonal_ratio: inputs.diagonal_ratio,
            dimensions_at_floor: inputs.dimensions_at_floor,
            wall_nanos,
            rebuild,
        }
    }

    /// Whether this rebuild repaired the spectrum rather than merely reading
    /// it (´dec:posterior:spectral-floor´).
    ///
    /// True where the floor's deficit was positive, which is the only
    /// condition under which the rebuild adds anything: the matrix the model
    /// installs is then the shifted one, and that shift is a declared prior
    /// with a stated effect on every eigenvalue rather than a note attached to
    /// an inverse. This is the quantity the model's repaired count reports,
    /// and the reason that count stays a measurement now that a factorisation
    /// is attempted once.
    ///
    /// False where no spectrum could be read, because a rebuild that took no
    /// reading applied no floor.
    #[must_use]
    pub fn floor_repaired(&self) -> bool {
        self.spectrum().is_some_and(|s| s.deficit > 0.0)
    }
}

/// What a visit needs beyond the two matrices it is handed.
///
/// The first three are the fast check's own readings for this label, carried
/// into the baseline so that the next label's check compares against the state
/// the visit left rather than recomputing them
/// (´dec:posterior:recomputation-trigger´). The fourth is the threshold the
/// measurement judges its residual against and that a rebuilt covariance has
/// to come in under to be adopted (´def:config:synchronisation-threshold´).
/// The last two are what the measurement and the rebuild respectively need of
/// the prior: the clamp mass is read at every visit, and the identity-shaped
/// mass only where a spectrum is taken.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VisitInputs<'a> {
    /// Labels the model has absorbed since the previous visit.
    pub labels_since_recompute: u32,
    /// The cheap diagonal ratio read this label
    /// (´def:monitoring:condition-number´).
    pub diagonal_ratio: f64,
    /// The floored-dimension count read this label
    /// (´req:monitoring:replenishment-floor´).
    pub dimensions_at_floor: usize,
    /// The drift threshold at this model's width.
    pub sync_error_threshold: f64,
    /// The identity-shaped share the prior already has standing in the
    /// precision matrix, decayed to this label
    /// (´dec:posterior:spectral-floor´).
    pub spectral_floor_mass: f64,
    /// The coordinate-shaped share the replenishment clamp has added since
    /// the last adopted rebuild, decayed to this label
    /// (´req:gaussian:prior-replenishment-floor´).
    ///
    /// The mass since the rebuild and not the model's lifetime figure: a
    /// rebuild takes the covariance from the precision matrix the clamp has
    /// already raised, so everything added before it is absorbed into the
    /// pair and is no longer a disagreement to attribute.
    pub clamp_mass: &'a [f64],
}

impl<'a> VisitInputs<'a> {
    /// Builds the inputs from the trigger that fired and the model's width
    /// threshold, so that the two readings the check has already taken are not
    /// taken again.
    #[must_use]
    pub const fn from_trigger(
        trigger: &RecomputeTrigger,
        labels_since_recompute: u32,
        sync_error_threshold: f64,
        spectral_floor_mass: f64,
        clamp_mass: &'a [f64],
    ) -> Self {
        Self {
            labels_since_recompute,
            diagonal_ratio: trigger.diagonal_ratio(),
            dimensions_at_floor: trigger.dimensions_at_floor(),
            sync_error_threshold,
            spectral_floor_mass,
            clamp_mass,
        }
    }
}

/// Trigger reason for Cholesky recomputation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RecomputeTrigger {
    /// No recomputation needed.
    NotNeeded {
        /// The cheap diagonal ratio read this label, a lower bound on the
        /// true condition number (´def:monitoring:condition-number´).
        diagonal_ratio: f64,
        /// Dimensions at the replenishment floor
        /// (´req:monitoring:replenishment-floor´).
        dimensions_at_floor: usize,
    },
    /// Counter-based trigger: `labels_since_recompute >= n_recompute_effective`.
    Counter {
        /// Number of labels processed since last recompute.
        labels: u32,
        /// The cheap diagonal ratio read this label.
        diagonal_ratio: f64,
        /// Dimensions at the replenishment floor
        /// (´req:monitoring:replenishment-floor´).
        dimensions_at_floor: usize,
    },
    /// The diagonal ratio has grown past its baseline by the configured factor.
    ConditioningGrowth {
        /// The cheap diagonal ratio read this label.
        diagonal_ratio: f64,
        /// The ratio the last rebuild recorded, which this one outgrew.
        baseline_ratio: f64,
        /// Dimensions at the replenishment floor
        /// (´req:monitoring:replenishment-floor´).
        dimensions_at_floor: usize,
    },
    /// The count of dimensions resting at the replenishment floor has moved
    /// since the last rebuild, so the evidence structure the baseline's
    /// spectrum was read from is no longer the one in hand.
    FlooredCountChanged {
        /// The cheap diagonal ratio read this label.
        diagonal_ratio: f64,
        /// Dimensions at the replenishment floor now.
        dimensions_at_floor: usize,
        /// Dimensions at the replenishment floor at the last rebuild.
        baseline_dimensions_at_floor: usize,
    },
    /// A precision diagonal entry is not finite, or is not positive. The
    /// matrix is outside the definite cone by the cheapest reading there is,
    /// and the rebuild is immediate.
    Degenerate {
        /// The cheap diagonal ratio read this label, infinite where the least
        /// diagonal entry is not positive.
        diagonal_ratio: f64,
        /// Dimensions at the replenishment floor
        /// (´req:monitoring:replenishment-floor´).
        dimensions_at_floor: usize,
    },
}

impl RecomputeTrigger {
    /// Returns the cheap diagonal ratio from any variant.
    ///
    /// The name says what the quantity is. It is the ratio of the largest
    /// precision diagonal entry to the smallest, which bounds the true
    /// condition number from below and does not estimate it
    /// (´def:monitoring:condition-number´).
    #[must_use]
    pub const fn diagonal_ratio(&self) -> f64 {
        match self {
            Self::NotNeeded { diagonal_ratio, .. }
            | Self::Counter { diagonal_ratio, .. }
            | Self::ConditioningGrowth { diagonal_ratio, .. }
            | Self::FlooredCountChanged { diagonal_ratio, .. }
            | Self::Degenerate { diagonal_ratio, .. } => *diagonal_ratio,
        }
    }

    /// Returns true where the trigger calls for a visit.
    ///
    /// A visit is a measurement first and a rebuild only where the
    /// measurement was bad, so this no longer says a factorisation will be
    /// paid for (´dec:posterior:recomputation-trigger´).
    #[must_use]
    pub const fn is_needed(&self) -> bool {
        !matches!(self, Self::NotNeeded { .. })
    }

    /// Returns true where the visit goes to the rebuild whatever the
    /// measurement says.
    ///
    /// # The arms differ in what they detect, so they differ in what can
    /// answer them
    ///
    /// The counter arm says that labels have been absorbed and the pair
    /// should be looked at. That is a question about drift between the two
    /// matrices, and the measurement answers it exactly: it reads the drift,
    /// separates it, and says whether any of it is worth a factorisation.
    ///
    /// The other three arms say something the measurement cannot answer,
    /// because they are readings of the precision matrix rather than of the
    /// pair. The conditioning-growth arm and the floored-count arm fire when
    /// the matrix has moved away from the state the last rebuild's spectrum
    /// was read from (´alg:gaussian:condition-adaptive-recompute´),
    /// (´req:monitoring:replenishment-floor´), and what that staleness
    /// threatens is the spectral floor — which is derived from that spectrum
    /// and applied at the rebuild, and nowhere else
    /// (´dec:posterior:spectral-floor´). A pair can be synchronised to the
    /// last bit while the matrix it is a pair of marches toward the edge of
    /// the definite cone, so a measurement asked to rule on these arms
    /// returns "good" and leaves the floor unapplied for as long as the drift
    /// stays small. Measured, that is not a hypothetical: with all three arms
    /// ruled on by the measurement, the width sweep's outcome-axis model —
    /// whose reading is almost entirely its replenishment clamp's and whose
    /// residual is therefore always good — stopped the label path outright at
    /// 3124 labels, its factorisation refused at pivot 1013 of a width of
    /// 1150, having taken no rebuild and therefore no floor since its drift
    /// last mattered (´rep:assayer:declined-rebuild-cadence´).
    ///
    /// The degenerate arm is the limiting case of the same argument and needs
    /// no separate one. A precision matrix carrying a diagonal entry that is
    /// not finite or not positive is outside the definite cone by the
    /// cheapest reading there is, and the drift reading taken against it is
    /// not a number a verdict can be read off. Letting the measurement decide
    /// would put the model on the arm whose residual comparison a non-finite
    /// value fails silently, which is the one case where "the measurement was
    /// good" would mean "the measurement was not a number".
    ///
    /// What the restructure keeps is the whole of what the reading asked for.
    /// The models it found pinned at the cadence's floor were pinned there by
    /// the counter arm — nothing rests at their replenishment floor and their
    /// diagonal ratio grows slowly enough that the growth arm fires a handful
    /// of times over thousands of labels — so the arm that stops rebuilding
    /// for them is exactly the arm that was rebuilding for nothing.
    ///
    /// The measurement is still taken on every one of these paths — one
    /// product on a visit that is about to pay for a decomposition and a
    /// factorisation — because what it reads is the evidence the record and
    /// any alarm publish.
    #[must_use]
    pub const fn skips_the_measurements_verdict(&self) -> bool {
        matches!(
            self,
            Self::Degenerate { .. } | Self::ConditioningGrowth { .. } | Self::FlooredCountChanged { .. }
        )
    }

    /// Returns the dimensions-at-floor count from any variant.
    #[must_use]
    pub const fn dimensions_at_floor(&self) -> usize {
        match self {
            Self::NotNeeded { dimensions_at_floor, .. }
            | Self::Counter { dimensions_at_floor, .. }
            | Self::ConditioningGrowth { dimensions_at_floor, .. }
            | Self::FlooredCountChanged { dimensions_at_floor, .. }
            | Self::Degenerate { dimensions_at_floor, .. } => *dimensions_at_floor,
        }
    }
}

/// What one visit to a model concluded.
///
/// There are three answers where there used to be two, and the third is the
/// one the restructure exists to make sayable: a visit that measured and found
/// nothing to do (´dec:posterior:recomputation-trigger´). The cadence reads
/// this and nothing else (´dec:posterior:adaptive-cadence´).
#[derive(Clone, Debug)]
pub enum RecomputeOutcome {
    /// The measurement was good and nothing was rebuilt.
    ///
    /// Either the residual came in under the model's width-scaled threshold,
    /// or it came in at or below the resolution of the reading it was taken
    /// out of. The model holds the pair it already had, the counter is reset,
    /// and the interval recovers as a clean rebuild's did.
    Measured {
        /// What was left of the measured drift once the part the prior put
        /// there was taken out of it
        /// (´def:monitoring:synchronisation-error´).
        ///
        /// The residual and not the measured reading, because this is the
        /// quantity the cadence acts on. The measured reading, the prior's
        /// own component and the reading's resolution are all carried on
        /// [`RecomputeBaseline`], which is where the health surface reads
        /// them from.
        sync_error_residual: f64,
        /// Whether the residual was good because it was at the resolution of
        /// its own reading rather than under the threshold.
        at_resolution: bool,
    },
    /// The measurement was bad, the rebuild ran, and the measurement of its
    /// result adopted it (´dec:posterior:measured-adoption´).
    ///
    /// The drift was real and the rebuild removed it, so the cadence should
    /// visit sooner: this is the one outcome that shortens on evidence that
    /// something needed doing.
    Rebuilt {
        /// The residual the measurement found.
        sync_error_residual: f64,
        /// Whether that residual is what called for the rebuild, as against
        /// the replenishment clamp's own contribution having done so. Only
        /// the first shortens the interval.
        drift_was_real: bool,
    },
    /// The alarm: a rebuild that was needed and left the model no better, or
    /// one the factorisation refused outright.
    ///
    /// The pair the model holds is kept, because a covariance that is not an
    /// improvement is not one whatever called for it
    /// (´dec:posterior:measured-adoption´). There is no longer any decline
    /// that is not this, because there is no longer any rebuild that was not
    /// needed.
    Alarm {
        /// The residual that called for the rebuild.
        sync_error_residual: f64,
        /// The drift the rebuild measured after itself, and the drift the
        /// model would have taken on had the result been adopted. Equal to
        /// the drift before where the factorisation offered nothing.
        sync_error_after: f64,
        /// The pivot at which the factorisation was refused, absent where it
        /// succeeded and the measurement declined its answer
        /// (´dec:posterior:repair-cascade´).
        refused_pivot: Option<usize>,
    },
}

impl RecomputeOutcome {
    /// Returns true where the visit rebuilt nothing.
    #[must_use]
    pub const fn is_measurement(&self) -> bool {
        matches!(self, Self::Measured { .. })
    }

    /// Returns true where the visit found no drift the arithmetic had
    /// accumulated — a measurement that called for nothing, or a rebuild the
    /// replenishment clamp asked for rather than the residual.
    ///
    /// This is what the recovery run counts, because it is what "the
    /// condition has passed" means: a rebuild that absorbed a clamp
    /// contribution has not told the cadence that the model is drifting
    /// (´dec:posterior:adaptive-cadence´).
    #[must_use]
    pub const fn found_no_drift(&self) -> bool {
        matches!(
            self,
            Self::Measured { .. }
                | Self::Rebuilt {
                    drift_was_real: false,
                    ..
                }
        )
    }

    /// Returns true where the visit paid for a factorisation.
    #[must_use]
    pub const fn is_rebuild(&self) -> bool {
        !matches!(self, Self::Measured { .. })
    }

    /// Returns true where the visit raised the alarm.
    #[must_use]
    pub const fn is_alarm(&self) -> bool {
        matches!(self, Self::Alarm { .. })
    }

    /// Returns true where the factorisation was refused
    /// (´dec:posterior:repair-cascade´).
    #[must_use]
    pub const fn is_cascade_terminus(&self) -> bool {
        matches!(
            self,
            Self::Alarm {
                refused_pivot: Some(_),
                ..
            }
        )
    }

    /// The pivot at which the factorisation was refused, where one was.
    #[must_use]
    pub const fn refused_pivot(&self) -> Option<usize> {
        match self {
            Self::Alarm { refused_pivot, .. } => *refused_pivot,
            Self::Measured { .. } | Self::Rebuilt { .. } => None,
        }
    }

    /// Returns the drift the visit measured with the prior's own component
    /// taken out of it — the quantity the cadence acts on
    /// (´def:monitoring:synchronisation-error´).
    #[must_use]
    pub const fn sync_error_residual(&self) -> f64 {
        match self {
            Self::Measured { sync_error_residual, .. }
            | Self::Rebuilt { sync_error_residual, .. }
            | Self::Alarm { sync_error_residual, .. } => *sync_error_residual,
        }
    }
}

/// Per-model precision health tracking state (unit-test helper).
///
/// The cadence and diagnostic fields the trigger reads and writes
/// (´dec:posterior:recomputation-trigger´), including the count of clean
/// high-error recomputations that actually shortened the interval.
#[derive(Clone, Debug)]
pub struct ModelPrecisionHealth {
    /// Effective recompute interval (halved on failure, restored on recovery).
    pub n_recompute_effective: u32,
    /// Consecutive clean (unregularised) recomputes.
    pub consecutive_clean_recomputes: u32,
    /// The cheap diagonal ratio last read, a lower bound on the true
    /// condition number (´def:monitoring:condition-number´).
    pub last_diagonal_ratio: f64,
    /// Last synchronisation error (´def:monitoring:synchronisation-error´),
    /// the whole reading with both its components in it.
    pub last_sync_error: f64,
    /// The part of that reading the prior put there
    /// (´req:gaussian:prior-replenishment-floor´).
    pub last_prior_induced_sync_error: f64,
    /// What was left of it once the prior's part was taken out — the quantity
    /// the cadence acts on (´dec:posterior:adaptive-cadence´).
    pub last_sync_error_residual: f64,
    /// Rebuilds that were needed, were adopted, and shortened this model's
    /// interval (´dec:posterior:adaptive-cadence´).
    pub sync_error_shortenings: u64,
}

impl Default for ModelPrecisionHealth {
    fn default() -> Self {
        Self {
            n_recompute_effective: 1000,
            consecutive_clean_recomputes: 0,
            last_diagonal_ratio: 1.0,
            last_sync_error: 0.0,
            last_prior_induced_sync_error: 0.0,
            last_sync_error_residual: 0.0,
            sync_error_shortenings: 0,
        }
    }
}

impl ModelPrecisionHealth {
    /// Creates a new health state with a custom `n_recompute_effective`.
    #[must_use]
    pub const fn with_interval(n_recompute: u32) -> Self {
        Self {
            n_recompute_effective: n_recompute,
            consecutive_clean_recomputes: 0,
            last_diagonal_ratio: 1.0,
            last_sync_error: 0.0,
            last_prior_induced_sync_error: 0.0,
            last_sync_error_residual: 0.0,
            sync_error_shortenings: 0,
        }
    }

    /// Records the outcome of a visit and updates health state.
    ///
    /// Updates `consecutive_clean_recomputes` and the residual reading. An
    /// outcome carries the residual alone; the measured reading, the prior's
    /// component and the reading's resolution are on the baseline, and a
    /// caller holding one should use [`Self::record_baseline_readings`]
    /// beside this.
    ///
    /// A good measurement is what the run of clean visits is now counted in.
    /// A rebuild interrupts that run whether or not its result was adopted,
    /// because a model that needed rebuilding is not one whose condition has
    /// passed (´dec:posterior:adaptive-cadence´).
    #[allow(clippy::missing_const_for_fn)] // mut self prevents const
    pub fn record_outcome(&mut self, outcome: &RecomputeOutcome) {
        self.last_sync_error_residual = outcome.sync_error_residual();

        if outcome.is_measurement() {
            self.consecutive_clean_recomputes += 1;
        } else {
            self.consecutive_clean_recomputes = 0;
        }
    }

    /// Records the two readings only a rebuild's baseline carries: the
    /// measured drift and the part of it the prior put there
    /// (´def:monitoring:synchronisation-error´).
    ///
    /// The residual arrives with the outcome, because that is what the cadence
    /// acts on; these two arrive with the baseline, because that is what the
    /// health surface reports.
    #[allow(clippy::missing_const_for_fn)] // mut self prevents const
    pub fn record_baseline_readings(&mut self, baseline: &RecomputeBaseline) {
        self.last_sync_error = baseline.sync_error_before;
        self.last_prior_induced_sync_error = baseline.sync_error_prior_induced;
        self.last_sync_error_residual = baseline.sync_error_residual;
    }

    /// Updates the interval based on the visit's outcome. Returns true if the
    /// interval changed.
    ///
    /// Delegates to [`compute_new_interval`] — the single source of truth
    /// for the shortening and recovery policy.
    pub fn update_interval(&mut self, n_recompute_configured: u32, outcome: &RecomputeOutcome) -> bool {
        if !outcome.is_measurement() {
            self.consecutive_clean_recomputes = 0;
        }
        let new = compute_new_interval(
            self.n_recompute_effective,
            n_recompute_configured,
            self.consecutive_clean_recomputes,
            outcome,
        );
        if new == self.n_recompute_effective {
            false
        } else {
            self.n_recompute_effective = new;
            self.sync_error_shortenings += u64::from(matches!(
                outcome,
                RecomputeOutcome::Rebuilt {
                    drift_was_real: true,
                    ..
                }
            ));
            true
        }
    }
}

/// Computes the new effective recomputation interval after a visit.
///
/// Single source of truth for the shortening and recovery policy
/// (´dec:posterior:adaptive-cadence´).
/// Used by both [`ModelPrecisionHealth::update_interval`] and
/// [`BayesianLinearModel::apply_recompute_outcome`](super::bayesian::BayesianLinearModel::apply_recompute_outcome).
///
/// - **Measured:** the visit found nothing to do. The interval recovers on a
///   run of [`N_RECOVERY_CLEAN`] such visits and is otherwise unchanged. This
///   is the arm the restructure adds, and it is why the two models the
///   reading found pinned at the floor can leave it: they were being
///   shortened by rebuilds that were never needed
///   (´rep:assayer:declined-rebuild-cadence´).
/// - **Rebuilt on a real residual:** the measurement found drift the
///   arithmetic had accumulated, and the rebuild removed it. Halve with floor
///   at [`N_RECOMPUTE_FLOOR`] — the model is accumulating drift faster than
///   the current interval anticipates and should be visited sooner.
/// - **Rebuilt on the clamp's contribution alone:** the residual was under
///   the threshold and the replenishment clamp's own contribution to the
///   reading was not, so the rebuild ran to absorb it
///   (´req:gaussian:prior-replenishment-floor´). The interval is unchanged
///   and the visit counts toward recovery. Shortening here would rebuild the
///   trap this record retired: the clamp's contribution scales with the
///   interval, so an interval shortening against it converges on the floor
///   and stays there while the quantity it is reacting to is one no
///   factorisation reduces.
/// - **Alarm:** a rebuild that was needed left the model no better, or was
///   refused. Straight to [`N_RECOMPUTE_FLOOR`] rather than halved: halving
///   is a proportional response to drift that is being removed, and nothing
///   is being removed here. The model is looked at as often as the cadence
///   permits until a measurement comes back good.
///
/// The threshold no longer appears, and its absence is the restructure. A
/// visit's verdict on the residual is taken at the measurement, where the
/// resolution of the reading is also in hand; by the time an outcome reaches
/// the cadence the comparison has been made and the outcome names the answer
/// (´dec:posterior:measured-adoption´).
#[must_use]
pub const fn compute_new_interval(current: u32, configured: u32, consecutive_clean: u32, outcome: &RecomputeOutcome) -> u32 {
    match outcome {
        RecomputeOutcome::Rebuilt {
            drift_was_real: true, ..
        } => {
            let halved = current / 2;
            if halved > N_RECOMPUTE_FLOOR {
                halved
            } else {
                N_RECOMPUTE_FLOOR
            }
        }
        RecomputeOutcome::Alarm { .. } => N_RECOMPUTE_FLOOR,
        RecomputeOutcome::Measured { .. }
        | RecomputeOutcome::Rebuilt {
            drift_was_real: false, ..
        } => {
            if consecutive_clean >= N_RECOVERY_CLEAN && current < configured {
                configured
            } else {
                current
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Core Functions
// ═══════════════════════════════════════════════════════════════════════════════

/// The fast check: once per model per label, `O(p)`, and it decides only
/// whether a rebuild is due.
///
/// # What it tests, and why every test is relative
///
/// Every arm compares against the baseline the last rebuild wrote
/// (´dec:posterior:recomputation-trigger´), so every quantity it can fire on
/// is one a rebuild resets. That is the property the retired conditioning arm
/// did not have. It tested the ratio of the largest precision diagonal entry
/// to the smallest against a fixed threshold, and the smallest is the
/// replenishment floor by construction, because the update clamps it there on
/// every label (´req:gaussian:prior-replenishment-floor´). Once any dimension
/// rests at the floor the ratio is pinned above the threshold and stays there,
/// so the arm fired on every label for ever — and the rebuild it called for
/// recomputes the covariance from the precision matrix without touching the
/// precision matrix, so the quantity the arm tested is one the rebuild cannot
/// move. Replacing the diagonal ratio with the true eigenvalue ratio would
/// have trapped in exactly the same way, because the trap is the fixed
/// threshold on a matrix property, not the crudeness of the estimate.
///
/// The four arms:
///
/// - **Degenerate.** A precision diagonal entry that is not finite or not
///   positive rebuilds immediately. This is the one absolute test left, and it
///   is absolute because it is not a matter of degree: the matrix is outside
///   the definite cone by the cheapest reading available.
/// - **Counter.** The labels absorbed have reached the interval the baseline's
///   outcome set, which is the arm that bounds drift the conditioning cannot
///   reveal (´dec:posterior:adaptive-cadence´).
/// - **Conditioning growth.** The diagonal ratio has grown past the ratio the
///   last rebuild recorded by the configured factor. A rebuild resets that
///   record, so the arm fires at most once per multiplication of the ratio by
///   the factor rather than once per label.
/// - **Floored count changed.** The number of dimensions resting at the
///   replenishment floor is no longer what it was at the last rebuild
///   (´req:monitoring:replenishment-floor´). The count was computed on this
///   path already and discarded; it is the cheapest available signal that the
///   evidence structure the baseline's spectrum was read from has moved, which
///   is what makes the spectral floor derived from it stale.
///
/// A model with no baseline — a fresh one, or one restored from a checkpoint
/// written before the baseline existed — has nothing to be relative to, so
/// only the counter and the degenerate arm can fire, and the first rebuild
/// establishes the record the other two need.
///
/// # Arguments
///
/// * `precision` — the precision matrix `B` to check
/// * `labels_since_recompute` — labels processed since the last rebuild
/// * `n_recompute_effective` — the current effective interval
/// * `baseline` — the last rebuild's record, absent before the first one
/// * `kappa_growth_factor` — the multiple of the baseline ratio that calls for
///   a rebuild
/// * `lambda_floor` — the replenishment floor, for the floored-dimension count
///
/// # Returns
///
/// The trigger reason, carrying the diagonal ratio and the floored-dimension
/// count whether or not it fired.
#[must_use]
pub fn should_recompute(
    precision: &SymmetricMatrix,
    labels_since_recompute: u32,
    n_recompute_effective: u32,
    baseline: Option<&RecomputeBaseline>,
    kappa_growth_factor: f64,
    lambda_floor: f64,
) -> RecomputeTrigger {
    // The cheap conditioning proxy: an O(p) diagonal scan. It bounds the true
    // condition number from below and does not estimate it
    // (´def:monitoring:condition-number´).
    let (max_diag, min_diag) = precision.diagonal_min_max();
    let diagonal_ratio = if min_diag > 0.0 { max_diag / min_diag } else { f64::INFINITY };

    // Floor-dimension count: O(p) diagonal scan
    // (´req:monitoring:replenishment-floor´).
    let dimensions_at_floor = precision.count_diagonal_at_or_below(lambda_floor);

    if !max_diag.is_finite() || !min_diag.is_finite() || min_diag <= 0.0 {
        return RecomputeTrigger::Degenerate {
            diagonal_ratio,
            dimensions_at_floor,
        };
    }

    if labels_since_recompute >= n_recompute_effective {
        return RecomputeTrigger::Counter {
            labels: labels_since_recompute,
            diagonal_ratio,
            dimensions_at_floor,
        };
    }

    if let Some(record) = baseline {
        let baseline_ratio = record.diagonal_ratio;
        if baseline_ratio.is_finite() && baseline_ratio > 0.0 && diagonal_ratio > baseline_ratio * kappa_growth_factor {
            return RecomputeTrigger::ConditioningGrowth {
                diagonal_ratio,
                baseline_ratio,
                dimensions_at_floor,
            };
        }

        if dimensions_at_floor != record.dimensions_at_floor {
            return RecomputeTrigger::FlooredCountChanged {
                diagonal_ratio,
                dimensions_at_floor,
                baseline_dimensions_at_floor: record.dimensions_at_floor,
            };
        }
    }

    RecomputeTrigger::NotNeeded {
        diagonal_ratio,
        dimensions_at_floor,
    }
}

/// Computes the synchronisation error `‖BΣ − I‖_F` fixed by
/// (´def:monitoring:synchronisation-error´).
///
/// The whole matrix, not its diagonal: a maximum over the diagonal is
/// bounded above by this norm and can therefore stay small while
/// off-diagonal error grows without limit, which is the substitution
/// the definition keeps to one site in order to forbid. The full
/// product is O(p³); the diagonal alone would be O(p²).
///
/// # Expected values
///
/// | Condition | Expected ε |
/// |-----------|------------|
/// | After recompute | < 10⁻¹⁴ |
/// | After 100 labels | < 10⁻⁹ |
/// | After 1,000 labels (normal) | < 10⁻⁶ |
/// | After 1,000 labels (20 floored dims) | < 10⁻⁴ |
/// | > 10⁻³ at any point | Abnormal |
#[must_use]
pub fn synchronisation_error(precision: &SymmetricMatrix, covariance: &SymmetricMatrix) -> f64 {
    let p = precision.dim();
    debug_assert_eq!(p, covariance.dim(), "dimension mismatch");

    if p == 0 {
        return 0.0;
    }

    let product = precision.multiply(covariance);

    let mut sum_sq = 0.0_f64;
    for j in 0..p {
        for i in 0..p {
            let expected = if i == j { 1.0 } else { 0.0 };
            let diff = product[(i, j)] - expected;
            sum_sq = diff.mul_add(diff, sum_sq);
        }
    }
    sum_sq.sqrt()
}

/// A synchronisation reading and the rounding level of that reading.
///
/// The two travel together because neither answers the cadence's question
/// alone. A reading says how far the tracked pair has come apart; the
/// resolution says how much of that figure the arithmetic that produced it
/// could have manufactured, and a difference smaller than the resolution of
/// the numbers it was taken between is not a measurement of anything
/// (´def:monitoring:synchronisation-error´).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SynchronisationReading {
    /// The drift `‖BΣ − I‖_F` the definition fixes.
    pub measured: f64,
    /// The absolute rounding level of that reading, `γ_p · ‖ |B| |Σ| ‖_F`.
    ///
    /// A residual below this is the rounding of a subtraction rather than
    /// drift, however far above the width-scaled threshold it happens to sit.
    pub resolution: f64,
}

/// The drift the held pair carries, and the resolution of that reading, in
/// one pass (´def:monitoring:synchronisation-error´).
///
/// # Why the reading carries its own resolution
///
/// The reading is a difference, and the cadence acts on a further difference
/// taken out of it: the residual is the measured drift less the part the
/// replenishment clamp put there. Where the clamp's part dominates, the two
/// operands agree to every printed digit and what is left is the rounding of
/// the subtraction rather than any drift the arithmetic accumulated. A model
/// publishing a reading of order `10⁵`, a prior-induced component of the same
/// order, and a residual near one is not reporting drift near one; it is
/// reporting that its two figures agree to about a part in a hundred
/// thousand. Testing that remainder against an absolute threshold shortens
/// the cadence on the rounding of a subtraction.
///
/// # The derivation, and why there is no free constant in it
///
/// The reading is computed by forming `P = fl(BΣ)` and taking the Frobenius
/// norm of `P − I`. The standard entrywise bound for a floating-point matrix
/// product at width `p` is `|P − BΣ| ≤ γ_p · |B||Σ|`, so the norm of the error
/// in the reading is at most `γ_p · ‖ |B| |Σ| ‖_F` — which is what this
/// returns as the resolution, with `γ_p` from [`product_error_growth`]. The
/// subtraction of the identity contributes nothing further: an entry near one
/// less one is exact in binary floating point, and an off-diagonal entry is
/// unchanged. The norm's own accumulation adds a relative `γ_{p²}`, orders
/// below the leading term, and is not carried.
///
/// **The operand-magnitude product is the whole of the content, and it is why
/// the resolution is not simply the reading scaled.** Where the pair is well
/// scaled, `‖ |B| |Σ| ‖_F` is of the order of `‖BΣ‖_F` and the resolution
/// reduces to the reading times a few multiples of the unit roundoff and the
/// width — a level no meaningful reading approaches. Where the precision
/// matrix spans many orders, the products `|B_ik||Σ_kj|` summed along a row
/// are larger than the entry they cancel down to by about the conditioning,
/// and the resolution rises with it. That is the case the refinement exists
/// for, and reading it off the operands rather than off the answer is what
/// keeps the test from firing on a healthy model whose reading happens to be
/// small.
///
/// Cost: a second `O(p³)` product, on a path that is taken at the cadence's
/// interval rather than per label, and which this reading exists to keep from
/// escalating to a rebuild — an eigendecomposition, a factorisation, an
/// inverse and a further product.
#[must_use]
pub fn synchronisation_reading(precision: &SymmetricMatrix, covariance: &SymmetricMatrix) -> SynchronisationReading {
    let p = precision.dim();
    debug_assert_eq!(p, covariance.dim(), "dimension mismatch");

    let measured = synchronisation_error(precision, covariance);

    if p == 0 {
        return SynchronisationReading {
            measured,
            resolution: 0.0,
        };
    }

    // The magnitude product `|B||Σ|`. Both operands are symmetric and taking
    // magnitudes preserves that, so each is absorbed as the symmetric matrix
    // it is rather than mirrored into one.
    let precision_inner = precision.as_inner();
    let covariance_inner = covariance.as_inner();
    let magnitudes = SymmetricMatrix::from_computation(Mat::from_fn(p, p, |i, j| precision_inner[(i, j)].abs())).multiply(
        &SymmetricMatrix::from_computation(Mat::from_fn(p, p, |i, j| covariance_inner[(i, j)].abs())),
    );

    let mut sum_sq = 0.0_f64;
    for j in 0..p {
        for i in 0..p {
            let entry = magnitudes[(i, j)];
            sum_sq = entry.mul_add(entry, sum_sq);
        }
    }

    SynchronisationReading {
        measured,
        resolution: product_error_growth(p) * sum_sq.sqrt(),
    }
}

/// The part of the synchronisation reading (´def:monitoring:synchronisation-error´)
/// the prior put there rather than the arithmetic.
///
/// The replenishment clamp raises a floored diagonal entry on every label at
/// which a coordinate has decayed below the floor
/// (´req:gaussian:prior-replenishment-floor´), and the Sherman–Morrison
/// covariance does not follow that addition. So the tracked pair disagrees by
/// the clamp's contribution seen through the covariance, `‖diag(c)Σ‖_F`, and
/// the identity is exact rather than asymptotic: under decay the precision
/// matrix is its decayed self plus exactly that contribution, so the product
/// `BΣ` is `I + diag(c)Σ` term for term.
///
/// **The contribution is the mass accumulated since the last rebuild, not the
/// mass the model has accumulated in its life.** A rebuild takes the
/// covariance from the precision matrix the clamp has already raised, so
/// everything the clamp added before it is absorbed into the pair and stops
/// being a disagreement. The mass tracked on the model is the lifetime figure,
/// because the share of its precision a model borrowed rather than earned is a
/// lifetime quantity and is read as one elsewhere; the two coincide only on a
/// model that has never rebuilt, which is the condition of the closed-form
/// test that pins this arithmetic.
///
/// The spectral floor has no term here. It is an addition to the precision
/// matrix too, but it is applied where the covariance is being rebuilt from
/// that same matrix (´dec:posterior:spectral-floor´), so the pair it leaves is
/// consistent by construction and it contributes nothing for the monitor to
/// read. An identity-shaped term would subtract `f·‖Σ‖_F` of drift that the
/// prior did not put there, and the residual would understate real rounding by
/// exactly that much.
///
/// A mass shorter than the covariance reads zero in the coordinates it does
/// not cover, which is what a model carrying no mass at all reports.
#[must_use]
pub fn prior_induced_synchronisation_error(covariance: &SymmetricMatrix, clamp_mass: &[f64]) -> f64 {
    let p = covariance.dim();
    if p == 0 {
        return 0.0;
    }

    let inner = covariance.as_inner();

    let mut sum_sq = 0.0_f64;
    for i in 0..p {
        let row_mass = clamp_mass.get(i).copied().unwrap_or(0.0);
        if row_mass == 0.0 {
            continue;
        }
        for j in 0..p {
            let entry = row_mass * inner[(i, j)];
            sum_sq = entry.mul_add(entry, sum_sq);
        }
    }
    sum_sq.sqrt()
}

/// What is left of a synchronisation reading once the part the prior put there
/// is taken out of it — the drift the arithmetic accumulated, and the only
/// part a rebuild can remove.
///
/// Floored at zero, because the two quantities are measured against the same
/// pair but by different routes and a residual below zero is a rounding
/// artefact of the subtraction rather than a negative drift.
#[must_use]
pub fn residual_synchronisation_error(measured: f64, prior_induced: f64) -> f64 {
    if !measured.is_finite() {
        return measured;
    }
    (measured - prior_induced).max(0.0)
}

/// What one measurement found, and the verdict it reached on its own
/// evidence.
///
/// The measurement is the cheap half of a visit: one matrix product for the
/// drift, one for the resolution of that drift, and an `O(p²)` pass for the
/// part the prior put there. No spectrum is read and no factorisation is
/// attempted, so a model whose measurements keep coming back good never pays
/// a cubic factorisation at all (´dec:posterior:recomputation-trigger´).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SynchronisationMeasurement {
    /// The drift on the pair the model holds, and the rounding level of that
    /// reading (´def:monitoring:synchronisation-error´).
    pub reading: SynchronisationReading,
    /// The part of the reading the replenishment clamp put there since the
    /// last rebuild, seen through the covariance
    /// (´req:gaussian:prior-replenishment-floor´).
    pub prior_induced: f64,
    /// What is left once that part is taken out — the drift the arithmetic
    /// accumulated (´dec:posterior:adaptive-cadence´).
    pub residual: f64,
    /// Whether the residual is at or below the resolution of the reading it
    /// was taken out of.
    ///
    /// A reading rather than a verdict: it says the residual is the rounding
    /// of the subtraction that produced it, and it does not on its own make a
    /// residual over the threshold good.
    pub at_resolution: bool,
    /// Whether the measurement calls for nothing — both components at or
    /// under the model's threshold.
    pub good: bool,
    /// Whether the *residual* is what called for a rebuild.
    ///
    /// The two bad arms mean different things to the cadence. A residual over
    /// the threshold is drift the arithmetic accumulated, and a model
    /// accumulating it faster than its interval anticipates should be visited
    /// sooner. A prior-induced component over the threshold is the
    /// replenishment clamp's doing: the rebuild absorbs it, but nothing about
    /// it says the model needs looking at more often, and shortening on it
    /// would re-create the trap the cadence record retired — an interval
    /// driven to its floor by a quantity shortening does not reduce
    /// (´dec:posterior:adaptive-cadence´).
    pub drift_is_real: bool,
}

/// The measurement: what the held pair's drift is, and whether it is worth
/// rebuilding for.
///
/// # The verdict reads both components against the same threshold
///
/// **The residual.** It is good when it is at or under the model's
/// width-scaled threshold, which is the test the cadence has always applied.
/// That is the drift the arithmetic accumulated and the part a rebuild
/// removes by recomputing.
///
/// **The prior-induced component.** It is good when it too is at or under
/// that threshold, and this arm is what the falsifier put here. The component
/// is the replenishment clamp's contribution *since the last rebuild* seen
/// through the covariance (´req:gaussian:prior-replenishment-floor´), and the
/// only operation that removes it is an adopted rebuild, which takes the
/// covariance from the precision matrix the clamp has already raised. A
/// measurement cannot remove it and no amount of waiting reduces it: it
/// compounds, because the covariance it is read through grows as the
/// precision decays. So a measurement that looked only at the residual would
/// report a healthy model while the pair it holds ceased to be an inverse
/// pair at all — measured, the schedule's outcome-axis model ran its reading
/// from `1.7e2` to `2.0e10` across a single rebuild while every measurement
/// in between returned good on its residual, took no spectral floor for
/// thousands of labels, and stopped the label path outright with a refused
/// factorisation (´rep:assayer:declined-rebuild-cadence´).
///
/// The bar is the same threshold and not a new one, and applying it to this
/// component does not restore the trap the cadence record retired. That trap
/// was the *interval* shortening against a quantity shortening could not
/// remove, so the model sat at the cadence floor for ever and the
/// factorisations it paid for achieved nothing
/// (´dec:posterior:adaptive-cadence´). Here the quantity calls for a rebuild
/// and the rebuild removes it — that is the one thing a rebuild is
/// unambiguously for — while the interval still moves on the residual alone.
/// A model with no clamp mass reads exactly zero here and is untouched, which
/// is the case the restructure was built for.
///
/// **The resolution.** A residual at or below the rounding level of the
/// reading it was taken out of is the rounding of that subtraction rather
/// than drift, and the measurement records that (´def:monitoring:synchronisation-error´).
/// It is a reading and not a reprieve: it cannot carry a residual over the
/// threshold, because the resolution is computed from the operands and a pair
/// that has diverged by orders reports a resolution large enough to excuse
/// anything. That is the refinement's vetoed half, and the flag survives
/// because what it says is true and worth publishing.
///
/// A non-finite reading is good on no ground, and the finiteness test comes
/// first. Nothing meaningful was measured, and a comparison against a
/// threshold that a NaN fails silently would read the absence of a number as
/// the absence of drift.
#[must_use]
pub fn measure_synchronisation(
    precision: &SymmetricMatrix,
    covariance: &SymmetricMatrix,
    inputs: &VisitInputs<'_>,
) -> SynchronisationMeasurement {
    let reading = synchronisation_reading(precision, covariance);
    let prior_induced = prior_induced_synchronisation_error(covariance, inputs.clamp_mass);
    let residual = residual_synchronisation_error(reading.measured, prior_induced);

    let finite = residual.is_finite() && reading.measured.is_finite() && prior_induced.is_finite();
    let at_resolution = finite && residual <= reading.resolution;
    let drift_is_real = !finite || residual > inputs.sync_error_threshold;
    let good = finite && residual <= inputs.sync_error_threshold && prior_induced <= inputs.sync_error_threshold;

    SynchronisationMeasurement {
        reading,
        prior_induced,
        residual,
        at_resolution,
        good,
        drift_is_real,
    }
}

/// The rebuild: the spectrum, the floor, the factorisation, the measurement
/// of what it produced, and the verdict on whether that pair is better than
/// the pair the model holds.
///
/// This is the only site that applies the spectral floor and the only site
/// that measures a rebuilt pair, and those two facts are the same fact
/// (´dec:posterior:measured-adoption´), (´dec:posterior:spectral-floor´). A
/// floor is an addition to the precision matrix, and any addition to the
/// precision matrix that the covariance does not follow is read by the
/// monitor as drift — exactly and immediately, since `‖(B + FI)Σ − I‖`
/// exceeds `‖BΣ − I‖` by `F‖Σ‖`, which at the floor's own magnitude is of
/// order one. Applying the floor where the covariance is being rebuilt anyway
/// costs nothing beyond the factorisation already paid for and leaves the
/// pair consistent by construction.
///
/// **The spectrum is read here and not at the measurement, and that placement
/// is the design.** It is an eigendecomposition, cubic and several times the
/// cost of the product the measurement takes, and the floor it derives is
/// applied at the rebuild — so a measurement that read it would be paying for
/// a quantity it could not act on. What the fast check needs of the last
/// spectrum it gets from the record, which a measurement carries forward
/// unchanged because a visit that applied no floor changed no floor.
///
/// The order is: read the spectrum, derive the floor from the largest
/// eigenvalue, add the deficit the least eigenvalue is short of it, factor
/// the floored matrix, and measure the result against that same floored
/// matrix — which is the matrix the model will hold if the result is adopted.
///
/// **Adoption is by measurement, not by verdict, and the test is unchanged.**
/// The pair is returned — and therefore installed — only where the drift
/// after is finite, is no worse than the drift before, and is under the
/// threshold the synchronisation monitor fixes
/// (´alg:gaussian:synchronisation-monitor´). What has changed is what a
/// failure of that test means: a rebuild now runs only where a measurement
/// found drift worth acting on, so a rebuild that leaves the model no better
/// is the alarm rather than a routine decline
/// (´rep:assayer:declined-rebuild-cadence´).
///
/// The caller remains responsible for installing the returned pair, resetting
/// the label counter, adding the deficit to the identity-shaped mass, storing
/// the baseline, applying the cadence policy, and emitting health events. This
/// function changes no state.
///
/// **The maintained covariance is not an argument, and its absence is a
/// consequence of the split worth naming.** The rebuild's only use for it was
/// the drift-before reading of the adoption test, and that reading is now
/// taken at the measurement and handed here. What is left is a function of
/// the precision matrix alone: read its spectrum, floor it, factor it, and
/// measure the answer against the matrix that was factored. Nothing the model
/// currently holds enters the arithmetic, which is why a rebuild can be asked
/// for directly by a caller that wants one.
///
/// # Arguments
///
/// * `precision` — the precision matrix `B` the model holds
/// * `measurement` — what the visit's measurement found, whose reading is the
///   drift-before operand of the adoption test and whose residual the outcome
///   carries to the cadence
/// * `inputs` — the fast check's readings, the standing prior mass, and the
///   drift threshold this rebuild must come in under
///
/// # Returns
///
/// The outcome, the pair where it was adopted and `None` where it was not,
/// and the record of what the rebuild answered about itself.
#[must_use]
pub fn rebuild_covariance(
    precision: &SymmetricMatrix,
    measurement: &SynchronisationMeasurement,
    inputs: &VisitInputs<'_>,
) -> (RecomputeOutcome, Option<RebuiltPair>, RebuildRecord) {
    let started = Instant::now();
    let sync_error_before = measurement.reading.measured;

    // The true spectrum, the floor the evidence in it derives, and the deficit
    // the least eigenvalue is short of that floor
    // (´dec:posterior:spectral-floor´). Absent where the eigensolver returned
    // no verdict, in which case no floor is applied.
    let spectrum = SpectralReading::measure(precision, inputs.spectral_floor_mass);
    let deficit = spectrum.map_or(0.0, |reading| reading.deficit);

    // The matrix the rebuild is about, and the matrix the model will hold if
    // the rebuild is adopted. They are the same matrix, which is the point.
    let floored = if deficit > 0.0 {
        precision.add_scaled_identity(deficit)
    } else {
        precision.clone()
    };

    let (candidate, verdict, refused_pivot) = match cholesky_inverse(&floored, CholeskyCallSite::Periodic) {
        CholeskyInverseResult::Clean(sigma_new) => (Some(sigma_new), RebuildVerdict::Clean, None),
        CholeskyInverseResult::Failed { pivot } => (None, RebuildVerdict::Refused, Some(pivot)),
    };

    // The drift after, against the floored matrix — which is the matrix the
    // model will hold. Measuring against a matrix the model would not hold is
    // the substitution that lets a shifted inverse certify itself, and the
    // floor is written into the model precisely so that no such substitution
    // is needed here. A refused factorisation offered nothing, so the reading
    // that stands after it is the one that stood before it.
    let sync_error_after = candidate
        .as_ref()
        .map_or(sync_error_before, |sigma| synchronisation_error(&floored, sigma));

    // Unchanged: finite, no worse than before, and under the threshold. A
    // reading equal to the one before is not a worse one, because a rebuild of
    // a matrix that has not moved reproduces the covariance already held.
    let is_better =
        sync_error_after.is_finite() && sync_error_after <= sync_error_before && sync_error_after < inputs.sync_error_threshold;
    let adopted = is_better && candidate.is_some();

    let wall_nanos = u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX);

    let record = RebuildRecord {
        spectrum,
        sync_error_after,
        verdict,
        adopted,
        wall_nanos,
    };

    let outcome = if adopted {
        RecomputeOutcome::Rebuilt {
            sync_error_residual: measurement.residual,
            drift_was_real: measurement.drift_is_real,
        }
    } else {
        RecomputeOutcome::Alarm {
            sync_error_residual: measurement.residual,
            sync_error_after,
            refused_pivot,
        }
    };

    let pair = if adopted {
        candidate.map(|sigma| RebuiltPair {
            precision: floored,
            covariance: sigma,
        })
    } else {
        None
    };

    (outcome, pair, record)
}

/// One visit to a model: measure, and rebuild only where the measurement was
/// bad.
///
/// # The restructure this function is
///
/// The cadence and the cheap arms trigger *this*, and this measures. Where
/// the measurement is good nothing is factored, the reading is recorded, and
/// the interval recovers; where it is bad the rebuild runs and its result is
/// adopted or raises the alarm (´dec:posterior:recomputation-trigger´),
/// (´dec:posterior:adaptive-cadence´). The arrangement it replaces triggered
/// the rebuild directly, so a healthy model whose drift sat three orders under
/// its own threshold paid a cubic factorisation at the cadence's floor rate
/// and had that factorisation declined every time — the declines were correct,
/// the model was healthy, and the interval was pinned at its floor by them
/// (´rep:assayer:declined-rebuild-cadence´).
///
/// The consequence worth stating plainly is that there is no longer any
/// decline that is not an alarm. A rebuild runs only where a measurement found
/// drift over the threshold and above the resolution of its own reading, so a
/// rebuild that leaves the model no better is a model whose drift is real and
/// whose fresh inverse is worse than the pair it holds. That is a condition to
/// publish, not a routine outcome to count.
///
/// `skip_the_measurements_verdict` comes from the trigger, and is set by every
/// arm except the counter: the measurement is still taken, and the rebuild is
/// entered whatever it says. Those arms read the precision matrix rather than
/// the pair, and the measurement has nothing to say about what they found.
///
/// `previous` is the record the last visit left, and it is read for one
/// purpose: a measurement carries the last rebuild's own record forward
/// unchanged, because the spectrum in that record and the floor derived from
/// it are still the ones in force.
///
/// # Returns
///
/// The outcome, the pair where a rebuild ran and was adopted, and the baseline
/// the fast check will read until the next visit.
#[must_use]
pub fn synchronisation_visit(
    precision: &SymmetricMatrix,
    covariance: &SymmetricMatrix,
    previous: Option<&RecomputeBaseline>,
    skip_the_measurements_verdict: bool,
    inputs: &VisitInputs<'_>,
) -> (RecomputeOutcome, Option<RebuiltPair>, RecomputeBaseline) {
    let started = Instant::now();

    let measurement = measure_synchronisation(precision, covariance, inputs);

    let (outcome, pair, rebuild) = if measurement.good && !skip_the_measurements_verdict {
        (
            RecomputeOutcome::Measured {
                sync_error_residual: measurement.residual,
                at_resolution: measurement.at_resolution,
            },
            None,
            previous.and_then(|record| record.rebuild),
        )
    } else {
        let (outcome, pair, record) = rebuild_covariance(precision, &measurement, inputs);
        (outcome, pair, Some(record))
    };

    let wall_nanos = u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX);

    (
        outcome,
        pair,
        RecomputeBaseline::from_visit(&measurement, inputs, wall_nanos, rebuild),
    )
}
