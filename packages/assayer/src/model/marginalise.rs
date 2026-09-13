// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Schur complement marginalisation, whose correction uses a half-solve
//! (´dec:posterior:half-solve´).
//!
//! Implements the information-preserving dimension removal via the
//! Schur complement, replacing the $B_{kk}$ fallback.
//!
//! # Five-Stage Algorithm
//!
//! ```text
//! Stage 1 — Spectrum: λ_max, λ_min of B_rr    O(r³)
//! Stage 2 — Guards:   posture on κ(B_rr),     O(1)
//!                     solvability on κ(B_rr + δI)
//! Stage 3 — Factor:   (B_rr + δI) = LLᵀ       O(r³/3)
//! Stage 4 — Solve:    LW = B_rk; C = WᵀW      O(r²k + rk²)
//! Stage 5 — Verify:   (B' − floor·I) = LLᵀ      O(k³/3)
//! ```
//!
//! Stage 1 replaces a diagonal ratio that was never the condition number it
//! was read as (´alg:gaussian:regularised-schur´). It is what makes the two
//! guards separable: the raw spectrum answers the host's question about the
//! source block, and because adding δI shifts every eigenvalue by δ and does
//! nothing else, the same spectrum answers the package's question about the
//! matrix the correction is solved against. It also supplies the denominator
//! of the error result's bound, so one decomposition serves three readers.
//!
//! Its cost is argued rather than measured, this lane having run no benchmark.
//! At the reference configuration — p = 638 losing r = 68 — the eigensolve's
//! r³ stands against the verification's k³/3 at k = 570, which is over two
//! decimal orders larger, and against the O(r²k) solve the correction already
//! pays. Where instead nearly every dimension is removed the two costs meet,
//! but the factorisation the algorithm already performs is cubic in r as well,
//! so the increase is in the constant and not in the order.
//!
//! Stage 5 factorises B' and does not read its diagonal
//! (´req:gaussian:positive-definiteness´). A Cholesky attempt on the block
//! lowered by the resolution floor completes exactly when the requirement is
//! met, so the question is answered by the cheapest witness that answers it
//! rather than by a quantity nobody asked for. It is the one stage whose cost
//! is not dominated by an earlier one, and the requirement argues the trade:
//! a positive minimum diagonal is necessary for positive definiteness and is
//! not sufficient for it, and Stage 4's correction is precisely what loads the
//! off-diagonal structure where the two tests part company. The cubic term it
//! adds is the one general marginalisation already pays to recompute the
//! covariance (´tab:gaussian:operation-costs´).
//!
//! The trade was measured rather than assumed. At the reference configuration
//! — p = 638 losing r = 68, so the verification runs at k = 570 — a whole
//! marginalisation cost about 117 ms against about 95 ms for the diagonal
//! scan while the stage computed a spectrum, so the verification's own share
//! was roughly 22 ms, about a quarter. The witness is a factorisation of the
//! floored block, which settles the same question at k³/3 where a symmetric
//! eigendecomposition spends an order of magnitude more, so that share is now
//! a fraction of the measured one and the trade is decided the same way with
//! more room. It is affordable in either case because this is not a
//! per-request path: marginalisation happens on registration and
//! deregistration, which are deployment events
//! (´tab:gaussian:operation-costs´). Were it hot the requirement would still
//! win and the cost would be reported, rather than the cheap test kept.
//!
//! Every path produces a valid `SymmetricMatrix`. The function is
//! infallible: it always returns B' (with correction) or B_kk
//! (without).
//!
//! # The error is reported, not budgeted
//!
//! Every marginalisation returns what it cost: the precision mass the offset
//! retained over the exact marginal, together with the share of the correction
//! that did not survive, each labelled as a measurement or as a bound
//! (´alg:gaussian:regularised-schur´). The offset alone is not that figure —
//! it scales with the removed block's least eigenvalue and the coupling — so
//! the algorithm computes the quantity instead of declaring a tolerance for
//! it. Where the correction is discarded the report does not fall silent: the
//! discard is the offset's own endpoint and its error is the whole correction.
//!
//! # Oracle
//!
//! In debug builds (or with a `DEBUG`-level tracing subscriber),
//! Stage 4 computes the correction via **both** methods — Option C
//! (half-solve, WᵀW Gram matrix) and Method B (full-solve,
//! B_kr · Y) — and asserts per-element agreement within 10⁻¹².
//! Same pattern as sentinel's SVD oracle.
//!
//! # Cross-References
//!
//! - (´dec:posterior:half-solve´) — the Schur complement marginalisation and
//!   the half-solve its correction uses
//! - (´thm:gaussian:marginalisation´) — the result the dimension removal
//!   preserves information under
//! - (´alg:gaussian:regularised-schur´) — the numerical conditioning, its
//!   regularisation and its condition guard
//! - (´req:gaussian:positive-definiteness´) — the verification B' must pass
//!   before it is adopted, and why a reading of its diagonal is not it
//! - (´req:gaussian:rank-one-verification´) — the same contract on the rank-1
//!   path, discharged by an outward-rounded Gershgorin certificate where it
//!   can be and by the factorisation where it cannot
//! - (´req:gaussian:marginalisation-error-reported´) — the error every path
//!   reports, why the offset alone is not it, and how it is labelled
//! - (´tab:gaussian:operation-costs´) — the marginalisation costs

// Justified: b_rr, b_kk, b_kr, b_rk are standard matrix partition names
#![allow(clippy::similar_names)]
// Justified: pervasive matrix-partition notation (B_kk, B_rr, κ̂, etc.)
#![allow(clippy::doc_markdown)]

use crate::linalg::bridge::{LltError, cholesky, eigenvalue_min_max, schur_correction_oracle, schur_half_solve};
use crate::linalg::symmetric::SymmetricMatrix;

// ═══════════════════════════════════════════════════════════════════════════════
// Verification resolution
// ═══════════════════════════════════════════════════════════════════════════════

/// Resolution of the B' verification, in multiples of the width-scaled machine
/// epsilon.
///
/// The verification asks whether B' is positive definite
/// (´req:gaussian:positive-definiteness´), and it asks arithmetic of finite
/// precision. Whichever way the question is put — a factorisation completing
/// or meeting a non-positive pivot, a disc bound clearing zero or not — the
/// answer is decided by quantities carrying an absolute error of order
/// `c(k) · ε_mach · λ_max`, where λ_max is the matrix's own spectral norm and
/// `c(k)` grows modestly with the width. The error is therefore proportional
/// to the matrix's scale, and no absolute figure can stand as the boundary:
/// the same figure would be far too coarse for a precision matrix near the
/// replenishment floor and far too fine for one carrying a well-observed
/// dimension. Inside that band the verdict is arithmetic noise rather than a
/// verdict, and the requirement's "strictly positive" cannot be read off it.
///
/// The magnitude below multiplies the bare `k · ε_mach` textbook bound, standing
/// three binary orders above it to cover the constant that the factorisation's
/// pivot growth contributes on top of the width term. It is a
/// representation limit rather than a semantic bound
/// (´dec:degradation:guard-magnitudes´): it marks where double precision stops
/// distinguishing a small positive eigenvalue from a small negative one, not
/// where a precision stops meaning anything.
///
/// Its direction is the safe one. A B' whose least eigenvalue cannot be told
/// from zero at this precision is refused, and refusal costs only the
/// correction — the fallback adopts B_kk and discards transferred information,
/// rather than adopting a precision matrix whose inverse, the covariance the
/// marginalisation goes on to restore, would be meaningless. What the margin
/// admits is everything down to a condition number near 10¹², four decimal
/// orders beyond the 10⁸ at which this same algorithm already declines to
/// invert a block (´alg:gaussian:regularised-schur´), so no B' the model would
/// treat as healthy is touched by it.
///
/// ´const:assayer:schur-verification-resolution´ (´alg:const:scalar´)
/// ´const:assayer:schur-verification-resolution-scalar-8p0´
const VERIFICATION_RESOLUTION_FACTOR: f64 = 8.0;

/// Ceiling on the condition number of the regularised block the correction is
/// actually solved against, above which the solve is declined.
///
/// This is the lower of the two guards and it is ours rather than the host's
/// (´alg:gaussian:regularised-schur´). It protects one thing: that the
/// triangular solves behind the correction still carry an answer. The matrix
/// it reads is the one they factor, `B_rr + δI`, whose spectrum is the removed
/// block's shifted by δ, so the figure is exact wherever the removed block's
/// spectrum was computed at all.
///
/// It is not host-tunable because it is not a judgement a deployment is in a
/// position to make. Where the condition number of the factored matrix reaches
/// the reciprocal of the machine epsilon the solve has no significant digits
/// left, and no posture about information quality changes that. The magnitude
/// below stands four decimal orders short of that limit, which is where the
/// verification this algorithm already performs stops being able to tell a
/// small positive least eigenvalue from a small negative one
/// (´const:assayer:schur-verification-resolution´). Setting the two at the same
/// place is deliberate: a correction whose solve is worse conditioned than the
/// verification can adjudicate would be admitted on a verdict that is itself
/// arithmetic noise.
///
/// ´const:assayer:schur-solvability-condition-ceiling´ (´alg:const:scalar´)
/// ´const:assayer:schur-solvability-condition-ceiling-scalar-1e12´
const SOLVABILITY_CONDITION_CEILING: f64 = 1e12;

/// Digits of headroom demanded before the exact marginalisation error is
/// computed rather than bounded.
///
/// The exact error is the difference between the correction the exact inverse
/// would give and the correction the regularised inverse did give. It is
/// computed by solving against the unregularised removed block, and that solve
/// carries a relative error of order κ(B_rr) · ε_mach, while the difference
/// being resolved is a fraction δ/(λ_min + δ) of the quantity it is taken
/// between. Where the error of the terms is not comfortably smaller than the
/// difference between them, the subtraction returns noise and the figure is
/// not a measurement of anything.
///
/// The factor below is that comfort, one decimal order of it. It is a
/// resolution test rather than a quality threshold: it decides which of two
/// honest answers the host receives, never whether an answer is given
/// (´alg:gaussian:regularised-schur´).
///
/// ´const:assayer:schur-exact-error-headroom´ (´alg:const:scalar´)
/// ´const:assayer:schur-exact-error-headroom-scalar-10p0´
const EXACT_ERROR_HEADROOM: f64 = 10.0;

// ═══════════════════════════════════════════════════════════════════════════════
// Configuration
// ═══════════════════════════════════════════════════════════════════════════════

/// Configuration for the Schur complement computation.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SchurConfig {
    /// Regularisation factor ε_Schur. The regularisation the computation
    /// adds is derived from it rather than held beside it:
    /// δ = λ_prior × ε_Schur, taken against the prior precision of the
    /// model being marginalised (´alg:gaussian:regularised-schur´). The
    /// factor is what a deployment sets, so a deployment that moves its
    /// prior moves the regularisation with it.
    /// Default: 10⁻⁴.
    pub epsilon_schur: f64,

    /// The host's posture ceiling on the condition number of the block
    /// about to be removed, read before regularisation.
    ///
    /// This is the upper of the two guards and the only one a deployment
    /// sets (´alg:gaussian:regularised-schur´). It does not protect the
    /// arithmetic — the regularised solve is protected by a package-side
    /// guard the host cannot move — it declares how degraded a source
    /// block the deployment is willing to fold into its surviving models.
    /// A block whose raw condition number exceeds this ceiling carries
    /// information the host has said it does not want at any arithmetic
    /// quality, and the correction is declined.
    ///
    /// Raising it admits more degraded sources and, because the exact
    /// error term is then itself numerically unreachable, moves the
    /// reported marginalisation error from a measurement to a bound.
    /// Default: 10⁸, the figure the single guard it descends from carried.
    pub posture_condition_ceiling: f64,
}

impl Default for SchurConfig {
    fn default() -> Self {
        Self {
            epsilon_schur: 1e-4,
            posture_condition_ceiling: 1e8,
        }
    }
}

impl SchurConfig {
    /// The regularisation δ = λ_prior × ε_Schur this configuration asks for
    /// against a model whose prior precision is `lambda_prior`
    /// (´alg:gaussian:regularised-schur´).
    ///
    /// At the default factor and the default prior this is 10⁻⁵, which is
    /// the value the fixed constant this derivation replaced carried.
    #[must_use]
    pub fn delta(&self, lambda_prior: f64) -> f64 {
        lambda_prior * self.epsilon_schur
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Diagnostics
// ═══════════════════════════════════════════════════════════════════════════════

/// Which computation path the Schur complement took.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchurCorrectionOutcome {
    /// Full correction applied. Information transferred.
    Applied,
    /// The host's posture guard fired: the removed block's raw condition
    /// number exceeded the ceiling the deployment declared, so the block was
    /// refused as a source rather than as a computation
    /// (´alg:gaussian:regularised-schur´). Correction skipped.
    PostureGuardSkip,
    /// The package's solvability guard fired: the regularised block the
    /// correction would be solved against is worse conditioned than the
    /// arithmetic supports (´alg:gaussian:regularised-schur´). Correction
    /// skipped. This one the host cannot move.
    SolvabilityGuardSkip,
    /// Cholesky on B_rr + δI failed. Correction skipped.
    FactorisationFailed {
        /// Index of the first non-positive pivot.
        pivot_index: usize,
    },
}

/// What the reported condition figure of the removed block actually is.
///
/// The distinction is not decoration. A diagonal ratio is a lower bound on the
/// spectral condition number and can be one for a block whose true condition
/// number is arbitrary, so a guard reading it refuses far less than it appears
/// to (´alg:gaussian:regularised-schur´).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionMeasure {
    /// λ_max/λ_min of the removed block, from its computed spectrum. This is
    /// the condition number, up to the eigensolver's own error.
    Spectral,
    /// max/min of the removed block's diagonal, used because the eigensolver
    /// did not converge. It is a certified lower bound on the condition
    /// number and certifies nothing when it is small.
    DiagonalRatioLowerBound,
}

/// Which certificate the positive-definiteness verdict of B' was read from.
///
/// The rank-1 path carries two certificates rather than one, and the
/// difference between them is a cost difference and not a strength
/// difference: both decide the same boundary
/// (´req:gaussian:rank-one-verification´). Naming the one that decided is what
/// makes the cheap certificate's acceptance rate a measurement rather than an
/// assumption — the figure on which the whole arrangement's value rests, and
/// the one thing no amount of analysis supplies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationCertificate {
    /// No B' was put to a certificate. Either the correction was refused
    /// before it was formed, or the surviving block has no dimensions and the
    /// requirement quantifies over an empty spectrum. Events reporting this
    /// stand outside the acceptance-rate denominator, having asked nothing of
    /// either certificate.
    Vacuous,
    /// The outward-rounded Gershgorin bound cleared the resolution floor, so
    /// the discs alone certified the result at O(p²) and no spectrum was
    /// computed.
    Gershgorin,
    /// A non-positive diagonal entry disproved positive definiteness, so the
    /// verdict was refusal at O(p) and no spectrum was computed. A diagonal
    /// entry is a Rayleigh quotient, so the refusing direction of the diagonal
    /// test is a proof where the accepting direction is not.
    DiagonalDisproof,
    /// The factorisation decided, the cheap certificate having been
    /// inconclusive or never sought. A Cholesky attempt on B' lowered by the
    /// resolution floor either completed, which is the verdict of acceptance,
    /// or met a non-positive pivot, which is the verdict of refusal. This is
    /// the arm that costs O(p³/3), and it is the arm the general path always
    /// takes.
    Factorisation,
}

/// The marginalisation error one event committed, and whether the figure is a
/// measurement of it or an upper bound on it.
///
/// The quantity both arms carry is the trace of `E_δ = S_δ − S_0`, the
/// precision the regularisation retained over the exact marginal
/// (´alg:gaussian:regularised-schur´). Because `E_δ` is positive
/// semi-definite its trace is also an upper bound on its spectral norm, so one
/// scalar serves both readings; and because the fallback is the endpoint at
/// which the whole correction is discarded, the same quantity describes it
/// with no separate accounting.
///
/// The two arms are separate variants rather than a number beside a flag so
/// that a bound cannot be read as a measurement by a caller that forgot to
/// look at the flag. Reading either figure costs a match on which it is.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MarginalisationError {
    /// The error was computed from the exact correction and is a measurement.
    Measured {
        /// tr(E_δ): the precision mass regularisation retained over the exact
        /// marginal, in the precision matrix's own units.
        discarded_precision_mass: f64,
        /// The share of the exact correction that did not survive, in `[0, 1]`
        /// — dimensionless, and so comparable across events whose scales the
        /// forgetting has moved apart. One means the correction was discarded
        /// whole.
        correction_loss_fraction: f64,
    },
    /// The exact error was not numerically reachable and the figure is an
    /// upper bound on it, from the removed block's least eigenvalue and the
    /// coupling's Frobenius norm.
    Bounded {
        /// An upper bound on tr(E_δ).
        discarded_precision_mass: f64,
        /// An upper bound on the share of the exact correction that did not
        /// survive, in `[0, 1]`.
        correction_loss_fraction: f64,
    },
    /// Neither figure is available: the removed block has no usable least
    /// eigenvalue, so the bound's denominator does not exist. The error is
    /// real and this says only that the corpus can put no finite figure on it.
    Unbounded,
}

impl MarginalisationError {
    /// The exact zero of an event that removed nothing.
    pub const NONE: Self = Self::Measured {
        discarded_precision_mass: 0.0,
        correction_loss_fraction: 0.0,
    };

    /// The discarded precision mass, whether measured or bounded, or `None`
    /// where no finite figure exists. Callers that must distinguish the two
    /// match on the variant instead.
    #[must_use]
    pub const fn discarded_precision_mass(&self) -> Option<f64> {
        match *self {
            Self::Measured {
                discarded_precision_mass,
                ..
            }
            | Self::Bounded {
                discarded_precision_mass,
                ..
            } => Some(discarded_precision_mass),
            Self::Unbounded => None,
        }
    }

    /// The share of the correction lost, whether measured or bounded, or
    /// `None` where no finite figure exists.
    #[must_use]
    pub const fn correction_loss_fraction(&self) -> Option<f64> {
        match *self {
            Self::Measured {
                correction_loss_fraction,
                ..
            }
            | Self::Bounded {
                correction_loss_fraction,
                ..
            } => Some(correction_loss_fraction),
            Self::Unbounded => None,
        }
    }

    /// Whether the figures are measurements rather than bounds.
    #[must_use]
    pub const fn is_measured(&self) -> bool {
        matches!(*self, Self::Measured { .. })
    }
}

/// Diagnostic information from the Schur complement computation.
#[derive(Debug, Clone)]
pub struct SchurDiagnostics {
    /// Diagonal ratio max_i B_rr[i,i] / min_i B_rr[i,i], read before
    /// regularisation.
    ///
    /// This is a lower bound on the spectral condition number of B_rr and
    /// not a measurement of it. A matrix with equal diagonals reports one
    /// here while carrying an arbitrarily large true condition number in
    /// its off-diagonal structure, so a small figure certifies nothing and
    /// only a large figure is evidence (´alg:gaussian:regularised-schur´).
    pub brr_cond_estimate: f64,
    /// What the figure above is: the spectrum, or the diagonal ratio standing
    /// in for it because the eigensolver did not converge.
    pub brr_cond_measure: ConditionMeasure,
    /// The condition number of the regularised block the solvability guard
    /// reads, `(λ_max + δ)/(λ_min + δ)`. Absent where the spectrum was not
    /// available, in which case that guard had nothing to read and was not
    /// evaluated.
    pub brr_regularised_cond: Option<f64>,
    /// Which computation path was taken.
    pub correction_outcome: SchurCorrectionOutcome,
    /// Whether B' passed the SPD verification
    /// (´req:gaussian:positive-definiteness´).
    pub bprime_verification_passed: bool,
    /// Which certificate that verdict was read from
    /// (´req:gaussian:rank-one-verification´).
    ///
    /// The general path always answers with the spectrum, so on that path this
    /// carries no information the flag above does not. On the rank-1 path it
    /// carries the measurement the arrangement was adopted on: how often the
    /// cheap certificate is enough.
    pub bprime_verification_certificate: VerificationCertificate,
    /// Whether the final result is B_kk (no information transfer)
    /// rather than the full Schur complement.
    pub fell_back_to_bkk: bool,
    /// The approximation this event actually committed, computed rather than
    /// assumed, and labelled as a measurement or a bound.
    pub error: MarginalisationError,
}

impl SchurDiagnostics {
    /// Returns a no-op diagnostics value for the identity case
    /// (nothing removed).
    pub const fn noop() -> Self {
        Self {
            brr_cond_estimate: 1.0,
            brr_cond_measure: ConditionMeasure::Spectral,
            brr_regularised_cond: None,
            correction_outcome: SchurCorrectionOutcome::Applied,
            bprime_verification_passed: true,
            bprime_verification_certificate: VerificationCertificate::Vacuous,
            fell_back_to_bkk: false,
            error: MarginalisationError::NONE,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Accumulation
// ═══════════════════════════════════════════════════════════════════════════════

/// What repeated marginalisation has cost the models, folded across lifecycle
/// events.
///
/// Each event's error is a statement about that event alone. The corpus states
/// no law composing them — the events act on different partitions and are
/// separated by updates and forgetting — so this is a monitor and not a bound
/// (´inv:guarantee:structural-exactness´). What it answers is the question a
/// single event cannot: whether a deployment is accumulating approximation at
/// a rate that argues for a recompute, or has simply had one bad removal.
///
/// The counters are monotone and are never reset. They live with the model
/// owner and cross its whole-state checkpoint, so they describe the deployment
/// across process restarts rather than only the current process
/// (´dec:durability:checkpoint-journal´).
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MarginalisationErrorLedger {
    /// Total marginalisations folded in, across every model and event.
    pub events: u64,
    /// Total whose error was measured rather than bounded.
    pub measured_events: u64,
    /// Total whose error could only be bounded, the exact term being
    /// numerically out of reach.
    pub bounded_events: u64,
    /// Total for which no finite figure existed at all.
    pub unbounded_events: u64,
    /// Total that discarded the correction and adopted B_kk, by any of the two
    /// guards, the factorisation, or the verification.
    pub fallback_events: u64,
    /// Total that put a formed B' to a positive-definiteness certificate, by
    /// either path (´req:gaussian:rank-one-verification´). Events refused
    /// before B' existed are not counted, having asked nothing of either
    /// certificate; this is the denominator the figure below is read against.
    pub verified_events: u64,
    /// Total of those the outward-rounded Gershgorin bound settled on its own,
    /// without a spectrum being computed.
    ///
    /// This is the measurement the cheap certificate was adopted on. The
    /// argument for it is that dense healthy matrices need not be strictly
    /// diagonally dominant, so how often the discs are enough is an empirical
    /// question about the precision matrices a deployment actually produces,
    /// and no analysis settles it in advance. Counted here so that a
    /// deployment answering it badly is visible rather than assumed away.
    pub gershgorin_certified_events: u64,
    /// The sum of the per-event discarded precision mass, over the events that
    /// carried a figure. A sum of measurements and bounds alike, so it is an
    /// upper estimate; in the precision matrices' own units, which forgetting
    /// moves, so it is read as a trend rather than as a level.
    pub cumulative_discarded_precision_mass: f64,
    /// The largest share of a correction any single event lost, in `[0, 1]`.
    /// Dimensionless, so unlike the sum above it is comparable across a
    /// model's whole life.
    pub worst_correction_loss_fraction: f64,
}

impl MarginalisationErrorLedger {
    /// Folds one event's diagnostics into the running picture.
    pub fn record(&mut self, diagnostics: &SchurDiagnostics) {
        self.events += 1;
        if diagnostics.fell_back_to_bkk {
            self.fallback_events += 1;
        }
        match diagnostics.bprime_verification_certificate {
            VerificationCertificate::Gershgorin => {
                self.verified_events += 1;
                self.gershgorin_certified_events += 1;
            }
            VerificationCertificate::DiagonalDisproof | VerificationCertificate::Factorisation => {
                self.verified_events += 1;
            }
            VerificationCertificate::Vacuous => {}
        }
        match diagnostics.error {
            MarginalisationError::Measured {
                discarded_precision_mass,
                correction_loss_fraction,
            } => {
                self.measured_events += 1;
                self.cumulative_discarded_precision_mass += discarded_precision_mass;
                self.worst_correction_loss_fraction = self.worst_correction_loss_fraction.max(correction_loss_fraction);
            }
            MarginalisationError::Bounded {
                discarded_precision_mass,
                correction_loss_fraction,
            } => {
                self.bounded_events += 1;
                self.cumulative_discarded_precision_mass += discarded_precision_mass;
                self.worst_correction_loss_fraction = self.worst_correction_loss_fraction.max(correction_loss_fraction);
            }
            MarginalisationError::Unbounded => {
                self.unbounded_events += 1;
                // An event with no finite figure is the one case where the
                // worst-loss figure cannot be moved honestly. It is counted
                // above so that the count and the maximum are read together.
            }
        }
    }

    /// Total corrections skipped: the events that adopted B_kk instead of the
    /// Schur complement (´entry:posterior:skip-counter´).
    #[must_use]
    pub const fn corrections_skipped(&self) -> u64 {
        self.fallback_events
    }

    /// The share of verified events the cheap certificate settled on its own,
    /// in `[0, 1]`, or `None` before any B' has been verified
    /// (´req:gaussian:rank-one-verification´).
    ///
    /// This is the study's open measurement made readable. It is the share of
    /// verifications that stayed quadratic, so it is what says whether the
    /// fast certificate is buying a deployment anything; a figure near zero
    /// means the spectral fallback is being paid on nearly every removal and
    /// the arrangement is costing more than the plain spectral rule it stands
    /// in front of. It is not a correctness figure — every verdict is the same
    /// verdict either way — and must not be read as one.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn gershgorin_acceptance_rate(&self) -> Option<f64> {
        (self.verified_events > 0).then(|| self.gershgorin_certified_events as f64 / self.verified_events as f64)
    }
}

/// Result of a Schur complement computation.
#[derive(Clone, Debug)]
pub struct SchurResult {
    /// The marginal precision B'.
    /// Either B_kk − correction (full Schur) or B_kk (fallback).
    pub b_prime: SymmetricMatrix,
    /// Diagnostic information for health reporting.
    pub diagnostics: SchurDiagnostics,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Algorithm
// ═══════════════════════════════════════════════════════════════════════════════

/// Computes the Schur complement for dimension marginalisation.
///
/// Given precision matrix B partitioned as:
/// ```text
///     ┌───────┬───────┐
///     │ B_kk  │ B_kr  │
///     ├───────┼───────┤
///     │ B_rk  │ B_rr  │
///     └───────┴───────┘
/// ```
/// Computes B' = B_kk − B_kr (B_rr + δI)⁻¹ B_rk.
///
/// The regularisation is derived here rather than supplied ready-made:
/// δ = λ_prior × ε_Schur, where `lambda_prior` is the prior precision of
/// the model whose precision this is and ε_Schur is the configured factor
/// (´alg:gaussian:regularised-schur´). The scale δ has to hold against is
/// the scale of B itself, which the prior sets, so the caller passes what
/// it built the model with.
///
/// Always returns a valid `SymmetricMatrix`:
/// - B' = B_kk − correction (if correction applied and B' is SPD)
/// - B_kk (if guard fired, factorisation failed, or verification failed)
pub fn compute_schur_complement(
    precision: &SymmetricMatrix,
    remove_indices: &[usize],
    keep_indices: &[usize],
    config: &SchurConfig,
    lambda_prior: f64,
) -> SchurResult {
    debug_assert!(
        remove_indices.windows(2).all(|w| w[0] < w[1]),
        "remove_indices must be sorted ascending"
    );
    debug_assert!(
        keep_indices.windows(2).all(|w| w[0] < w[1]),
        "keep_indices must be sorted ascending"
    );
    debug_assert_eq!(
        remove_indices.len() + keep_indices.len(),
        precision.dim(),
        "remove + keep must partition 0..p"
    );

    let r = remove_indices.len();

    // Edge case: nothing to remove.
    if r == 0 {
        return SchurResult {
            b_prime: precision.clone(),
            diagnostics: SchurDiagnostics::noop(),
        };
    }

    // ── Step 1: Extract B_rr, B_kk and the coupling ────────────────────
    let b_rr = precision.extract_symmetric_submatrix(remove_indices);
    let b_kk = precision.extract_symmetric_submatrix(keep_indices);

    // B_kr: k × r (rows = kept, cols = removed). The extraction used to be
    // deferred past the guard to save it on a refused block. A refused block
    // is now exactly the case whose error the host is owed — discarding the
    // correction is an error of the correction's own size — and the coupling
    // is what sizes it, so it is extracted on every path.
    let b_kr = precision.extract_cross_block(keep_indices, remove_indices);
    let b_rk = b_kr.transpose().to_owned();

    // ── Step 2: Read the removed block's spectrum ──────────────────────
    let delta = config.delta(lambda_prior);
    let conditioning = Conditioning::read(&b_rr, delta);

    // ── Step 3: The two guards ─────────────────────────────────────────
    if let Some(outcome) = conditioning.guard_verdict(config.posture_condition_ceiling) {
        tracing::info!(
            ?outcome,
            raw_condition = conditioning.estimate,
            raw_condition_is = ?conditioning.measure,
            regularised_condition = ?conditioning.regularised,
            posture_ceiling = config.posture_condition_ceiling,
            solvability_ceiling = SOLVABILITY_CONDITION_CEILING,
            "Schur guard fired; B' = B_kk"
        );
        let error = marginalisation_error(&b_rr, &b_rk, delta, conditioning.spectrum, None);
        return conditioning.refusal(b_kk, outcome, error);
    }

    // ── Step 4: Regularise and factor ──────────────────────────────────
    // B_rr_reg = B_rr + δI, with δ = λ_prior × ε_Schur.
    let b_rr_reg = b_rr.add_scaled_identity(delta);

    let llt = match cholesky(&b_rr_reg) {
        Ok(llt) => llt,
        Err(llt_error) => {
            let LltError::NonPositivePivot { index } = llt_error;
            tracing::warn!(
                pivot_index = index,
                delta,
                "Schur B_rr factorisation failed despite δ regularisation"
            );
            let error = marginalisation_error(&b_rr, &b_rk, delta, conditioning.spectrum, None);
            let outcome = SchurCorrectionOutcome::FactorisationFailed { pivot_index: index };
            return conditioning.refusal(b_kk, outcome, error);
        }
    };

    // ── Step 5: Half-solve and form correction (Option C) ────────────────
    // Solve L W = B_rk via forward substitution (half-solve).
    // Correction = W^T W (Gram matrix: symmetric and PSD by construction).
    //
    // W^T W = B_rk^T L^{-T} L^{-1} B_rk = B_kr (LL^T)^{-1} B_rk
    //       = B_kr (B_rr + δI)^{-1} B_rk    ✓
    //
    // Option C (half-solve) is preferred over Method B (full-solve) because
    // W^T W is structurally PSD (´dec:posterior:half-solve´).
    let w = schur_half_solve(&llt, &b_rk);

    // Correction = W^T W   (r × k)^T · (r × k) = k × k
    let correction_inner = w.transpose() * &w;

    // Oracle: cross-check against Method B in debug builds.
    schur_correction_oracle(&correction_inner, &llt, &b_rk, &b_kr);

    // The trace of the applied correction is the squared Frobenius norm of the
    // half-solved factor, which costs O(rk) and needs no k × k matrix formed
    // to read it: tr(WᵀW) = ‖W‖_F².
    let applied_correction_trace = frobenius_squared(&w);
    let error = marginalisation_error(&b_rr, &b_rk, delta, conditioning.spectrum, Some(applied_correction_trace));

    let correction = SymmetricMatrix::from_computation(correction_inner);

    // ── Step 6: Subtract and verify ────────────────────────────────────
    // B' = B_kk − correction
    let b_prime = b_kk.subtract(&correction);

    // Verify B' positive definite by factorising it, not by reading its
    // diagonal (´req:gaussian:positive-definiteness´). The correction
    // subtracted just above is what loads the off-diagonal structure, so this
    // is exactly the position in which a strictly positive diagonal stops
    // implying a strictly positive spectrum.
    let (verification_passed, certificate, refusal) = verify_positive_definite(&b_prime);
    if verification_passed {
        SchurResult {
            b_prime,
            diagnostics: conditioning.diagnostics(SchurCorrectionOutcome::Applied, true, certificate, false, error),
        }
    } else {
        if let Some(FactorisationRefusal { pivot_index, floor }) = refusal {
            tracing::warn!(pivot_index, floor, "Schur B' verification failed; falling back to B_kk");
        } else {
            tracing::warn!("Schur B' verification failed on a B' that bounds nothing; falling back to B_kk");
        }
        SchurResult {
            b_prime: b_kk,
            diagnostics: conditioning.diagnostics(
                SchurCorrectionOutcome::Applied,
                false,
                certificate,
                true,
                // The correction was computed and then refused, so what this
                // event lost is the whole of it and not the offset's share.
                marginalisation_error(&b_rr, &b_rk, delta, conditioning.spectrum, None),
            ),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// The two guards
// ═══════════════════════════════════════════════════════════════════════════════

/// What the guards read, taken from one decomposition of the removed block.
///
/// One symmetric eigensolve answers three questions at once: what the host's
/// posture guard reads of the raw block, what the package's solvability guard
/// reads of the matrix the correction is actually solved against, and what the
/// error result's bound divides by. Adding δI shifts every eigenvalue by δ and
/// does nothing else, so the second follows from the first with no further
/// decomposition (´alg:gaussian:regularised-schur´).
struct Conditioning {
    /// `(λ_max, λ_min)` of the removed block, absent where the eigensolver
    /// did not converge.
    spectrum: Option<(f64, f64)>,
    /// The raw condition figure the posture guard reads.
    estimate: f64,
    /// What that figure is.
    measure: ConditionMeasure,
    /// The regularised condition figure the solvability guard reads, absent
    /// where there was no spectrum to shift.
    regularised: Option<f64>,
}

impl Conditioning {
    /// Reads the removed block once.
    fn read(b_rr: &SymmetricMatrix, delta: f64) -> Self {
        let spectrum = removed_block_spectrum(b_rr);
        let (estimate, measure) = spectrum.map_or_else(
            || {
                // Without a spectrum the diagonal ratio is what is left. It is
                // a certified lower bound on the condition number rather than
                // the number, so the posture guard still refuses everything it
                // can prove bad and proves nothing by admitting.
                let (max_diagonal, min_diagonal) = b_rr.diagonal_min_max();
                (
                    condition_ratio(max_diagonal, min_diagonal),
                    ConditionMeasure::DiagonalRatioLowerBound,
                )
            },
            |(lambda_max, lambda_min)| (condition_ratio(lambda_max, lambda_min), ConditionMeasure::Spectral),
        );
        let regularised = spectrum.map(|(lambda_max, lambda_min)| condition_ratio(lambda_max + delta, lambda_min + delta));
        Self {
            spectrum,
            estimate,
            measure,
            regularised,
        }
    }

    /// Which guard, if either, refuses this block.
    ///
    /// The host's posture is asked first because it is a question about the
    /// block as a source of information, which is prior to any question about
    /// the arithmetic that would consume it. The package's solvability guard
    /// is asked second, and only where a spectrum exists for it to read: with
    /// none, it has nothing to say and the factorisation is left as the only
    /// thing standing between an unsolvable block and a correction.
    /// The posture comparison is stated as admission rather than as refusal so
    /// that a ceiling which is not a number refuses rather than admits. It is
    /// otherwise the same comparison — equality still passes — but under the
    /// refusal form a host that set a not-a-number ceiling would silently
    /// disable its own guard, every comparison against it being false.
    fn guard_verdict(&self, posture_ceiling: f64) -> Option<SchurCorrectionOutcome> {
        let within_posture = self.estimate <= posture_ceiling;
        if !within_posture {
            return Some(SchurCorrectionOutcome::PostureGuardSkip);
        }
        if self.regularised.is_some_and(|kappa| kappa > SOLVABILITY_CONDITION_CEILING) {
            return Some(SchurCorrectionOutcome::SolvabilityGuardSkip);
        }
        None
    }

    /// The diagnostics of one marginalisation, carrying the conditioning
    /// figures both guards were judged on.
    const fn diagnostics(
        &self,
        correction_outcome: SchurCorrectionOutcome,
        bprime_verification_passed: bool,
        bprime_verification_certificate: VerificationCertificate,
        fell_back_to_bkk: bool,
        error: MarginalisationError,
    ) -> SchurDiagnostics {
        SchurDiagnostics {
            brr_cond_estimate: self.estimate,
            brr_cond_measure: self.measure,
            brr_regularised_cond: self.regularised,
            correction_outcome,
            bprime_verification_passed,
            bprime_verification_certificate,
            fell_back_to_bkk,
            error,
        }
    }

    /// The result of adopting the kept block instead of a correction.
    ///
    /// A refusal reached before B' was formed put nothing to a certificate,
    /// which is what the vacuous verdict records.
    const fn refusal(&self, b_kk: SymmetricMatrix, outcome: SchurCorrectionOutcome, error: MarginalisationError) -> SchurResult {
        SchurResult {
            b_prime: b_kk,
            diagnostics: self.diagnostics(outcome, true, VerificationCertificate::Vacuous, true, error),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// The error result
// ═══════════════════════════════════════════════════════════════════════════════

/// The removed block's spectrum as `(λ_max, λ_min)`, or `None` where the
/// eigensolver did not converge.
fn removed_block_spectrum(b_rr: &SymmetricMatrix) -> Option<(f64, f64)> {
    match eigenvalue_min_max(b_rr) {
        Ok(spectrum) => Some(spectrum),
        Err(error) => {
            tracing::warn!(
                %error,
                "Schur removed-block spectrum did not converge; the posture guard reads the diagonal ratio and the solvability guard is not evaluated"
            );
            None
        }
    }
}

/// A condition number from the two ends of a spectrum, infinite where the
/// smaller end cannot support the division.
fn condition_ratio(largest: f64, smallest: f64) -> f64 {
    if smallest > 0.0 { largest / smallest } else { f64::INFINITY }
}

/// The squared Frobenius norm: the sum of the squares of every entry.
fn frobenius_squared(m: &faer::Mat<f64>) -> f64 {
    let mut total = 0.0;
    for j in 0..m.ncols() {
        for i in 0..m.nrows() {
            total = m[(i, j)].mul_add(m[(i, j)], total);
        }
    }
    total
}

/// The error this marginalisation committed: measured where the exact term is
/// numerically reachable, bounded where it is not.
///
/// The quantity is tr(E_δ) for `E_δ = S_δ − S_0`, which the algorithm gives as
/// `δ B_kr B_rr⁻¹ (B_rr + δI)⁻¹ B_rk ⪰ 0` (´alg:gaussian:regularised-schur´).
/// Two facts make it cheap. Its trace is the difference of the traces of the
/// exact and the applied corrections, and each of those is the squared
/// Frobenius norm of a half-solved factor, so no k × k matrix is formed to
/// read either. And because `E_δ` is positive semi-definite, its trace also
/// dominates its spectral norm, so the single figure serves the norm the
/// bound is stated in.
///
/// `applied_correction_trace` is `None` where the correction was discarded.
/// That is the offset's own endpoint — as δ grows without bound the correction
/// vanishes and the result approaches B_kk — so the fallback needs no separate
/// accounting: it is this expression with the whole correction as the loss.
///
/// The exact term is refused where the solve that would produce it has less
/// resolution than the difference it is taken across, which is the
/// ill-conditioned regime and precisely where `B_rr⁻¹` is untrustworthy. The
/// bound then stands in, and the returned variant says which happened.
fn marginalisation_error(
    b_rr: &SymmetricMatrix,
    b_rk: &faer::Mat<f64>,
    delta: f64,
    spectrum: Option<(f64, f64)>,
    applied_correction_trace: Option<f64>,
) -> MarginalisationError {
    // Without a least eigenvalue the bound has no denominator. The error is
    // no less real; there is simply no finite figure to put on it.
    let Some((lambda_max, lambda_min)) = spectrum else {
        return MarginalisationError::Unbounded;
    };
    if lambda_min <= 0.0 || !lambda_min.is_finite() {
        return MarginalisationError::Unbounded;
    }

    let applied = applied_correction_trace.unwrap_or(0.0);

    // The share of the correction the offset is expected to remove, which is
    // the size of the difference the exact computation would have to resolve.
    // Where the correction was discarded the whole of it is the difference.
    let resolved_share = if applied_correction_trace.is_some() {
        delta / (lambda_min + delta)
    } else {
        1.0
    };
    let solve_resolution = condition_ratio(lambda_max, lambda_min) * f64::EPSILON;

    let exact_factor = if solve_resolution * EXACT_ERROR_HEADROOM < resolved_share {
        cholesky(b_rr).ok()
    } else {
        None
    };
    if let Some(exact_llt) = exact_factor {
        let exact_w = schur_half_solve(&exact_llt, b_rk);
        let exact_correction_trace = frobenius_squared(&exact_w);
        // E_δ ⪰ 0 holds in exact arithmetic, so a difference below zero is
        // cancellation under the resolution rather than a negative error.
        let mass = (exact_correction_trace - applied).max(0.0);
        let fraction = if exact_correction_trace > 0.0 {
            (mass / exact_correction_trace).clamp(0.0, 1.0)
        } else {
            0.0
        };
        return MarginalisationError::Measured {
            discarded_precision_mass: mass,
            correction_loss_fraction: fraction,
        };
    }

    // The bound is the audit's, stated for the trace: from
    // tr(Cᵀ M C) ≤ ‖M‖₂ ‖C‖_F² with M = δ B_rr⁻¹(B_rr + δI)⁻¹, whose largest
    // eigenvalue is δ/[λ_min(λ_min + δ)]. Discarding the correction is the
    // same expression at the offset's endpoint, where the multiplier is
    // 1/λ_min.
    let attenuation = if applied_correction_trace.is_some() {
        delta / (lambda_min * (lambda_min + delta))
    } else {
        1.0 / lambda_min
    };
    let mass = attenuation * frobenius_squared(b_rk);
    if !mass.is_finite() {
        return MarginalisationError::Unbounded;
    }
    // tr(C_0) = tr(C_δ) + tr(E_δ), so an upper bound on the second gives an
    // upper bound on the share, the ratio being increasing in it.
    let denominator = applied + mass;
    let fraction = if denominator > 0.0 {
        (mass / denominator).clamp(0.0, 1.0)
    } else {
        0.0
    };
    MarginalisationError::Bounded {
        discarded_precision_mass: mass,
        correction_loss_fraction: fraction,
    }
}

/// The exact marginalisation error of a rank-one removal, in closed form.
///
/// At r = 1 the correction is an outer product and the audit's inequality is
/// an equality, so both figures are measurements rather than bounds: the
/// offset removes exactly δ/(B₁₁ + δ) of a correction whose trace is ‖C‖²/B₁₁
/// (´alg:gaussian:regularised-schur´). `exact_correction_trace` is `None`
/// where the removed scalar is not a usable precision, in which case there is
/// no finite figure to report. `applied` says whether the correction survived;
/// where it did not, the whole of it is the error.
#[must_use]
pub fn rank1_marginalisation_error(
    exact_correction_trace: Option<f64>,
    delta: f64,
    denominator: f64,
    applied: bool,
) -> MarginalisationError {
    let Some(trace) = exact_correction_trace else {
        return MarginalisationError::Unbounded;
    };
    if applied {
        MarginalisationError::Measured {
            discarded_precision_mass: trace * delta / denominator,
            correction_loss_fraction: delta / denominator,
        }
    } else {
        MarginalisationError::Measured {
            discarded_precision_mass: trace,
            correction_loss_fraction: if trace > 0.0 { 1.0 } else { 0.0 },
        }
    }
}

/// The error of discarding a correction outright, for a caller that took one
/// from this module and then refused it downstream.
///
/// This is the offset's endpoint case of the figure the algorithm already
/// reports (´alg:gaussian:regularised-schur´): a correction that is not
/// adopted is a correction lost whole. It re-reads the removed block's
/// spectrum and coupling from the parent precision rather than carrying them
/// out of the algorithm, so it costs what that algorithm's first stages cost
/// and belongs on escape hatches rather than on a common path.
#[must_use]
pub fn discarded_correction_error(
    precision: &SymmetricMatrix,
    remove_indices: &[usize],
    keep_indices: &[usize],
    config: &SchurConfig,
    lambda_prior: f64,
) -> MarginalisationError {
    if remove_indices.is_empty() {
        return MarginalisationError::NONE;
    }
    let b_rr = precision.extract_symmetric_submatrix(remove_indices);
    let b_rk = precision
        .extract_cross_block(keep_indices, remove_indices)
        .transpose()
        .to_owned();
    let spectrum = removed_block_spectrum(&b_rr);
    marginalisation_error(&b_rr, &b_rk, config.delta(lambda_prior), spectrum, None)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Verification
// ═══════════════════════════════════════════════════════════════════════════════

/// What a refused factorisation carries into the log.
///
/// A factorisation walks the block in order, so the pivot index names the
/// leading principal submatrix that first failed to be positive definite —
/// a statement about B' and not about the arithmetic that met it — and the
/// floor says how far from zero the boundary it failed against stood.
#[derive(Debug, Clone, Copy)]
pub struct FactorisationRefusal {
    /// The position at which the factorisation met a non-positive pivot.
    pub pivot_index: usize,
    /// The resolution floor B' was lowered by before the attempt.
    pub floor: f64,
}

/// Whether B' passes the positive-definiteness verification, together with the
/// certificate the verdict was read from and, on refusal, what refused it.
///
/// The requirement asks a yes or a no (´req:gaussian:positive-definiteness´),
/// and a Cholesky factorisation answers exactly that question: it completes if
/// and only if the matrix put to it is positive definite. Reading the diagonal
/// instead would be reading a necessary condition as though it were a
/// sufficient one, and this is the position in which the difference tells: the
/// Schur correction is subtracted from B_kk's diagonal and added to its
/// off-diagonal structure alike, so a B' can emerge with every diagonal entry
/// comfortably positive and a negative eigenvalue carried between them.
///
/// # What is factorised, and why it is the same test
///
/// The boundary is a floor relative to B''s own scale
/// (´const:assayer:schur-verification-resolution-scalar-8p0´), so what is put
/// to the factorisation is `B' − floor·I`, which factorises in exact
/// arithmetic if and only if `λ_min(B') > floor`. The floor is taken at `U`,
/// the outward-rounded Gershgorin upper bound, for which `U ≥ λ_max(B')` holds
/// as a theorem about the binary64 values rather than as an approximation that
/// usually holds. The floor is therefore at or above the floor the least
/// eigenvalue would have been judged against: every B' a test on the spectrum
/// refuses is refused here too, and the refusals the two do not share lie in
/// the band between the two floors, where the least eigenvalue is within a few
/// multiples of the width-scaled epsilon of zero. Refusing there is the
/// direction the verification takes everywhere else, because the fallback
/// costs the correction while an adoption would carry forward a precision
/// matrix whose inverse — the covariance the marginalisation goes on to
/// restore — is meaningless.
///
/// The factorisation runs in the same finite arithmetic, and its pivots carry
/// a backward error of order `p · ε_mach · ‖B'‖`. The floor's margin is
/// `8 · p · ε_mach · U`, which is the same width term at the same scale, since
/// `U` is at or above the spectral norm: the boundary stands a factor of eight
/// above the error the pivots carry. A factorisation that completes therefore
/// certifies `λ_min(B') > 0`, rather than certifying that the arithmetic
/// failed to notice otherwise.
///
/// A B' carrying a non-finite entry bounds nothing, and refuses for the reason
/// the disc pass declines to certify it: there is no verdict to adopt B' on,
/// and the fallback is the safe reading.
pub fn verify_positive_definite(b_prime: &SymmetricMatrix) -> (bool, VerificationCertificate, Option<FactorisationRefusal>) {
    // A B' of no dimensions is vacuously positive definite — the requirement
    // quantifies over an empty spectrum — and marginalising every dimension
    // away reaches here. The rank-1 path admits the same case explicitly
    // rather than putting an empty matrix to a factorisation.
    if b_prime.dim() == 0 {
        return (true, VerificationCertificate::Vacuous, None);
    }

    let Some(floor) = verification_floor(b_prime) else {
        tracing::warn!("Schur B' carries a non-finite entry; refusing the correction");
        return (false, VerificationCertificate::Factorisation, None);
    };

    match cholesky(&b_prime.add_scaled_identity(-floor)) {
        Ok(_) => (true, VerificationCertificate::Factorisation, None),
        Err(LltError::NonPositivePivot { index }) => (
            false,
            VerificationCertificate::Factorisation,
            Some(FactorisationRefusal {
                pivot_index: index,
                floor,
            }),
        ),
    }
}

/// The floor the verification lowers B' by before factorising it, absent where
/// B' carries a non-finite entry and nothing bounds its spectrum.
///
/// One O(p²) pass supplies it, and it is the pass that also answers the disc
/// verdict the rank-1 path reads, so the two boundaries are one computation
/// stated once rather than two that agree by inspection.
pub fn verification_floor(b_prime: &SymmetricMatrix) -> Option<f64> {
    gershgorin_bounds(b_prime).map(|bounds| resolution_floor(b_prime.dim(), bounds.upper))
}

/// The resolution floor at a given upper bound on the spectrum
/// (´const:assayer:schur-verification-resolution-scalar-8p0´).
///
/// The factor is exact — `8 · width` is a small integer and ε is a power of
/// two, so both products scale rather than round — and only the last
/// multiplication needs directing, upward, so that a value cleared against
/// this floor is cleared against one at or above what the exact bound would
/// have given. Where the bound is not itself positive the floor collapses to
/// zero and the test becomes the bare `λ_min > 0`, which such a B' fails on
/// its own since `λ_min ≤ λ_max ≤ 0`.
fn resolution_floor(width: usize, upper: f64) -> f64 {
    #[allow(clippy::cast_precision_loss)]
    let width = width as f64;
    let scale = VERIFICATION_RESOLUTION_FACTOR * width * f64::EPSILON;
    (scale * upper.max(0.0)).next_up()
}

/// What one outward-rounded Gershgorin pass over B' was able to settle.
enum GershgorinVerdict {
    /// Every disc lies strictly right of the resolution floor, so the result
    /// is positive definite at the boundary the shared verifier uses.
    Certified,
    /// A diagonal entry is not strictly positive, so the result is not
    /// positive definite and no spectrum can rescue it.
    Refuted,
    /// The discs reach across the floor. Gershgorin is sufficient and not
    /// necessary, so this settles nothing either way.
    Inconclusive,
}

/// What one outward-rounded Gershgorin pass over B' returns.
#[derive(Debug, Clone, Copy)]
struct GershgorinBounds {
    /// A binary64 value at or below `λ_min(B')`.
    lower: f64,
    /// A binary64 value at or above `λ_max(B')`, which is what the resolution
    /// floor is taken at.
    upper: f64,
    /// The least diagonal entry of B'. A diagonal entry is the Rayleigh
    /// quotient at a coordinate vector, so this is at or above `λ_min(B')`,
    /// and its sign is evidence in the refusing direction alone.
    min_diagonal: f64,
}

/// One outward-rounded Gershgorin pass over B'.
///
/// Gershgorin places every eigenvalue of a symmetric matrix inside
/// `[min_i(s_ii − r_i), max_i(s_ii + r_i)]` for row sums
/// `r_i = Σ_{j≠i} |s_ij|`. That is a statement about the matrix actually
/// computed, which is what the rank-1 path needs: the entrywise-rounded
/// correction it subtracts is neither rank one nor positive semi-definite, so
/// the exact-arithmetic theorem behind the old diagonal check says nothing
/// about the result the path is about to adopt
/// (´req:gaussian:rank-one-verification´).
///
/// # The rounding discipline
///
/// A bound computed in round-to-nearest is not a bound. Every rounding here is
/// therefore directed away from acceptance, so that the numbers this returns
/// bracket the true ones rather than approximating them:
///
/// - each row sum accumulates as `(acc + |s_ij|).next_up()`, and since `fl`
///   and `next_up` are both monotone, induction carries `radius ≥ Σ|s_ij|`
///   through the whole accumulation;
/// - the lower bound is `(s_ii − radius).next_down()`, at or below the real
///   number `s_ii − radius`, which is itself at or below the exact disc edge
///   because `radius` is at or above the exact row sum;
/// - the upper bound is `(s_ii + radius).next_up()`, at or above the exact
///   disc edge by the same argument in the other direction.
///
/// So `lower ≤ λ_min(B')` and `upper ≥ λ_max(B')` hold as theorems about the
/// binary64 values returned, not as approximations that usually hold. An
/// ordinary under-rounded row sum would certify nothing at all, which is the
/// whole reason the directed steps are here.
///
/// A non-finite entry anywhere returns no bounds rather than propagating a
/// comparison whose result is false for the wrong reason. Withholding them is
/// the safe direction: neither reader can then certify, and the general path's
/// factorisation witness refuses outright.
fn gershgorin_bounds(b_prime: &SymmetricMatrix) -> Option<GershgorinBounds> {
    let p = b_prime.dim();
    let inner = b_prime.as_inner();

    let mut lower = f64::INFINITY;
    let mut upper = f64::NEG_INFINITY;
    let mut min_diagonal = f64::INFINITY;

    for i in 0..p {
        let mut radius = 0.0_f64;
        for j in 0..p {
            if j != i {
                radius = (radius + inner[(i, j)].abs()).next_up();
            }
        }
        let diagonal = inner[(i, i)];

        // Guarding finiteness here rather than at the comparisons is what
        // keeps a NaN from being swallowed: `f64::min` and `f64::max` return
        // the other operand when one is NaN, so a NaN reaching the running
        // extremes would vanish and leave a bound that certifies a matrix
        // nobody bounded. Every entry of B' is visited either as a diagonal or
        // as an off-diagonal of some row, so these two tests cover the matrix.
        if !diagonal.is_finite() || !radius.is_finite() {
            return None;
        }

        min_diagonal = min_diagonal.min(diagonal);
        lower = lower.min((diagonal - radius).next_down());
        upper = upper.max((diagonal + radius).next_up());
    }

    Some(GershgorinBounds {
        lower,
        upper,
        min_diagonal,
    })
}

/// What the discs settle about B', at the boundary the shared verification
/// applies (´const:assayer:schur-verification-resolution-scalar-8p0´).
fn gershgorin_verdict(b_prime: &SymmetricMatrix) -> GershgorinVerdict {
    let Some(bounds) = gershgorin_bounds(b_prime) else {
        return GershgorinVerdict::Inconclusive;
    };

    let floor = resolution_floor(b_prime.dim(), bounds.upper);

    if bounds.lower > floor {
        // λ_min ≥ lower > floor ≥ the floor at λ_max, since the floor rises
        // with its argument and upper ≥ λ_max. So the factorisation witness
        // would have accepted, and this accepts on its behalf.
        return GershgorinVerdict::Certified;
    }

    if bounds.min_diagonal <= 0.0 {
        // A diagonal entry is the Rayleigh quotient at a coordinate vector, so
        // λ_min ≤ min_i s_ii ≤ 0, and the floor is never negative. The
        // factorisation would refuse, and running it to learn that would pay a
        // cubic price for a fact a sign already settled. This is the one
        // direction in which the diagonal remains evidence.
        return GershgorinVerdict::Refuted;
    }

    GershgorinVerdict::Inconclusive
}

/// Whether B' passes the rank-1 path's positive-definiteness verification,
/// together with the certificate the verdict was read from.
///
/// The contract is the general path's contract and the boundary is the general
/// path's boundary (´req:gaussian:rank-one-verification´). What differs is the
/// order in which the evidence is sought: the cheap certificate first, because
/// the shared one costs a Cholesky factorisation of the whole block and would
/// move the frequent path out of the cost class the corpus prices it in
/// (´tab:gaussian:operation-costs´). Nothing is adopted on evidence the shared
/// contract would not adopt it on; the arrangement buys time, not latitude.
pub fn verify_rank1_positive_definite(b_prime: &SymmetricMatrix) -> (bool, VerificationCertificate) {
    if b_prime.dim() == 0 {
        return (true, VerificationCertificate::Vacuous);
    }

    match gershgorin_verdict(b_prime) {
        GershgorinVerdict::Certified => (true, VerificationCertificate::Gershgorin),
        GershgorinVerdict::Refuted => (false, VerificationCertificate::DiagonalDisproof),
        GershgorinVerdict::Inconclusive => {
            let (passed, certificate, _refusal) = verify_positive_definite(b_prime);
            (passed, certificate)
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Keep-Indices Utility
// ═══════════════════════════════════════════════════════════════════════════════

/// Computes the sorted complement of `remove` in `0..p`.
///
/// Precondition: `remove` is sorted ascending with all elements
/// in `0..p` and no duplicates.
#[must_use]
pub fn compute_keep_indices(p: usize, remove: &[usize]) -> Vec<usize> {
    let mut keep = Vec::with_capacity(p.saturating_sub(remove.len()));
    let mut ri = 0;
    for i in 0..p {
        if ri < remove.len() && remove[ri] == i {
            ri += 1;
        } else {
            keep.push(i);
        }
    }
    debug_assert_eq!(keep.len(), p - remove.len());
    keep
}
