// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`new_starts_at_zero_mean_under_the_isotropic_prior`] | bayes | A newly constructed model believes nothing in particular: the mean sits at the origin and both matrices are multiples of the identity, the precision at the requested prior and the covariance at its reciprocal. Every coordinate therefore starts equally uncertain and equally uncommitted, which is what lets a model be created before anyone knows which features will matter. |
//! | [`new_p1`] | bayes | cites (´claim:bayes:a-fresh-model-starts-at-a-zero-mean-under-an-isotropic-prior-set-by-the-precision-argument´) |
//! | [`new_prior_precision_scales`] | bayes | cites (´claim:bayes:a-fresh-model-starts-at-a-zero-mean-under-an-isotropic-prior-set-by-the-precision-argument´) |
//! | [`extend_raises_the_dimension_and_starts_new_coordinates_at_the_prior`] | bayes | A model can grow: extension raises the dimension by the requested amount and gives the arriving coordinates the same prior shape a fresh model would have given them. New features can therefore be admitted mid-life without rebuilding the model and throwing away everything it has learned. |
//! | [`extend_zero`] | bayes | Extending by nothing costs nothing and changes nothing — the dimension is exactly what it was. A caller computing how many features to add need not guard the zero case before calling. |
//! | [`extend_preserves_existing`] | bayes | What the model already knew survives extension untouched, even when the arriving block is given a different prior precision from the one the original coordinates were built with. Admitting a new feature is an addition, never a partial reset of the posterior already earned. |
//! | [`marginalise_last_two`] | bayes | Marginalisation returns a genuinely smaller model — the removed coordinates are gone from the dimension — and what remains is still a usable posterior, with strictly positive precision and covariance on every surviving diagonal. A shrinking model must stay invertible, or the very next update would have nothing well-defined to work from. |
//! | [`marginalise_followed_by_roundtrip`] | bayes | cites (´claim:bayes:a-snapshot-round-trip-restores-the-mean-and-covariance-and-rebuilds-the-precision-from-them´) |
//! | [`marginalise_unsorted_panics`] | bayes | Removal indices must arrive sorted ascending, and a caller who passes them the other way round is told so outright in a debug build rather than quietly receiving a model whose blocks were gathered in the wrong order. Silent index scrambling would corrupt the posterior in a way no later assertion could attribute back to its cause. |
//! | [`to_from_parameters_roundtrip`] | bayes | A snapshot is a faithful description of the posterior: restoring from one gives back the same mean and the same covariance, and the precision is rebuilt from the covariance rather than carried, coming back as its inverse. The stored form can therefore be smaller than the live model without the restored model being any less usable. |
//! | [`from_parameters_resets_recompute_counter`] | bayes | A restored model owes no recomputation: its label counter starts at zero because the precision it holds was just derived from the covariance by a fresh factorisation. Carrying the pre-snapshot count across would schedule a repair for drift that no longer exists. |
//! | [`new_dual_tracking`] | bayes | The model carries precision and covariance side by side as a mutually inverse pair, and at construction their product is the identity to machine precision. Both are needed on the hot path — the covariance for leverage, the precision for the update — and keeping them in step is what makes it safe to read either one without factorising first. |
//! | [`extend_dual_tracking`] | bayes | cites (´claim:bayes:precision-and-covariance-are-maintained-as-a-mutually-inverse-pair´) |
//! | [`marginalise_dual_tracking`] | bayes | cites (´claim:bayes:precision-and-covariance-are-maintained-as-a-mutually-inverse-pair´) |
//! | [`extend_cross_blocks_zero`] | bayes | Coordinates that arrive by extension arrive independent: every entry linking an old dimension to a new one is exactly zero in both the precision and the covariance. A newly admitted feature starts with no borrowed correlation, so whatever relationship it eventually shows must have been learned from observations rather than inherited from construction. |
//! | [`marginalise_resets_counter`] | bayes | cites (´claim:bayes:a-multi-dimension-marginalisation-restarts-the-recompute-cadence´) |
//! | [`marginalise_empty_indices`] | bayes | cites (´claim:bayes:removing-no-dimensions-leaves-the-precision-exactly-as-it-was´) |
//! | [`marginalise_all_but_one`] | bayes | cites (´claim:bayes:marginalisation-yields-a-smaller-model-that-is-still-positive-on-every-surviving-diagonal´) |
//! | [`to_parameters_excludes_b`] | bayes | A snapshot carries the covariance and not the precision: the stored matrix holds the reciprocal of the prior on its diagonal, not the prior itself. Storing only one of the pair halves the persisted size and removes any possibility of loading a snapshot whose two halves disagree. |
//! | [`from_parameters_with_bad_sigma`] | bayes | A snapshot whose covariance cannot be factorised — here one carrying a non-numeric diagonal — is refused, both the plain and the regularised factorisation having failed. Loading it anyway would produce a model whose precision is meaningless, and the failure would surface far from the corrupt input that caused it. |
//! | [`new_l024_defaults`] | bayes | A fresh model starts its numerical-health bookkeeping from a clean slate: the effective recomputation interval is the configured one rather than a halved value, and no clean streak, conditioning estimate, or sync error is yet on record. Adaptive cadence has to begin optimistic — a model with no history has given no reason to distrust it. |
//! | [`clean_high_sync_error_updates_model_cadence`] | bayes | The live model applies the clean-high-error rule rather than merely exposing it through a helper: an error above the width-scaled threshold halves its interval, clears recovery, records the error and advances the shortening count. |
//! | [`effective_weight_is_the_smallest_of_target_ceiling_and_leverage_bound`] | bayes | The weight an observation actually receives is the smallest of three limits — the weight asked for, the global importance ceiling, and the leverage bound derived from how far the feature sticks out under the current covariance — and each of the three is shown taking its turn as the binding one. Any one of them alone would be insufficient: the ceiling caps runaway importance, the leverage bound caps updates that a single observation could otherwise dominate. |
//! | [`leverage_bound_fires_at_cold_start`] | bayes | At the cold-start prior the covariance is wide, so almost any feature has large leverage and the leverage limit — not the requested importance — is what decides the weight: across a hundred varied feature vectors it binds on essentially all of them. This is the regime the bound exists for, since an early observation admitted at full weight would move a model that has nothing yet to hold it steady. |
//! | [`labels_since_recompute_incremented`] | bayes | Every posterior update advances by exactly one the count of labels absorbed since the last exact factorisation. That count is the model's own measure of how much incremental drift it may have taken on, and it is what the recomputation schedule reads to decide when to pay for a fresh inversion. |
//! | [`mean_update_ordering`] | bayes | The residual is taken against the mean as it stood before the update, so a positive target pulls the mean in the positive direction. Were the mean mutated first and the residual measured afterwards, the observation would partly explain away its own surprise and the model would under-react to exactly the evidence it most needs to absorb. |
//! | [`sigma_phi_identity`] | bayes | The rank-one update leaves an exact algebraic trace: applying the updated covariance to the very feature just observed reproduces that feature's pre-update image, shrunk by the forgetting factor plus the weighted leverage. This is the identity the incremental formula is derived from, so it holding element-wise is what distinguishes a correct update from one that merely moves the matrix in a plausible direction. |
//! | [`dual_tracking_after_update`] | bayes | cites (´claim:bayes:precision-and-covariance-are-maintained-as-a-mutually-inverse-pair´) |
//! | [`zero_effective_weight`] | bayes | An observation admitted at zero weight teaches the model nothing — the mean does not move at all — yet time still passes for it: the precision is decayed by the forgetting factor and clamped at its floor exactly as on any other update. Forgetting is a property of the step, not of the evidence, so a rejected observation must not also freeze the model in place. |
//! | [`q1_model_moves_from_prior`] | bayes | A couple of hundred labels of mixed valence are enough to carry the mean a measurable distance from the origin it started at. The leverage bound and the forgetting factor both hold updates back, and this pins that they hold it back rather than pinning it: a model that never leaves its prior would pass every stability check while learning nothing. |
//! | [`n1_window_safety`] | bayes | The incremental path can be run past a full recomputation window — 1,400 labels with no exact refactorisation at all — and still hold together: every value stays finite, the two matrices are still each other's inverse on the diagonal, and the covariance still returns non-negative quadratic forms. The recomputation cadence exists to bound drift, and this fixes that the model degrades gracefully rather than falling apart the moment the cadence is exceeded. |
//! | [`combined_decay_equivalence`] | bayes | Forgetting by the clock and forgetting per label compose: over a hundred labels, folding both into one combined factor gives the same mean, precision, and covariance as decaying for elapsed time first and then applying the per-label factor through the update. Because they compose, the implementation may collapse two decays into one multiplication on the hot path instead of touching the matrices twice per observation. |
//! | [`floor_drift_bounded`] | bayes | The precision floor is a clamp, and a clamp breaks the exact inverse relationship the covariance update assumes. Under a floor chosen to fire on every dimension of every one of two hundred labels, the resulting mismatch stays bounded rather than compounding, and the model stays finite with positive diagonals throughout. The floor is what keeps a decayed precision from collapsing toward zero, so its cost has to be a small standing error and not a divergence. |
//! | [`dimension_1_scalar`] | bayes | At one dimension the matrix update collapses to a scalar recursion that can be written out by hand, and the implementation reproduces every step of it — decayed precision, floor, rank-one addition, covariance shrinkage, mean shift — to within rounding, with the two scalars still reciprocal afterwards. The scalar case is the one place the whole formula can be checked against arithmetic rather than against another implementation of itself. |
//! | [`anchor_p15_dual_tracking`] | bayes | The small dimension the anchor models actually run at is not a special case: after fifty updates the precision and covariance are still inverses to the full matrix tolerance, and the covariance-feature identity still holds exactly on a further update. Small models take the same code path as large ones, and the invariants that were fixed at larger dimensions are shown to survive where the arithmetic has fewest terms to average over. |
//! | [`a_negative_leverage_refuses_the_update_and_leaves_the_model_where_it_stood`] | bayes | A leverage read against a covariance that has lost positive semi-definiteness is negative, and the bound divides by it, so the weight it produces is negative rather than small — the harm a leverage bound exists to prevent, arriving through the bound itself. The update refuses the reading in every build profile and mutates nothing: the mean, the covariance and the precision all stand where they stood, and the label counter has not advanced. Applying at a negative weight would subtract information the model never received, carrying the precision matrix further from the property whose loss the reading was reporting. |
//! | [`golden_vectors`] | bayes | Across three fixed configurations — differing in prior, weight, target sign, forgetting factor, and floor — the update lands on the values the explicit formulas predict, in the mean, in the precision diagonal, and in the covariance-feature identity, with the inverse pairing intact. Fixing known inputs to known outputs is what catches a refactor that changes the answer while leaving every self-consistency property satisfied. |
//! | [`a_drifted_covariance_is_repaired_and_the_rebuild_adopted`] | bayes | A precision matrix carried a long way by cheap rank-one updates has a maintained covariance that has drifted from its inverse, and the rebuild both repairs that and is adopted for it: the drift measured after the rebuild is more than an order of magnitude below the drift measured before it — a factor near twenty-five at this width and this many labels — and both readings sit under the width-scaled threshold, so the covariance the rebuild offered is the one the model ends up holding. Adoption on the measurement rather than on the factorisation's verdict is what makes the test mean anything, so the case where the rebuild genuinely helps has to pass it. |
//! | [`the_rebuild_applies_the_floor_and_leaves_the_pair_consistent`] | bayes | The floor is applied where it is measured. A rebuild on a matrix whose least eigenvalue has fallen below the floor installs a precision matrix whose least eigenvalue is exactly the floor, together with the covariance taken from that same matrix — so the pair is consistent and the monitor's reading after the rebuild is machine-level rather than the size of the floor. Between rebuilds nothing is added: the least eigenvalue decays with everything else, by a positive factor that preserves definiteness exactly and by no more than the decay the interval allows, which the next rebuild's own measurement absorbs. The alternative this test exists to exclude is an increment applied per label to the precision matrix alone. That is arithmetically identical at the precision matrix and catastrophic at the monitor, because the covariance does not follow it: the pair then disagrees by the increment times the covariance, which at the floor's magnitude is of order one per label, and the cadence reads its own floor as drift. |
//! | [`the_prior_masses_follow_the_precision_matrix`] | bayes | The two shares of the precision matrix the prior is holding up are tracked rather than inferred, and each behaves like the part of the matrix it describes. The identity-shaped share takes the deficit a rebuild adds and then decays with the matrix it is part of, reaching the closed form the decay implies over the labels that follow, because it is scaled by the same factor the matrix is scaled by on every one of them. The coordinate-shaped share rides alongside the matrix through a structural change: marginalising a coordinate takes that coordinate's accumulated clamp with it and leaves the survivors' in the order the survivors are now in. A floor whose size is known and whose contribution is not would be only half a declaration, so both have to be carried rather than recomputed from a matrix that no longer distinguishes prior from evidence. |
//! | [`the_clamp_mass_predicts_its_own_contribution_to_the_reading`] | bayes | The synchronisation reading a model with exhausted coordinates carries has a component the prior put there, and its size is the coordinate-shaped clamp mass scaled by the covariance — exactly, not approximately. The replenishment clamp raises a diagonal entry every label and the Sherman–Morrison covariance does not follow it, so the tracked pair disagrees by the clamp's accumulated contribution times the covariance, which is what this test computes both ways and compares. The consequence is the whole reason to record it. The monitor reads that component as drift, and the cadence shortens the interval in response to a quantity no recomputation can reduce below one interval's worth of it — which is why a model with coordinates at the floor sits at the cadence floor and reads a standing figure that scales with the interval rather than with the width. It is conservative in direction, since the covariance is wider than the inverse exactly along the directions the model has no evidence for, so it is not a fault; it is a prior the reading cannot tell apart from rounding. |
//! | [`the_borrowed_share_resolves_the_two_masses_to_a_direction`] | bayes | The two masses resolve to a share along a direction, and the share is a division rather than an inference. A model whose floors have never acted reports no share at all along any direction, so nothing downstream of the reading can move on a healthy model. Once the clamp has bound, the share along a coordinate is that coordinate's own clamp mass over that coordinate's own posterior precision, which is a number a reader can work out by hand from the two published quantities — here a fifth along the coordinate an observation has since sharpened, and a half along the three it has not. The direction is what makes the reading useful and what makes it partial. An aggregate degradation figure says a model is propped up somewhere; this says whether it is propped up *here*, along the features of the request in hand, which is the only form in which the fact can be attached to a verdict. What it does not say is anything about the subspace the direction lies in, because a Rayleigh quotient reads one direction and reports one number. |
//! | [`the_repaired_count_counts_the_floors_repairs`] | bayes | The repaired count counts the floor's repairs and nothing else: a rebuild whose least eigenvalue was short of the floor increments it, and a rebuild on a matrix the evidence already holds above the floor leaves it where it was. The counter is the one the retired repair cascade used to fill, and its meaning is unchanged — this model's precision needed help — because the help it now records is the floor's rather than a factorisation shift's. A counter left on the retired device could hold only zero, which is a constant reported where a measurement is promised, so the move is what keeps the reading honest rather than merely keeping a field alive. |

//! Crate-level tests for `model::bayesian::BayesianLinearModel`.
//!
//! Dual-tracking Frobenius bounds and finite-value checks use the
//! shared helpers [`assert_below`](crate::testing::assert_below) and
//! [`assert_finite`](crate::testing::assert_finite) from
//! [`crate::testing`]. Numerical tolerances remain encoded per-test
//! rather than pulled from [`DEFAULT_TOLERANCES`](crate::testing::DEFAULT_TOLERANCES)
//! because these unit tests bound the tight implementation invariants
//! (1e-15 at construction, 1e-10 after updates), not the looser CI
//! policy bounds the shared tolerances describe.

#![allow(clippy::cast_precision_loss, clippy::suboptimal_flops)]

use crate::linalg::symmetric::SymmetricMatrix;
use crate::model::bayesian::BayesianLinearModel;
use crate::model::marginalise::SchurConfig;
use crate::model::parameters::{FloorMass, ModelParameters};
use crate::model::recompute::{RecomputeOutcome, synchronisation_error_threshold};
use crate::testing::{assert_below, assert_finite};
use crate::types::ModelId;

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Frobenius norm of (M * N - I).
fn frobenius_deviation_from_identity(m: &SymmetricMatrix, n: &SymmetricMatrix) -> f64 {
    let p = m.dim();
    assert_eq!(p, n.dim());
    let product = m.as_inner() * n.as_inner();
    let mut sum_sq = 0.0;
    for j in 0..p {
        for i in 0..p {
            let expected = if i == j { 1.0 } else { 0.0 };
            let diff = product[(i, j)] - expected;
            sum_sq = diff.mul_add(diff, sum_sq);
        }
    }
    sum_sq.sqrt()
}

/// Check that all diagonal elements of a `SymmetricMatrix` are positive.
fn assert_positive_diagonal(m: &SymmetricMatrix, label: &str) {
    for i in 0..m.dim() {
        assert!(
            m.diagonal_element(i) > 0.0,
            "{label}: diagonal[{i}] = {} (expected positive)",
            m.diagonal_element(i),
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Construction
// ─────────────────────────────────────────────────────────────────────────────

/// A newly constructed model believes nothing in particular: the mean sits at
/// the origin and both matrices are multiples of the identity, the precision at
/// the requested prior and the covariance at its reciprocal. Every coordinate
/// therefore starts equally uncertain and equally uncommitted, which is what
/// lets a model be created before anyone knows which features will matter.
///
/// ´claim:bayes:a-fresh-model-starts-at-a-zero-mean-under-an-isotropic-prior-set-by-the-precision-argument´
/// ´test:crate:new-starts-at-zero-mean-under-the-isotropic-prior´
#[test]
fn new_starts_at_zero_mean_under_the_isotropic_prior() {
    let m = BayesianLinearModel::new(100, 0.1, 1000);
    assert_eq!(m.dim(), 100);

    // μ = 0
    for i in 0..100 {
        assert_eq!(m.mu()[i], 0.0, "mu[{i}] should be 0");
    }

    // B = 0.1 * I
    for i in 0..100 {
        assert!(
            (m.precision().diagonal_element(i) - 0.1).abs() < 1e-15,
            "B[{i},{i}] = {} expected 0.1",
            m.precision().diagonal_element(i),
        );
    }

    // Σ = 10 * I
    for i in 0..100 {
        assert!(
            (m.covariance().diagonal_element(i) - 10.0).abs() < 1e-15,
            "Σ[{i},{i}] = {} expected 10.0",
            m.covariance().diagonal_element(i),
        );
    }
}

/// The single-dimension model is built by the same rule as any other, with no
/// degenerate special case at p=1: one coordinate, a zero mean, and a
/// precision and covariance that are reciprocal scalars. The scalar case is
/// where the anchor models live, so it must not be an afterthought.
///
/// (´claim:bayes:a-fresh-model-starts-at-a-zero-mean-under-an-isotropic-prior-set-by-the-precision-argument´)
/// ´test:crate:new-p1´
#[test]
fn new_p1() {
    let m = BayesianLinearModel::new(1, 1.0, 1000);
    assert_eq!(m.dim(), 1);
    assert_eq!(m.mu()[0], 0.0);
    assert!((m.precision().diagonal_element(0) - 1.0).abs() < 1e-15);
    assert!((m.covariance().diagonal_element(0) - 1.0).abs() < 1e-15);
}

/// The prior precision is a dial, not a constant: raising it raises the
/// starting precision and lowers the starting covariance by exactly the same
/// reciprocal factor. A caller who wants a model that resists early evidence
/// asks for it with one number rather than by editing matrices afterwards.
///
/// (´claim:bayes:a-fresh-model-starts-at-a-zero-mean-under-an-isotropic-prior-set-by-the-precision-argument´)
/// ´test:crate:new-prior-precision-scales´
#[test]
fn new_prior_precision_scales() {
    let m = BayesianLinearModel::new(5, 2.0, 1000);
    for i in 0..5 {
        assert!((m.precision().diagonal_element(i) - 2.0).abs() < 1e-15);
        assert!((m.covariance().diagonal_element(i) - 0.5).abs() < 1e-15);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Extension
// ─────────────────────────────────────────────────────────────────────────────

/// A model can grow: extension raises the dimension by the requested amount
/// and gives the arriving coordinates the same prior shape a fresh model would
/// have given them. New features can therefore be admitted mid-life without
/// rebuilding the model and throwing away everything it has learned.
///
/// ´claim:bayes:extension-raises-the-dimension-and-starts-the-new-coordinates-at-the-prior´
/// ´test:crate:extend-raises-the-dimension-and-starts-new-coordinates-at-the-prior´
#[test]
fn extend_raises_the_dimension_and_starts_new_coordinates_at_the_prior() {
    let mut m = BayesianLinearModel::new(100, 0.1, 1000);
    m.extend(5, 0.1);
    assert_eq!(m.dim(), 105);

    // μ[0:100] unchanged (all zero), μ[100:105] = 0
    for i in 0..105 {
        assert_eq!(m.mu()[i], 0.0, "mu[{i}] should be 0");
    }

    // B[0:100,0:100] = 0.1I, B[100:105,100:105] = 0.1I
    for i in 0..105 {
        assert!(
            (m.precision().diagonal_element(i) - 0.1).abs() < 1e-15,
            "B[{i},{i}] = {} expected 0.1",
            m.precision().diagonal_element(i),
        );
    }

    // Σ[0:100,0:100] = 10I, Σ[100:105,100:105] = 10I
    for i in 0..105 {
        assert!(
            (m.covariance().diagonal_element(i) - 10.0).abs() < 1e-15,
            "Σ[{i},{i}] = {} expected 10.0",
            m.covariance().diagonal_element(i),
        );
    }
}

/// Extending by nothing costs nothing and changes nothing — the dimension is
/// exactly what it was. A caller computing how many features to add need not
/// guard the zero case before calling.
///
/// ´claim:bayes:extending-by-zero-dimensions-is-a-no-op´
/// ´test:crate:extend-zero´
#[test]
fn extend_zero() {
    let mut m = BayesianLinearModel::new(10, 1.0, 1000);
    m.extend(0, 1.0);
    assert_eq!(m.dim(), 10);
}

/// What the model already knew survives extension untouched, even when the
/// arriving block is given a different prior precision from the one the
/// original coordinates were built with. Admitting a new feature is an
/// addition, never a partial reset of the posterior already earned.
///
/// ´claim:bayes:extension-leaves-the-existing-coordinates-exactly-as-they-were´
/// ´test:crate:extend-preserves-existing´
#[test]
fn extend_preserves_existing() {
    let mut m = BayesianLinearModel::new(5, 0.5, 1000);
    // Capture original diagonal values.
    let orig_b: Vec<f64> = (0..5).map(|i| m.precision().diagonal_element(i)).collect();
    let orig_s: Vec<f64> = (0..5).map(|i| m.covariance().diagonal_element(i)).collect();

    m.extend(3, 0.2);
    assert_eq!(m.dim(), 8);

    // Original block unchanged.
    for i in 0..5 {
        assert!(
            (m.precision().diagonal_element(i) - orig_b[i]).abs() < 1e-15,
            "B[{i},{i}] changed after extend",
        );
        assert!(
            (m.covariance().diagonal_element(i) - orig_s[i]).abs() < 1e-15,
            "Σ[{i},{i}] changed after extend",
        );
    }

    // New block has correct values.
    for i in 5..8 {
        assert!(
            (m.precision().diagonal_element(i) - 0.2).abs() < 1e-15,
            "B[{i},{i}] = {} expected 0.2",
            m.precision().diagonal_element(i),
        );
        assert!(
            (m.covariance().diagonal_element(i) - 5.0).abs() < 1e-15,
            "Σ[{i},{i}] = {} expected 5.0",
            m.covariance().diagonal_element(i),
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Marginalisation at Layer 1, simplified for a dense, dynamically dimensioned model (´dec:substrate:dense-dynamic´)
// ─────────────────────────────────────────────────────────────────────────────

/// Marginalisation returns a genuinely smaller model — the removed
/// coordinates are gone from the dimension — and what remains is still a
/// usable posterior, with strictly positive precision and covariance on every
/// surviving diagonal. A shrinking model must stay invertible, or the very
/// next update would have nothing well-defined to work from.
///
/// ´claim:bayes:marginalisation-yields-a-smaller-model-that-is-still-positive-on-every-surviving-diagonal´
/// ´test:crate:marginalise-last-two´
#[test]
fn marginalise_last_two() {
    let mut m = BayesianLinearModel::new(100, 0.1, 1000);
    m.marginalise(&[98, 99], &SchurConfig::default(), ModelId::Operational);
    assert_eq!(m.dim(), 98);
    assert_positive_diagonal(m.precision(), "B after marginalise");
    assert_positive_diagonal(m.covariance(), "Σ after marginalise");
}

/// A model that has been shrunk is still snapshottable: taking its parameters
/// and rebuilding from them reproduces the mean and covariance to within
/// rounding and recovers a precision that is still their inverse. Persistence
/// therefore does not have to be scheduled around dimension changes.
///
/// (´claim:bayes:a-snapshot-round-trip-restores-the-mean-and-covariance-and-rebuilds-the-precision-from-them´)
/// ´test:crate:marginalise-followed-by-roundtrip´
#[test]
fn marginalise_followed_by_roundtrip() {
    let mut m = BayesianLinearModel::new(50, 0.1, 1000);
    m.marginalise(&[48, 49], &SchurConfig::default(), ModelId::Operational);
    assert_eq!(m.dim(), 48);

    let params = m.to_parameters();
    let restored = BayesianLinearModel::from_parameters(&params, &FloorMass::default(), 0.1, ModelId::Operational, 1000).unwrap();

    // μ identical.
    for i in 0..48 {
        assert!(
            (restored.mu()[i] - m.mu()[i]).abs() < 1e-14,
            "mu[{i}]: original={} restored={}",
            m.mu()[i],
            restored.mu()[i],
        );
    }

    // Σ identical.
    for i in 0..48 {
        assert!(
            (restored.covariance().diagonal_element(i) - m.covariance().diagonal_element(i)).abs() < 1e-14,
            "Σ[{i},{i}]: original={} restored={}",
            m.covariance().diagonal_element(i),
            restored.covariance().diagonal_element(i),
        );
    }

    // B reconstructed: ‖BΣ − I‖_F < 1e-10.
    let dev = frobenius_deviation_from_identity(restored.precision(), restored.covariance());
    assert_below(dev, 1e-10, "||BΣ - I||_F after marginalise + round-trip");
}

/// Removal indices must arrive sorted ascending, and a caller who passes them
/// the other way round is told so outright in a debug build rather than
/// quietly receiving a model whose blocks were gathered in the wrong order.
/// Silent index scrambling would corrupt the posterior in a way no later
/// assertion could attribute back to its cause.
///
/// ´claim:bayes:marginalisation-demands-its-removal-indices-in-ascending-order´
/// ´test:crate:marginalise-unsorted-panics´
#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "sorted ascending")]
fn marginalise_unsorted_panics() {
    let mut m = BayesianLinearModel::new(10, 0.1, 1000);
    let _ = m.marginalise(&[9, 8], &SchurConfig::default(), ModelId::Operational);
}

// ─────────────────────────────────────────────────────────────────────────────
// Snapshot round-trip
// ─────────────────────────────────────────────────────────────────────────────

/// A snapshot is a faithful description of the posterior: restoring from one
/// gives back the same mean and the same covariance, and the precision is
/// rebuilt from the covariance rather than carried, coming back as its
/// inverse. The stored form can therefore be smaller than the live model
/// without the restored model being any less usable.
///
/// ´claim:bayes:a-snapshot-round-trip-restores-the-mean-and-covariance-and-rebuilds-the-precision-from-them´
/// ´test:crate:to-from-parameters-roundtrip´
#[test]
fn to_from_parameters_roundtrip() {
    let m = BayesianLinearModel::new(50, 0.1, 1000);
    let params = m.to_parameters();
    let restored = BayesianLinearModel::from_parameters(&params, &FloorMass::default(), 0.1, ModelId::Operational, 1000).unwrap();

    // μ identical.
    for i in 0..50 {
        assert!((restored.mu()[i] - m.mu()[i]).abs() < 1e-15, "mu[{i}] differs");
    }

    // Σ identical.
    for i in 0..50 {
        assert!(
            (restored.covariance().diagonal_element(i) - m.covariance().diagonal_element(i)).abs() < 1e-15,
            "Σ[{i},{i}] differs",
        );
    }

    // B reconstructed: ‖BΣ − I‖_F < 1e-10.
    let dev = frobenius_deviation_from_identity(restored.precision(), restored.covariance());
    assert_below(dev, 1e-10, "||BΣ - I||_F after to/from parameters round-trip");
}

/// A restored model owes no recomputation: its label counter starts at zero
/// because the precision it holds was just derived from the covariance by a
/// fresh factorisation. Carrying the pre-snapshot count across would schedule
/// a repair for drift that no longer exists.
///
/// ´claim:bayes:a-model-restored-from-a-snapshot-begins-its-recompute-cadence-afresh´
/// ´test:crate:from-parameters-resets-recompute-counter´
#[test]
fn from_parameters_resets_recompute_counter() {
    let m = BayesianLinearModel::new(10, 0.1, 1000);
    let params = m.to_parameters();
    let restored = BayesianLinearModel::from_parameters(&params, &FloorMass::default(), 0.1, ModelId::Operational, 1000).unwrap();
    assert_eq!(restored.labels_since_recompute(), 0);
}

// ─────────────────────────────────────────────────────────────────────────────
// Dual tracking
// ─────────────────────────────────────────────────────────────────────────────

/// The model carries precision and covariance side by side as a mutually
/// inverse pair, and at construction their product is the identity to machine
/// precision. Both are needed on the hot path — the covariance for leverage,
/// the precision for the update — and keeping them in step is what makes it
/// safe to read either one without factorising first.
///
/// ´claim:bayes:precision-and-covariance-are-maintained-as-a-mutually-inverse-pair´
/// ´test:crate:new-dual-tracking´
#[test]
fn new_dual_tracking() {
    let m = BayesianLinearModel::new(20, 0.1, 1000);
    let dev = frobenius_deviation_from_identity(m.precision(), m.covariance());
    assert_below(dev, 1e-15, "||BΣ - I||_F at construction");
}

/// Growing the model does not break the pairing: after extension the two
/// matrices are still inverses of one another across the whole enlarged
/// dimension, old block and new block alike. Extension has to build the new
/// diagonal entries in both matrices, and getting only one of them right
/// would be invisible until the next leverage computation went wrong.
///
/// (´claim:bayes:precision-and-covariance-are-maintained-as-a-mutually-inverse-pair´)
/// ´test:crate:extend-dual-tracking´
#[test]
fn extend_dual_tracking() {
    let mut m = BayesianLinearModel::new(20, 0.1, 1000);
    m.extend(5, 0.2);
    let dev = frobenius_deviation_from_identity(m.precision(), m.covariance());
    assert_below(dev, 1e-10, "||BΣ - I||_F after extend");
}

/// Shrinking the model does not break the pairing either: the surviving
/// precision and covariance blocks are still inverses of one another. Because
/// the marginal precision is not simply the precision's submatrix, the
/// covariance cannot be carried over unexamined, and this is what pins that
/// the two sides were derived consistently.
///
/// (´claim:bayes:precision-and-covariance-are-maintained-as-a-mutually-inverse-pair´)
/// ´test:crate:marginalise-dual-tracking´
#[test]
fn marginalise_dual_tracking() {
    let mut m = BayesianLinearModel::new(20, 0.1, 1000);
    m.marginalise(&[18, 19], &SchurConfig::default(), ModelId::Operational);
    let dev = frobenius_deviation_from_identity(m.precision(), m.covariance());
    assert_below(dev, 1e-10, "||BΣ - I||_F after marginalise");
}

// ─────────────────────────────────────────────────────────────────────────────
// Extension: cross-blocks
// ─────────────────────────────────────────────────────────────────────────────

/// Coordinates that arrive by extension arrive independent: every entry
/// linking an old dimension to a new one is exactly zero in both the precision
/// and the covariance. A newly admitted feature starts with no borrowed
/// correlation, so whatever relationship it eventually shows must have been
/// learned from observations rather than inherited from construction.
///
/// ´claim:bayes:new-coordinates-arrive-uncorrelated-with-the-existing-ones´
/// ´test:crate:extend-cross-blocks-zero´
#[test]
fn extend_cross_blocks_zero() {
    let mut m = BayesianLinearModel::new(10, 0.5, 1000);
    m.extend(3, 0.2);

    let b = m.precision().as_inner();
    let s = m.covariance().as_inner();
    for i in 0..10 {
        for j in 10..13 {
            assert_eq!(b[(i, j)], 0.0, "B[{i},{j}] should be zero (cross-block)");
            assert_eq!(b[(j, i)], 0.0, "B[{j},{i}] should be zero (cross-block)");
            assert_eq!(s[(i, j)], 0.0, "Σ[{i},{j}] should be zero (cross-block)");
            assert_eq!(s[(j, i)], 0.0, "Σ[{j},{i}] should be zero (cross-block)");
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Marginalisation: edge cases
// ─────────────────────────────────────────────────────────────────────────────

/// Removing several dimensions at once leaves the label counter at zero,
/// because that path rebuilds the covariance by a fresh factorisation and so
/// discharges whatever drift had accumulated. The counter measures debt since
/// the last exact inversion, and this operation has just paid it.
///
/// (´claim:bayes:a-multi-dimension-marginalisation-restarts-the-recompute-cadence´)
/// ´test:crate:marginalise-resets-counter´
#[test]
fn marginalise_resets_counter() {
    let mut m = BayesianLinearModel::new(20, 0.1, 1000);
    m.marginalise(&[18, 19], &SchurConfig::default(), ModelId::Operational);
    assert_eq!(m.labels_since_recompute(), 0);
}

/// Marginalising an empty set leaves the model at its original dimension and
/// succeeds rather than erroring. Callers that compute a removal list can pass
/// it straight through when it happens to come out empty.
///
/// (´claim:bayes:removing-no-dimensions-leaves-the-precision-exactly-as-it-was´)
/// ´test:crate:marginalise-empty-indices´
#[test]
fn marginalise_empty_indices() {
    let mut m = BayesianLinearModel::new(10, 0.1, 1000);
    m.marginalise(&[], &SchurConfig::default(), ModelId::Operational);
    assert_eq!(m.dim(), 10);
}

/// Marginalisation holds all the way down to a single surviving coordinate:
/// the dimension falls to one and that coordinate still carries a positive
/// precision and covariance. There is no smallest useful model below which the
/// block arithmetic degenerates.
///
/// (´claim:bayes:marginalisation-yields-a-smaller-model-that-is-still-positive-on-every-surviving-diagonal´)
/// ´test:crate:marginalise-all-but-one´
#[test]
fn marginalise_all_but_one() {
    let mut m = BayesianLinearModel::new(10, 0.1, 1000);
    let remove: Vec<usize> = (1..10).collect();
    m.marginalise(&remove, &SchurConfig::default(), ModelId::Operational);
    assert_eq!(m.dim(), 1);
    assert_positive_diagonal(m.precision(), "B after marginalise to 1");
    assert_positive_diagonal(m.covariance(), "Σ after marginalise to 1");
}

// ─────────────────────────────────────────────────────────────────────────────
// Snapshot: excludes B, error path
// ─────────────────────────────────────────────────────────────────────────────

/// A snapshot carries the covariance and not the precision: the stored matrix
/// holds the reciprocal of the prior on its diagonal, not the prior itself.
/// Storing only one of the pair halves the persisted size and removes any
/// possibility of loading a snapshot whose two halves disagree.
///
/// ´claim:bayes:a-snapshot-carries-the-covariance-and-not-the-precision´
/// ´test:crate:to-parameters-excludes-b´
#[test]
fn to_parameters_excludes_b() {
    let m = BayesianLinearModel::new(5, 0.1, 1000);
    let params = m.to_parameters();

    // ModelParameters only has mu, covariance_data, and p — no
    // precision field. Verify structurally by checking the
    // covariance data matches Σ (not B).
    assert_eq!(params.p, 5);
    assert_eq!(params.mu.len(), 5);
    assert_eq!(params.covariance_data.len(), 25);

    // covariance_data should contain 1/0.1 = 10.0 on the diagonal,
    // not 0.1 (which would be B).
    for i in 0..5 {
        let idx = i * 5 + i; // column-major diagonal for square matrix
        assert!(
            (params.covariance_data[idx] - 10.0).abs() < 1e-15,
            "covariance_data diagonal[{i}] = {} (expected 10.0, not B=0.1)",
            params.covariance_data[idx],
        );
    }
}

/// A snapshot whose covariance cannot be factorised — here one carrying a
/// non-numeric diagonal — is refused, both the plain and the regularised
/// factorisation having failed. Loading it anyway would produce a model whose
/// precision is meaningless, and the failure would surface far from the
/// corrupt input that caused it.
///
/// ´claim:bayes:a-snapshot-whose-covariance-cannot-be-factorised-is-refused-rather-than-loaded´
/// ´test:crate:from-parameters-with-bad-sigma´
#[test]
fn from_parameters_with_bad_sigma() {
    // Build a covariance containing NaN — both plain and regularised
    // Cholesky will fail, triggering the `Failed` → `RevertError` path.
    let params = ModelParameters {
        mu: vec![0.0; 3],
        covariance_data: vec![
            f64::NAN,
            0.0,
            0.0, // col 0: NaN diagonal → unfactorisable
            0.0,
            1.0,
            0.0, // col 1
            0.0,
            0.0,
            1.0, // col 2
        ],
        p: 3,
    };
    let result = BayesianLinearModel::from_parameters(&params, &FloorMass::default(), 0.1, ModelId::Operational, 1000);
    assert!(result.is_err(), "should reject non-SPD covariance");
}

// ─────────────────────────────────────────────────────────────────────────────
// Condition-adaptive recomputation tracking fields
// (´alg:gaussian:condition-adaptive-recompute´)
// ─────────────────────────────────────────────────────────────────────────────

/// A fresh model starts its numerical-health bookkeeping from a clean slate:
/// the effective recomputation interval is the configured one rather than a
/// halved value, and no clean streak, conditioning estimate, or sync error is
/// yet on record. Adaptive cadence has to begin optimistic — a model with no
/// history has given no reason to distrust it.
///
/// ´claim:bayes:a-fresh-model-starts-with-the-configured-recompute-interval-and-no-health-history´
/// ´test:crate:new-l024-defaults´
#[test]
fn new_l024_defaults() {
    let m = BayesianLinearModel::new(10, 0.1, 1000);
    assert_eq!(m.n_recompute_effective(), 1000);
    assert_eq!(m.consecutive_clean_recomputes(), 0);
    assert_eq!(m.last_diagonal_ratio(), 0.0);
    assert_eq!(m.last_sync_error(), 0.0);
    assert_eq!(m.sync_error_shortenings(), 0);
}

/// The live model applies the clean-high-error rule rather than merely exposing
/// it through a helper: an error above the width-scaled threshold halves its
/// interval, clears recovery, records the error and advances the shortening
/// count.
///
/// ´claim:bayes:the-live-model-shortens-and-restarts-recovery-on-a-needed-rebuild´
/// ´test:crate:clean-high-sync-error-updates-model-cadence´
#[test]
fn clean_high_sync_error_updates_model_cadence() {
    let mut model = BayesianLinearModel::new(10, 0.1, 1000);
    let sync_error = synchronisation_error_threshold(model.dim()) * 2.0;
    let outcome = RecomputeOutcome::Rebuilt {
        sync_error_residual: sync_error,
        drift_was_real: true,
    };

    // A pair is supplied because an adopted rebuild is what this reads: the
    // alarm shortens too, and it shortens all the way to the floor
    // (´dec:posterior:measured-adoption´).
    let adopted = crate::model::recompute::RebuiltPair {
        precision: crate::linalg::symmetric::SymmetricMatrix::identity_scaled(model.dim(), 0.1),
        covariance: crate::linalg::symmetric::SymmetricMatrix::identity_scaled(model.dim(), 10.0),
    };
    model.apply_recompute_outcome(&outcome, Some(adopted), None, 1000);

    assert_eq!(model.n_recompute_effective(), 500);
    assert_eq!(model.consecutive_clean_recomputes(), 0);
    assert_eq!(model.last_sync_error(), sync_error);
    assert_eq!(model.sync_error_shortenings(), 1);
}

// ─────────────────────────────────────────────────────────────────────────────
// Sherman–Morrison update (´alg:update:sherman-morrison´)
// ─────────────────────────────────────────────────────────────────────────────

use crate::linalg::convert::vec_to_col;
use crate::model::update::{
    DEFAULT_IMPORTANCE_CEILING, DEFAULT_LEVERAGE_SAFETY_FACTOR, EPSILON_LEVERAGE, compute_effective_weight,
};

/// The weight an observation actually receives is the smallest of three
/// limits — the weight asked for, the global importance ceiling, and the
/// leverage bound derived from how far the feature sticks out under the
/// current covariance — and each of the three is shown taking its turn as the
/// binding one. Any one of them alone would be insufficient: the ceiling
/// caps runaway importance, the leverage bound caps updates that a single
/// observation could otherwise dominate.
///
/// ´claim:bayes:the-effective-weight-is-the-smallest-of-the-target-the-ceiling-and-the-leverage-bound´
/// ´test:crate:effective-weight-is-the-smallest-of-target-ceiling-and-leverage-bound´
#[test]
fn effective_weight_is_the_smallest_of_target_ceiling_and_leverage_bound() {
    // Case 1: leverage bound is tightest
    let w = compute_effective_weight(50.0, 100.0, 5.0, 1.0, 1e-8);
    assert!((w - 5.0 / 1.000_000_01).abs() < 1e-10);

    // Case 2: target is tightest
    let w = compute_effective_weight(2.0, 100.0, 5.0, 0.01, 1e-8);
    assert!((w - 2.0).abs() < 1e-15);

    // Case 3: ceiling is tightest
    let w = compute_effective_weight(200.0, 10.0, 500.0, 0.1, 1e-8);
    assert!((w - 10.0).abs() < 1e-15);
}

/// At the cold-start prior the covariance is wide, so almost any feature has
/// large leverage and the leverage limit — not the requested importance — is
/// what decides the weight: across a hundred varied feature vectors it binds
/// on essentially all of them. This is the regime the bound exists for, since
/// an early observation admitted at full weight would move a model that has
/// nothing yet to hold it steady.
///
/// ´claim:bayes:at-the-cold-start-prior-the-leverage-bound-is-what-limits-nearly-every-update´
/// ´test:crate:leverage-bound-fires-at-cold-start´
#[test]
fn leverage_bound_fires_at_cold_start() {
    let p = 100;
    let prior_precision = 0.1; // Σ = 10I at cold start
    let model = BayesianLinearModel::new(p, prior_precision, 1000);

    let mut bound_fired = 0;
    let total = 100;

    for seed in 0..total {
        // Generate deterministic "random" phi_hat
        let phi: Vec<f64> = (0..p).map(|i| ((seed * 17 + i * 31) % 100) as f64 / 50.0 - 1.0).collect();
        let phi_hat = vec_to_col(&phi);

        let (_, h) = model.covariance().quadratic_form_with_product(&phi_hat);

        let w_imp = 50.0;
        let w_eff = compute_effective_weight(
            w_imp,
            DEFAULT_IMPORTANCE_CEILING,
            DEFAULT_LEVERAGE_SAFETY_FACTOR,
            h,
            EPSILON_LEVERAGE,
        );

        if w_eff < w_imp {
            bound_fired += 1;
        }
    }

    // At least 95% should have leverage-bounded weight
    assert!(
        bound_fired >= 95,
        "leverage bound fired {bound_fired}/100 times (expected >= 95)"
    );
}

/// Every posterior update advances by exactly one the count of labels absorbed
/// since the last exact factorisation. That count is the model's own measure of
/// how much incremental drift it may have taken on, and it is what the
/// recomputation schedule reads to decide when to pay for a fresh inversion.
///
/// ´claim:bayes:each-posterior-update-advances-the-label-count-that-drives-recomputation´
/// ´test:crate:labels-since-recompute-incremented´
#[test]
fn labels_since_recompute_incremented() {
    let p = 10;
    let mut model = BayesianLinearModel::new(p, 0.1, 1000);
    assert_eq!(model.labels_since_recompute(), 0);

    let phi: Vec<f64> = (0..p).map(|i| i as f64 * 0.1).collect();
    let phi_hat = vec_to_col(&phi);

    for i in 1..=5 {
        let (v, h) = model.covariance().quadratic_form_with_product(&phi_hat);
        let w_eff = compute_effective_weight(1.0, 100.0, 5.0, h, 1e-8);
        model.apply_leverage_bounded_update(&phi_hat, &v, h, w_eff, 1.0, 0.999, 1e-4);
        assert_eq!(model.labels_since_recompute(), i);
    }
}

/// The residual is taken against the mean as it stood before the update, so a
/// positive target pulls the mean in the positive direction. Were the mean
/// mutated first and the residual measured afterwards, the observation would
/// partly explain away its own surprise and the model would under-react to
/// exactly the evidence it most needs to absorb.
///
/// ´claim:bayes:the-residual-is-measured-against-the-mean-that-preceded-the-update´
/// ´test:crate:mean-update-ordering´
#[test]
fn mean_update_ordering() {
    let p = 5;
    let prior_precision = 0.1;
    let mut model = BayesianLinearModel::new(p, prior_precision, 1000);

    // phi_hat = [1, 1, 1, 1, 1] (normalised input)
    let phi: Vec<f64> = vec![1.0; p];
    let phi_hat = vec_to_col(&phi);

    // At cold start μ = 0, so φ̂ᵀμ = 0, residual = r - 0 = r
    let r_rho = 1.0; // positive target

    let (v, h) = model.covariance().quadratic_form_with_product(&phi_hat);
    let w_eff = compute_effective_weight(1.0, 100.0, 5.0, h, 1e-8);
    model.apply_leverage_bounded_update(&phi_hat, &v, h, w_eff, r_rho, 0.999, 1e-4);

    // μ should have moved toward positive direction
    let mu_sum: f64 = (0..p).map(|i| model.mu()[i]).sum();
    assert!(mu_sum > 0.0, "μ sum = {mu_sum}, expected positive movement toward r=1");
}

/// The rank-one update leaves an exact algebraic trace: applying the updated
/// covariance to the very feature just observed reproduces that feature's
/// pre-update image, shrunk by the forgetting factor plus the weighted
/// leverage. This is the identity the incremental formula is derived from, so
/// it holding element-wise is what distinguishes a correct update from one
/// that merely moves the matrix in a plausible direction.
///
/// ´claim:bayes:after-an-update-the-covariance-applied-to-the-observed-feature-is-its-old-image-shrunk-by-the-update-denominator´
/// ´test:crate:sigma-phi-identity´
#[test]
fn sigma_phi_identity() {
    let p = 10;
    let prior_precision = 0.1;
    let mut model = BayesianLinearModel::new(p, prior_precision, 1000);

    let phi: Vec<f64> = (0..p).map(|i| (i as f64 + 1.0) / 10.0).collect();
    let phi_hat = vec_to_col(&phi);

    let (v, h) = model.covariance().quadratic_form_with_product(&phi_hat);
    let gamma_eff = 0.999;
    let w_eff = 0.5; // small weight for controlled test

    model.apply_leverage_bounded_update(&phi_hat, &v, h, w_eff, 0.0, gamma_eff, 1e-4);

    // Expected: Σ_new φ̂ = v / (γ + w·h)
    let expected_scale = 1.0 / (gamma_eff + w_eff * h);
    let sigma_phi = model.covariance().symmetric_matvec(&phi_hat);

    for i in 0..p {
        let expected = v[i] * expected_scale;
        let actual = sigma_phi[i];
        assert!(
            (actual - expected).abs() < 1e-12,
            "Σφ̂[{i}]: expected {expected}, got {actual}"
        );
    }
}

/// A run of incremental updates keeps the two matrices inverse to one another:
/// after ten varied observations no diagonal of their product has departed
/// measurably from one. The precision and covariance are updated by separate
/// rank-one formulas, so this is the check that the two formulas remain each
/// other's mirror rather than slowly parting company.
///
/// (´claim:bayes:precision-and-covariance-are-maintained-as-a-mutually-inverse-pair´)
/// ´test:crate:dual-tracking-after-update´
#[test]
fn dual_tracking_after_update() {
    let p = 20;
    let prior_precision = 0.1;
    let mut model = BayesianLinearModel::new(p, prior_precision, 1000);

    // Apply several updates
    for seed in 0..10 {
        let phi: Vec<f64> = (0..p).map(|i| ((seed * 13 + i * 7) % 100) as f64 / 50.0 - 1.0).collect();
        let phi_hat = vec_to_col(&phi);

        let (v, h) = model.covariance().quadratic_form_with_product(&phi_hat);
        let w_eff = compute_effective_weight(1.0, 100.0, 5.0, h, 1e-8);
        model.apply_leverage_bounded_update(&phi_hat, &v, h, w_eff, 0.5, 0.999, 1e-4);
    }

    // Check diagonal sync error: max_j |(BΣ)_{jj} − 1|
    let b = model.precision();
    let sigma = model.covariance();
    let product = b.as_inner() * sigma.as_inner();

    let mut max_error = 0.0f64;
    for j in 0..p {
        let diag = product[(j, j)];
        let error = (diag - 1.0).abs();
        max_error = max_error.max(error);
    }

    assert_below(max_error, 1e-10, "diagonal sync error");
}

/// An observation admitted at zero weight teaches the model nothing — the mean
/// does not move at all — yet time still passes for it: the precision is
/// decayed by the forgetting factor and clamped at its floor exactly as on any
/// other update. Forgetting is a property of the step, not of the evidence, so
/// a rejected observation must not also freeze the model in place.
///
/// ´claim:bayes:an-update-at-zero-effective-weight-still-forgets-but-does-not-move-the-mean´
/// ´test:crate:zero-effective-weight´
#[test]
fn zero_effective_weight() {
    let p = 5;
    let prior_precision = 0.1;
    let mut model = BayesianLinearModel::new(p, prior_precision, 1000);

    // Apply one update to set μ to something non-zero
    let phi1 = vec_to_col(&[1.0, 0.5, 0.3, 0.2, 0.1]);
    let (v1, h1) = model.covariance().quadratic_form_with_product(&phi1);
    model.apply_leverage_bounded_update(&phi1, &v1, h1, 1.0, 1.0, 0.999, 1e-4);

    // Capture μ before zero-weight update
    let mu_before: Vec<f64> = (0..p).map(|i| model.mu()[i]).collect();
    let b_diag_before: Vec<f64> = (0..p).map(|i| model.precision().diagonal_element(i)).collect();

    // Apply update with w_eff = 0.0
    let phi2 = vec_to_col(&[0.5, 0.5, 0.5, 0.5, 0.5]);
    let (v2, h2) = model.covariance().quadratic_form_with_product(&phi2);
    model.apply_leverage_bounded_update(&phi2, &v2, h2, 0.0, 1.0, 0.999, 1e-4);

    // μ should be unchanged (no rank-1 update to mean)
    for (i, &before) in mu_before.iter().enumerate() {
        assert!(
            (model.mu()[i] - before).abs() < 1e-15,
            "μ[{i}] changed: {before} → {}",
            model.mu()[i]
        );
    }

    // B should have decayed (scaled by γ) but no rank-1 update
    // B_new = γ · B_old (clamped), so diagonal should be approximately γ × old
    let gamma = 0.999;
    for (i, &before) in b_diag_before.iter().enumerate() {
        let expected = (gamma * before).max(1e-4);
        let actual = model.precision().diagonal_element(i);
        assert!(
            (actual - expected).abs() < 1e-12,
            "B[{i},{i}]: expected {expected}, got {actual}"
        );
    }
}

/// A couple of hundred labels of mixed valence are enough to carry the mean a
/// measurable distance from the origin it started at. The leverage bound and
/// the forgetting factor both hold updates back, and this pins that they hold
/// it back rather than pinning it: a model that never leaves its prior would
/// pass every stability check while learning nothing.
///
/// ´claim:bayes:a-few-hundred-labels-carry-the-mean-away-from-the-prior´
/// ´test:crate:q1-model-moves-from-prior´
#[test]
fn q1_model_moves_from_prior() {
    let p = 50;
    let prior_precision = 0.1;
    let mut model = BayesianLinearModel::new(p, prior_precision, 1000);

    // Process 200 labels with ~20% positive valence
    for i in 0..200 {
        let is_positive = i % 5 == 0; // 20% positive
        let r_rho = if is_positive { 1.0 } else { -1.0 };

        // Deterministic feature vector
        let phi: Vec<f64> = (0..p).map(|j| ((i * 17 + j * 31) % 100) as f64 / 100.0).collect();
        let phi_hat = vec_to_col(&phi);

        let (v, h) = model.covariance().quadratic_form_with_product(&phi_hat);
        let w_eff = compute_effective_weight(1.0, 100.0, 5.0, h, 1e-8);
        model.apply_leverage_bounded_update(&phi_hat, &v, h, w_eff, r_rho, 0.9998, 1e-4);
    }

    // ‖μ − 0‖₂ > 0.01 — model has moved from prior
    let mu_norm: f64 = (0..p).map(|i| model.mu()[i].powi(2)).sum::<f64>().sqrt();
    assert!(
        mu_norm > 0.01,
        "‖μ‖₂ = {mu_norm}, expected > 0.01 (model should have moved from prior)"
    );
}

/// The incremental path can be run past a full recomputation window — 1,400
/// labels with no exact refactorisation at all — and still hold together:
/// every value stays finite, the two matrices are still each other's inverse
/// on the diagonal, and the covariance still returns non-negative quadratic
/// forms. The recomputation cadence exists to bound drift, and this fixes that
/// the model degrades gracefully rather than falling apart the moment the
/// cadence is exceeded.
///
/// ´claim:bayes:the-model-survives-a-whole-recompute-window-of-updates-without-losing-finiteness-or-synchronisation´
/// ´test:crate:n1-window-safety´
#[test]
fn n1_window_safety() {
    let p = 30;
    let prior_precision = 0.1;
    let mut model = BayesianLinearModel::new(p, prior_precision, 1000);

    // Process 1,400 labels without Cholesky recomputation
    for i in 0..1400 {
        let r_rho = if i % 3 == 0 { 1.0 } else { -0.5 };

        let phi: Vec<f64> = (0..p).map(|j| ((i * 13 + j * 7) % 100) as f64 / 100.0 - 0.5).collect();
        let phi_hat = vec_to_col(&phi);

        let (v, h) = model.covariance().quadratic_form_with_product(&phi_hat);
        let w_eff = compute_effective_weight(1.0, 100.0, 5.0, h, 1e-8);
        model.apply_leverage_bounded_update(&phi_hat, &v, h, w_eff, r_rho, 0.9995, 1e-4);
    }

    // No NaN in μ, B diagonal, Σ diagonal.
    let mu_vals: Vec<f64> = (0..p).map(|i| model.mu()[i]).collect();
    let b_diag: Vec<f64> = (0..p).map(|i| model.precision().diagonal_element(i)).collect();
    let s_diag: Vec<f64> = (0..p).map(|i| model.covariance().diagonal_element(i)).collect();
    assert_finite(&mu_vals, "μ after 1,400 labels");
    assert_finite(&b_diag, "B diagonal after 1,400 labels");
    assert_finite(&s_diag, "Σ diagonal after 1,400 labels");

    // Diagonal sync error < 1e-8
    let b = model.precision();
    let sigma = model.covariance();
    let product = b.as_inner() * sigma.as_inner();

    let mut max_error = 0.0f64;
    for j in 0..p {
        let diag = product[(j, j)];
        let error = (diag - 1.0).abs();
        max_error = max_error.max(error);
    }

    assert_below(max_error, 1e-8, "diagonal sync error after 1,400 labels");

    // All quadratic forms non-negative
    for i in 0..10 {
        let phi: Vec<f64> = (0..p).map(|j| ((i * 11 + j * 3) % 100) as f64 / 100.0).collect();
        let phi_hat = vec_to_col(&phi);
        let qf = model.covariance().quadratic_form(&phi_hat);
        assert!(qf >= 0.0 || qf.abs() < 1e-14, "quadratic form {qf} is negative");
    }
}

/// Forgetting by the clock and forgetting per label compose: over a hundred
/// labels, folding both into one combined factor gives the same mean,
/// precision, and covariance as decaying for elapsed time first and then
/// applying the per-label factor through the update. Because they compose, the
/// implementation may collapse two decays into one multiplication on the hot
/// path instead of touching the matrices twice per observation.
///
/// ´claim:bayes:time-decay-and-label-decay-compose-into-a-single-equivalent-factor´
/// ´test:crate:combined-decay-equivalence´
#[test]
fn combined_decay_equivalence() {
    let p = 30;
    let prior_precision = 0.1;

    // Per-label decay and time decay factors
    let gamma_label: f64 = 0.9998;
    let gamma_t: f64 = 0.9995; // per hour
    let elapsed_hours: f64 = 0.25; // 15 minutes between labels
    let gamma_t_dt = gamma_t.powf(elapsed_hours);
    let gamma_combined = gamma_label * gamma_t_dt;

    // Model A: uses combined decay
    let mut model_a = BayesianLinearModel::new(p, prior_precision, 1000);

    // Model B: applies time-decay to B/Σ first, then label-decay via update
    let mut model_b = BayesianLinearModel::new(p, prior_precision, 1000);

    // Use small fixed w_eff to eliminate leverage-bound differences
    let w_eff = 0.1;

    // Process 100 labels
    for i in 0..100 {
        let r_rho = if i % 5 == 0 { 1.0 } else { -0.5 };
        let phi: Vec<f64> = (0..p).map(|j| ((i * 13 + j * 7) % 100) as f64 / 100.0 - 0.5).collect();
        let phi_hat = vec_to_col(&phi);

        // Model A: compute (v, h) and apply combined decay
        let (v_a, h_a) = model_a.covariance().quadratic_form_with_product(&phi_hat);
        model_a.apply_leverage_bounded_update(&phi_hat, &v_a, h_a, w_eff, r_rho, gamma_combined, 1e-4);

        // Model B: apply time-decay first
        model_b.apply_time_decay(gamma_t_dt);

        // Compute (v, h) from time-decayed Σ
        let (v_b, h_b) = model_b.covariance().quadratic_form_with_product(&phi_hat);

        // Apply label-only decay
        model_b.apply_leverage_bounded_update(&phi_hat, &v_b, h_b, w_eff, r_rho, gamma_label, 1e-4);
    }

    // Compare μ element-wise
    let mut max_mu_diff = 0.0f64;
    for i in 0..p {
        let diff = (model_a.mu()[i] - model_b.mu()[i]).abs();
        max_mu_diff = max_mu_diff.max(diff);
    }
    assert!(max_mu_diff < 1e-12, "max μ difference = {max_mu_diff:e}, expected < 1e-12");

    // Compare B diagonals
    let mut max_b_diff = 0.0f64;
    for i in 0..p {
        let diff = (model_a.precision().diagonal_element(i) - model_b.precision().diagonal_element(i)).abs();
        max_b_diff = max_b_diff.max(diff);
    }
    assert!(
        max_b_diff < 1e-12,
        "max B diagonal difference = {max_b_diff:e}, expected < 1e-12"
    );

    // Compare Σ diagonals
    let mut max_sigma_diff = 0.0f64;
    for i in 0..p {
        let diff = (model_a.covariance().diagonal_element(i) - model_b.covariance().diagonal_element(i)).abs();
        max_sigma_diff = max_sigma_diff.max(diff);
    }
    assert!(
        max_sigma_diff < 1e-12,
        "max Σ diagonal difference = {max_sigma_diff:e}, expected < 1e-12"
    );
}

/// The precision floor is a clamp, and a clamp breaks the exact inverse
/// relationship the covariance update assumes. Under a floor chosen to fire on
/// every dimension of every one of two hundred labels, the resulting
/// mismatch stays bounded rather than compounding, and the model stays finite
/// with positive diagonals throughout. The floor is what keeps a decayed
/// precision from collapsing toward zero, so its cost has to be a small
/// standing error and not a divergence.
///
/// ´claim:bayes:clamping-the-precision-at-its-floor-on-every-label-leaves-a-bounded-not-a-diverging-mismatch´
/// ´test:crate:floor-drift-bounded´
#[test]
fn floor_drift_bounded() {
    let p = 10;
    let prior_precision = 0.01; // Low prior → B_jj = 0.01, Σ_jj = 100
    let mut model = BayesianLinearModel::new(p, prior_precision, 1000);

    // Use a high floor that will fire on most dimensions after decay.
    // γ_eff = 0.999, so after decay B_jj = 0.999 * 0.01 = 0.00999 < 0.01.
    // The floor at 0.01 will clamp on every label.
    let lambda_floor = 0.01;
    let gamma_eff = 0.999;
    let n_labels = 200;

    for i in 0..n_labels {
        let phi: Vec<f64> = (0..p).map(|j| ((i * 17 + j * 11) % 100) as f64 / 100.0 - 0.5).collect();
        let phi_hat = vec_to_col(&phi);

        let (v, h) = model.covariance().quadratic_form_with_product(&phi_hat);
        let w_eff = compute_effective_weight(1.0, 100.0, 5.0, h, 1e-8);
        model.apply_leverage_bounded_update(&phi_hat, &v, h, w_eff, 0.5, gamma_eff, lambda_floor);
    }

    // Measure ‖BΣ − I‖_F — should be bounded, not divergent.
    let dev = frobenius_deviation_from_identity(model.precision(), model.covariance());

    // Under the prior replenishment floor (´req:gaussian:prior-replenishment-floor´),
    // at λ_floor=0.01, γ_eff≈0.999, d_floor≤p=10, N=200:
    // Cumulative ‖Δ‖ ≲ N · δ · d_floor = 200 · 1e-5 · 10 = 0.02.
    // Allow generous margin for the full Frobenius norm.
    assert_below(dev, 1.0, "‖BΣ − I‖_F with forced floor firing");

    // No NaN/Inf in μ; B and Σ strictly positive on the diagonal.
    let mu_vals: Vec<f64> = (0..p).map(|i| model.mu()[i]).collect();
    assert_finite(&mu_vals, "μ after forced-floor updates");
    assert_positive_diagonal(model.precision(), "B after forced-floor updates");
    assert_positive_diagonal(model.covariance(), "Σ after forced-floor updates");
}

/// At one dimension the matrix update collapses to a scalar recursion that can
/// be written out by hand, and the implementation reproduces every step of it
/// — decayed precision, floor, rank-one addition, covariance shrinkage, mean
/// shift — to within rounding, with the two scalars still reciprocal
/// afterwards. The scalar case is the one place the whole formula can be
/// checked against arithmetic rather than against another implementation of
/// itself.
///
/// ´claim:bayes:at-one-dimension-the-update-reproduces-the-closed-form-scalar-recursion´
/// ´test:crate:dimension-1-scalar´
#[test]
fn dimension_1_scalar() {
    let p = 1;
    let prior_precision = 0.5; // B=0.5, Σ=2.0
    let mut model = BayesianLinearModel::new(p, prior_precision, 1000);

    let phi_val = 0.8;
    let phi_hat = vec_to_col(&[phi_val]);
    let gamma_eff = 0.998;
    let lambda_floor = 1e-4;
    let r_rho = 1.0;

    // Compute expected values analytically.
    let b_0 = 0.5;
    let sigma_0 = 2.0;
    let mu_0 = 0.0;

    // v = Σ · φ̂, h = φ̂ · v
    let v_expected = sigma_0 * phi_val;
    let h_expected = phi_val * v_expected;

    // Effective weight
    let w_eff = compute_effective_weight(1.0, 100.0, 5.0, h_expected, 1e-8);

    // Step 3: B ← γ · B
    let b_decayed: f64 = gamma_eff * b_0;
    // Step 4: floor (b_decayed = 0.499 > 1e-4, no clamping)
    let b_floored = b_decayed.max(lambda_floor);
    // Step 5: B ← B + w · φ²
    let b_new = b_floored + w_eff * phi_val * phi_val;
    // Step 6: Σ_new via formula
    let denom = gamma_eff + w_eff * h_expected;
    let sigma_new = (1.0 / gamma_eff) * (sigma_0 - w_eff * v_expected * v_expected / denom);
    // Step 7: μ_new = μ + w(r − φ̂ᵀμ) · v / denom
    let residual = r_rho - phi_val * mu_0;
    let mu_new = mu_0 + w_eff * residual * v_expected / denom;

    // Apply via the actual implementation
    let (v, h) = model.covariance().quadratic_form_with_product(&phi_hat);
    assert!((v[0] - v_expected).abs() < 1e-15, "v mismatch");
    assert!((h - h_expected).abs() < 1e-15, "h mismatch");

    model.apply_leverage_bounded_update(&phi_hat, &v, h, w_eff, r_rho, gamma_eff, lambda_floor);

    assert!(
        (model.mu()[0] - mu_new).abs() < 1e-14,
        "μ: expected {mu_new}, got {}",
        model.mu()[0]
    );
    assert!(
        (model.precision().diagonal_element(0) - b_new).abs() < 1e-14,
        "B: expected {b_new}, got {}",
        model.precision().diagonal_element(0)
    );
    assert!(
        (model.covariance().diagonal_element(0) - sigma_new).abs() < 1e-14,
        "Σ: expected {sigma_new}, got {}",
        model.covariance().diagonal_element(0)
    );

    // Dual tracking: B·Σ ≈ 1
    let bs = model.precision().diagonal_element(0) * model.covariance().diagonal_element(0);
    assert!((bs - 1.0).abs() < 1e-12, "BΣ = {bs}, expected ≈ 1.0");
}

/// The small dimension the anchor models actually run at is not a special
/// case: after fifty updates the precision and covariance are still inverses
/// to the full matrix tolerance, and the covariance-feature identity still
/// holds exactly on a further update. Small models take the same code path as
/// large ones, and the invariants that were fixed at larger dimensions are
/// shown to survive where the arithmetic has fewest terms to average over.
///
/// ´claim:bayes:the-update-invariants-hold-unchanged-at-the-small-anchor-dimension´
/// ´test:crate:anchor-p15-dual-tracking´
#[test]
fn anchor_p15_dual_tracking() {
    let p = 15;
    let prior_precision = 0.1;
    let mut model = BayesianLinearModel::new(p, prior_precision, 1000);

    // Apply 50 updates — enough for meaningful drift measurement.
    for i in 0..50 {
        let r_rho = if i % 4 == 0 { 1.0 } else { -1.0 };
        let phi: Vec<f64> = (0..p).map(|j| ((i * 23 + j * 13) % 100) as f64 / 50.0 - 1.0).collect();
        let phi_hat = vec_to_col(&phi);

        let (v, h) = model.covariance().quadratic_form_with_product(&phi_hat);
        let w_eff = compute_effective_weight(1.0, 100.0, 5.0, h, 1e-8);
        model.apply_leverage_bounded_update(&phi_hat, &v, h, w_eff, r_rho, 0.9998, 1e-4);
    }

    // Full ‖BΣ − I‖_F check
    let dev = frobenius_deviation_from_identity(model.precision(), model.covariance());
    assert_below(dev, 1e-10, "‖BΣ − I‖_F at p=15 after 50 updates");

    // Σ_new φ̂ identity still holds after many updates
    let phi_test: Vec<f64> = (0..p).map(|j| (j as f64 + 1.0) / 15.0).collect();
    let phi_hat_test = vec_to_col(&phi_test);
    let gamma_eff = 0.9998;
    let w_eff = 0.3;

    let mut model_copy = BayesianLinearModel::new(p, prior_precision, 1000);
    // Replay same 50 updates to get identical state
    for i in 0..50 {
        let r_rho_replay = if i % 4 == 0 { 1.0 } else { -1.0 };
        let phi_r: Vec<f64> = (0..p).map(|j| ((i * 23 + j * 13) % 100) as f64 / 50.0 - 1.0).collect();
        let phi_hat_r = vec_to_col(&phi_r);
        let (v_r, h_r) = model_copy.covariance().quadratic_form_with_product(&phi_hat_r);
        let w_eff_r = compute_effective_weight(1.0, 100.0, 5.0, h_r, 1e-8);
        model_copy.apply_leverage_bounded_update(&phi_hat_r, &v_r, h_r, w_eff_r, r_rho_replay, 0.9998, 1e-4);
    }
    // Now do one more update on the copy to test the identity
    let (v, h) = model_copy.covariance().quadratic_form_with_product(&phi_hat_test);
    model_copy.apply_leverage_bounded_update(&phi_hat_test, &v, h, w_eff, 0.0, gamma_eff, 1e-4);

    let expected_scale = 1.0 / (gamma_eff + w_eff * h);
    let sigma_phi = model_copy.covariance().symmetric_matvec(&phi_hat_test);
    for i in 0..p {
        let expected = v[i] * expected_scale;
        assert!(
            (sigma_phi[i] - expected).abs() < 1e-10,
            "Σφ̂[{i}]: expected {expected}, got {}",
            sigma_phi[i]
        );
    }
}

/// A leverage read against a covariance that has lost positive semi-definiteness is negative, and the bound divides by it, so the weight it produces is negative rather than small — the harm a leverage bound exists to prevent, arriving through the bound itself. The update refuses the reading in every build profile and mutates nothing: the mean, the covariance and the precision all stand where they stood, and the label counter has not advanced. Applying at a negative weight would subtract information the model never received, carrying the precision matrix further from the property whose loss the reading was reporting.
///
/// ´claim:bayes:a-negative-leverage-is-refused-and-the-model-is-left-untouched´
/// ´test:crate:a-negative-leverage-refuses-the-update-and-leaves-the-model-where-it-stood´
#[test]
fn a_negative_leverage_refuses_the_update_and_leaves_the_model_where_it_stood() {
    use crate::model::bayesian::LabelUpdateOutcome;

    let mut model = BayesianLinearModel::new(4, 0.5, 1000);
    let phi_hat = vec_to_col(&[1.0, -0.5, 0.25, 2.0]);
    let (v, _h_true) = model.covariance().quadratic_form_with_product(&phi_hat);

    let before = model.to_parameters();
    let before_precision: Vec<f64> = (0..model.dim()).map(|i| model.precision().diagonal_element(i)).collect();
    let before_labels = model.labels_since_recompute();

    // The reading a covariance that is no longer one produces, and the weight
    // the bound then computes from it.
    let h = -1e-3;
    let w_eff = compute_effective_weight(
        1.0,
        DEFAULT_IMPORTANCE_CEILING,
        DEFAULT_LEVERAGE_SAFETY_FACTOR,
        h,
        EPSILON_LEVERAGE,
    );
    assert!(
        w_eff < 0.0,
        "the bound turns a negative leverage into a negative weight rather than a small one: {w_eff}"
    );

    let outcome = model.apply_leverage_bounded_update(&phi_hat, &v, h, w_eff, 1.0, 0.999, 1e-3);
    assert_eq!(
        outcome,
        LabelUpdateOutcome::RefusedIndefiniteCovariance,
        "the update reports the refusal to its caller"
    );

    let after = model.to_parameters();
    assert_eq!(after.mu, before.mu, "the mean stands where it stood");
    assert_eq!(
        after.covariance_data, before.covariance_data,
        "the covariance stands where it stood"
    );
    let after_precision: Vec<f64> = (0..model.dim()).map(|i| model.precision().diagonal_element(i)).collect();
    assert_eq!(
        after_precision, before_precision,
        "the precision diagonal was neither scaled nor floored"
    );
    assert_eq!(
        model.labels_since_recompute(),
        before_labels,
        "a refused label is not a label the model has processed"
    );
}

/// Across three fixed configurations — differing in prior, weight, target
/// sign, forgetting factor, and floor — the update lands on the values the
/// explicit formulas predict, in the mean, in the precision diagonal, and in
/// the covariance-feature identity, with the inverse pairing intact. Fixing
/// known inputs to known outputs is what catches a refactor that changes the
/// answer while leaving every self-consistency property satisfied.
///
/// ´claim:bayes:the-update-lands-on-independently-derived-values-across-fixed-configurations´
/// ´test:crate:golden-vectors´
#[test]
fn golden_vectors() {
    struct Golden {
        p: usize,
        prior_precision: f64,
        phi: Vec<f64>,
        w_eff: f64,
        r_rho: f64,
        gamma_eff: f64,
        lambda_floor: f64,
    }

    let configs = [
        // Config 1: Small dimension, moderate weight
        Golden {
            p: 3,
            prior_precision: 1.0,
            phi: vec![1.0, 0.0, 0.0],
            w_eff: 2.0,
            r_rho: 1.0,
            gamma_eff: 0.999,
            lambda_floor: 1e-4,
        },
        // Config 2: Unit weight, negative target
        Golden {
            p: 3,
            prior_precision: 0.5,
            phi: vec![0.0, 1.0, 0.0],
            w_eff: 1.0,
            r_rho: -1.0,
            gamma_eff: 0.995,
            lambda_floor: 1e-3,
        },
        // Config 3: Multi-component feature
        Golden {
            p: 3,
            prior_precision: 0.1,
            phi: vec![0.6, 0.8, 0.0],
            w_eff: 0.5,
            r_rho: 0.0,
            gamma_eff: 0.9998,
            lambda_floor: 1e-4,
        },
    ];

    for (idx, cfg) in configs.iter().enumerate() {
        let mut model = BayesianLinearModel::new(cfg.p, cfg.prior_precision, 1000);
        let phi_hat = vec_to_col(&cfg.phi);

        // Compute expected analytically (φ̂ is axis-aligned or simple).
        let (v, h) = model.covariance().quadratic_form_with_product(&phi_hat);

        // Expected B after steps 3–5
        let b_expected: Vec<f64> = (0..cfg.p)
            .map(|j| {
                let decayed = (cfg.gamma_eff * cfg.prior_precision).max(cfg.lambda_floor);
                decayed + cfg.w_eff * cfg.phi[j] * cfg.phi[j]
            })
            .collect();

        // Expected Σ diagonal (for diagonal initial Σ with axis-aligned
        // or simple φ̂, we verify via the formula).
        let denom = cfg.gamma_eff + cfg.w_eff * h;

        // Expected μ
        let residual = cfg.r_rho; // μ_0 = 0, so residual = r - 0
        let coeff = cfg.w_eff * residual / denom;
        let mu_expected: Vec<f64> = (0..cfg.p).map(|i| coeff * v[i]).collect();

        model.apply_leverage_bounded_update(&phi_hat, &v, h, cfg.w_eff, cfg.r_rho, cfg.gamma_eff, cfg.lambda_floor);

        // Verify B diagonal
        for (j, &b_exp) in b_expected.iter().enumerate() {
            assert!(
                (model.precision().diagonal_element(j) - b_exp).abs() < 1e-14,
                "Config {idx}: B[{j},{j}] expected {b_exp}, got {}",
                model.precision().diagonal_element(j)
            );
        }

        // Verify μ
        for (j, &mu_exp) in mu_expected.iter().enumerate() {
            assert!(
                (model.mu()[j] - mu_exp).abs() < 1e-14,
                "Config {idx}: μ[{j}] expected {mu_exp}, got {}",
                model.mu()[j]
            );
        }

        // Verify Σ_new φ̂ = v / denom (the identity)
        let sigma_phi = model.covariance().symmetric_matvec(&phi_hat);
        for j in 0..cfg.p {
            let expected = v[j] / denom;
            assert!(
                (sigma_phi[j] - expected).abs() < 1e-13,
                "Config {idx}: Σφ̂[{j}] expected {expected}, got {}",
                sigma_phi[j]
            );
        }

        // Verify dual tracking: ‖BΣ − I‖_F < tol
        let dev = frobenius_deviation_from_identity(model.precision(), model.covariance());
        assert_below(dev, 1e-10, &format!("Config {idx}: ‖BΣ − I‖_F"));
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Adoption by measurement (´dec:posterior:measured-adoption´)
// ─────────────────────────────────────────────────────────────────────────────

/// A precision matrix carried a long way by cheap rank-one updates has a
/// maintained covariance that has drifted from its inverse, and the rebuild
/// both repairs that and is adopted for it: the drift measured after the
/// rebuild is more than an order of magnitude below the drift measured before
/// it — a factor near twenty-five at this width and this many labels — and
/// both readings sit under the width-scaled threshold, so the covariance the
/// rebuild offered is the one the model ends up holding. Adoption on the
/// measurement rather than on the factorisation's verdict is what makes the
/// test mean anything, so the case where the rebuild genuinely helps has to
/// pass it.
///
/// ´claim:bayes:a-rebuild-that-repairs-a-drifted-covariance-is-adopted-on-its-own-measurement´
/// ´test:crate:a-drifted-covariance-is-repaired-and-the-rebuild-adopted´
#[test]
fn a_drifted_covariance_is_repaired_and_the_rebuild_adopted() {
    use crate::model::recompute::{RebuildVerdict, VisitInputs, synchronisation_error, synchronisation_visit};

    let p = 24;
    let mut model = BayesianLinearModel::new(p, 0.1, 1_000_000);

    // Nearly collinear features, so that the accumulated precision matrix
    // stretches the way a real geometry stretches it rather than staying
    // isotropic. The sequence is a small deterministic recurrence: the test
    // needs a fixed trajectory, not a random one.
    let mut state = 1_u64;
    for _ in 0..4_000 {
        let phi: Vec<f64> = (0..p)
            .map(|i| {
                state = state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                #[allow(clippy::cast_precision_loss)] // A 16-bit draw is exact in f64.
                let jitter = ((state >> 33) % 1_000) as f64 * 1e-6;
                // Three informative directions; every other coordinate is a
                // near-duplicate of the first, which is the collinearity the
                // geometry accumulates.
                if i < 3 { 1.0 + jitter } else { 0.25 + jitter }
            })
            .collect();
        let phi_hat = vec_to_col(&phi);
        let (v, h) = model.covariance().quadratic_form_with_product(&phi_hat);
        let w_eff = compute_effective_weight(
            1.0,
            DEFAULT_IMPORTANCE_CEILING,
            DEFAULT_LEVERAGE_SAFETY_FACTOR,
            h,
            EPSILON_LEVERAGE,
        );
        model.apply_leverage_bounded_update(&phi_hat, &v, h, w_eff, 1.0, 0.9995, 1e-3);
    }

    let threshold = synchronisation_error_threshold(p);
    let before = synchronisation_error(model.precision(), model.covariance());
    let inputs = VisitInputs {
        labels_since_recompute: model.labels_since_recompute(),
        diagonal_ratio: 1.0,
        dimensions_at_floor: 0,
        sync_error_threshold: threshold,
        spectral_floor_mass: 0.0,
        // No clamp mass: this rebuild is about rounding, and a model with
        // nothing at the floor attributes none of its drift to the prior.
        clamp_mass: &[],
    };
    let (outcome, new_covariance, baseline) =
        synchronisation_visit(model.precision(), model.covariance(), model.baseline(), true, &inputs);

    assert_eq!(
        baseline.verdict().expect("the visit rebuilt"),
        RebuildVerdict::Clean,
        "the drifted matrix still factors plainly"
    );
    assert!(
        baseline.adopted().expect("the visit rebuilt"),
        "a rebuild that repairs the drift is adopted: before {before:e}, after {:e}, threshold {threshold:e}",
        baseline.sync_error_after().expect("the visit rebuilt")
    );
    assert!(new_covariance.is_some(), "an adopted rebuild hands back its pair");
    assert_below(
        baseline.sync_error_after().expect("the visit rebuilt"),
        threshold,
        "the drift after the rebuild is under the width-scaled threshold",
    );
    assert!(
        baseline.sync_error_after().expect("the visit rebuilt") * 10.0 < baseline.sync_error_before,
        "the rebuild leaves the drift more than an order below where it found it: before {:e}, after {:e}",
        baseline.sync_error_before,
        baseline.sync_error_after().expect("the visit rebuilt")
    );

    // The model installs it, and the reading it then carries is the repaired one.
    model.apply_recompute_outcome(&outcome, new_covariance, Some(baseline), 1_000_000);
    let after_install = synchronisation_error(model.precision(), model.covariance());
    assert_below(
        after_install,
        threshold,
        "the installed covariance carries the repaired reading",
    );
}

/// The floor is applied where it is measured. A rebuild on a matrix whose
/// least eigenvalue has fallen below the floor installs a precision matrix
/// whose least eigenvalue is exactly the floor, together with the covariance
/// taken from that same matrix — so the pair is consistent and the monitor's
/// reading after the rebuild is machine-level rather than the size of the
/// floor. Between rebuilds nothing is added: the least eigenvalue decays with
/// everything else, by a positive factor that preserves definiteness exactly
/// and by no more than the decay the interval allows, which the next rebuild's
/// own measurement absorbs.
///
/// The alternative this test exists to exclude is an increment applied per
/// label to the precision matrix alone. That is arithmetically identical at
/// the precision matrix and catastrophic at the monitor, because the
/// covariance does not follow it: the pair then disagrees by the increment
/// times the covariance, which at the floor's magnitude is of order one per
/// label, and the cadence reads its own floor as drift.
///
/// ´claim:bayes:a-rebuild-installs-a-floored-precision-matrix-and-its-own-inverse-together´
/// ´test:crate:the-rebuild-applies-the-floor-and-leaves-the-pair-consistent´
#[test]
fn the_rebuild_applies_the_floor_and_leaves_the_pair_consistent() {
    use crate::linalg::bridge::eigenvalue_min_max;
    use crate::model::recompute::{VisitInputs, synchronisation_visit};
    use crate::testing::assert_near;

    let p = 8;
    // A matrix whose weakest direction has run far below anything the
    // coordinatewise clamp bounds, which is the state the drift study found at
    // width and the state the floor exists for.
    let mat = faer::Mat::from_fn(p, p, |i, j| {
        if i == j {
            if i == 0 {
                300.0
            } else if i == 1 {
                1e-18
            } else {
                1.0
            }
        } else {
            0.0
        }
    });
    let precision = SymmetricMatrix::from_computation(mat);
    let covariance = SymmetricMatrix::identity_scaled(p, 1.0);

    let inputs = VisitInputs {
        labels_since_recompute: 1000,
        diagonal_ratio: 1.0,
        dimensions_at_floor: 0,
        sync_error_threshold: synchronisation_error_threshold(p),
        spectral_floor_mass: 0.0,
        // No clamp mass: this rebuild is about rounding, and a model with
        // nothing at the floor attributes none of its drift to the prior.
        clamp_mass: &[],
    };
    let (outcome, rebuilt, baseline) = synchronisation_visit(&precision, &covariance, None, true, &inputs);

    let spectrum = baseline.spectrum().expect("the rebuild read a spectrum");
    assert!(spectrum.deficit > 0.0, "the weakest direction was below the floor");
    let pair = rebuilt.expect("the floored pair is adopted");

    // The precision matrix the model is handed carries the floor.
    let (_, floored_min) = eigenvalue_min_max(&pair.precision).expect("the floored spectrum");
    assert!(
        floored_min >= spectrum.lambda_floor * (1.0 - 1e-9),
        "the rebuild installs a least eigenvalue at the floor: {floored_min:e} against {:e}",
        spectrum.lambda_floor
    );
    assert!(
        floored_min <= spectrum.lambda_floor * (1.0 + 1e-9),
        "and not above it: {floored_min:e} against {:e}",
        spectrum.lambda_floor
    );

    // The covariance is that matrix's inverse, so the monitor reads nothing.
    let residual = crate::model::recompute::synchronisation_error(&pair.precision, &pair.covariance);
    assert_below(
        residual,
        inputs.sync_error_threshold,
        "the pair the rebuild installs is consistent",
    );

    // The model holds both halves, and the identity-shaped share the prior now
    // has in the matrix is the deficit the floor just added.
    let mut model = BayesianLinearModel::new(p, 0.1, 1_000_000);
    let deficit = spectrum.deficit;
    model.apply_recompute_outcome(&outcome, Some(pair), Some(baseline), 1_000_000);
    assert_near(
        model.spectral_floor_mass(),
        deficit,
        deficit * 1e-12,
        "the floor's own contribution is what the mass records",
    );
    let held = crate::model::recompute::synchronisation_error(model.precision(), model.covariance());
    assert_below(held, inputs.sync_error_threshold, "and the model holds a consistent pair");

    // Between rebuilds the least eigenvalue decays with everything else, which
    // is a positive scaling and therefore preserves definiteness exactly.
    let gamma = 0.9995_f64;
    let labels = 200;
    let phi_hat = vec_to_col(&vec![0.0; p]);
    for _ in 0..labels {
        let (v, h) = model.covariance().quadratic_form_with_product(&phi_hat);
        model.apply_leverage_bounded_update(&phi_hat, &v, h, 0.0, 0.0, gamma, 1e-30);
    }
    let (_, decayed_min) = eigenvalue_min_max(model.precision()).expect("the decayed spectrum");
    assert!(decayed_min > 0.0, "decay never leaves the definite cone");
    assert!(
        decayed_min >= floored_min * gamma.powi(labels) * (1.0 - 1e-6),
        "and it falls by no more than the decay the interval allows: {decayed_min:e} against {:e}",
        floored_min * gamma.powi(labels)
    );
}

/// The two shares of the precision matrix the prior is holding up are tracked
/// rather than inferred, and each behaves like the part of the matrix it
/// describes. The identity-shaped share takes the deficit a rebuild adds and
/// then decays with the matrix it is part of, reaching the closed form the
/// decay implies over the labels that follow, because it is scaled by the same
/// factor the matrix is scaled by on every one of them. The
/// coordinate-shaped share rides alongside the matrix through a structural
/// change: marginalising a coordinate takes that coordinate's accumulated
/// clamp with it and leaves the survivors' in the order the survivors are now
/// in. A floor whose size is known and whose contribution is not would be only
/// half a declaration, so both have to be carried rather than recomputed from
/// a matrix that no longer distinguishes prior from evidence.
///
/// ´claim:bayes:the-prior-masses-follow-the-precision-matrix-through-decay-and-marginalisation´
/// ´test:crate:the-prior-masses-follow-the-precision-matrix´
#[test]
fn the_prior_masses_follow_the_precision_matrix() {
    use crate::model::recompute::{VisitInputs, synchronisation_visit};
    use crate::testing::{DEFAULT_TOLERANCES, assert_near};

    let p = 6;
    let gamma = 0.5;
    let lambda_floor = 1e-30;
    let labels = 20;

    let mut model = BayesianLinearModel::new(p, 1.0, 1_000_000);
    assert!(
        (model.spectral_floor_mass() - 0.0).abs() < f64::EPSILON,
        "a fresh model has no prior share beyond its own construction"
    );

    // A rebuild on a matrix below the floor adds the deficit to the share.
    let mat = faer::Mat::from_fn(p, p, |i, j| {
        if i == j {
            if i == 0 {
                300.0
            } else if i == 1 {
                1e-18
            } else {
                1.0
            }
        } else {
            0.0
        }
    });
    let below = SymmetricMatrix::from_computation(mat);
    let inputs = VisitInputs {
        labels_since_recompute: 1000,
        diagonal_ratio: 1.0,
        dimensions_at_floor: 0,
        sync_error_threshold: synchronisation_error_threshold(p),
        spectral_floor_mass: model.spectral_floor_mass(),
        // No clamp mass: this rebuild is about rounding, and a model with
        // nothing at the floor attributes none of its drift to the prior.
        clamp_mass: &[],
    };
    let (outcome, rebuilt, baseline) =
        synchronisation_visit(&below, &SymmetricMatrix::identity_scaled(p, 1.0), None, true, &inputs);
    let deficit = baseline.spectrum().expect("a spectrum").deficit;
    assert!(deficit > 0.0, "the rebuild had a floor to apply");
    model.apply_recompute_outcome(&outcome, rebuilt, Some(baseline), 1_000_000);
    assert_near(
        model.spectral_floor_mass(),
        deficit,
        deficit * 1e-12,
        "the identity-shaped share takes the deficit the rebuild added",
    );

    // Between rebuilds it decays with the matrix it is part of, and nothing
    // adds to it.
    let phi_hat = vec_to_col(&vec![0.0; p]);
    for _ in 0..labels {
        let (v, h) = model.covariance().quadratic_form_with_product(&phi_hat);
        model.apply_leverage_bounded_update(&phi_hat, &v, h, 0.0, 0.0, gamma, lambda_floor);
    }
    let expected = deficit * gamma.powi(labels);
    assert_near(
        model.spectral_floor_mass(),
        expected,
        expected * 1e-9,
        "the identity-shaped share decays with the matrix and takes nothing on",
    );

    // Give the coordinates distinguishable clamp masses by clamping at a floor
    // that binds, so the survivors can be told apart after the removal.
    let binding_floor = model.precision().as_inner()[(0, 0)] * 2.0;
    let (v, h) = model.covariance().quadratic_form_with_product(&phi_hat);
    model.apply_leverage_bounded_update(&phi_hat, &v, h, 0.0, 0.0, gamma, binding_floor);
    let before: Vec<f64> = model.clamp_mass().to_vec();
    assert_eq!(before.len(), p);
    assert!(before.iter().all(|&m| m > 0.0), "the clamp bound on every coordinate");

    // Marginalising the second coordinate drops its mass and leaves the rest.
    let removed = 1_usize;
    model.marginalise(&[removed], &SchurConfig::default(), ModelId::Operational);
    let after = model.clamp_mass();
    assert_eq!(after.len(), p - 1, "one coordinate left and took its entry with it");
    let survivors: Vec<f64> = (0..p).filter(|&i| i != removed).map(|i| before[i]).collect();
    for (index, (kept, expected_mass)) in after.iter().zip(survivors.iter()).enumerate() {
        assert_near(
            *kept,
            *expected_mass,
            DEFAULT_TOLERANCES.bit_identical,
            &format!("the surviving coordinate {index} keeps its own accumulated clamp"),
        );
    }
}

/// The synchronisation reading a model with exhausted coordinates carries has
/// a component the prior put there, and its size is the coordinate-shaped
/// clamp mass scaled by the covariance — exactly, not approximately. The
/// replenishment clamp raises a diagonal entry every label and the
/// Sherman–Morrison covariance does not follow it, so the tracked pair
/// disagrees by the clamp's accumulated contribution times the covariance,
/// which is what this test computes both ways and compares.
///
/// The consequence is the whole reason to record it. The monitor reads that
/// component as drift, and the cadence shortens the interval in response to a
/// quantity no recomputation can reduce below one interval's worth of it —
/// which is why a model with coordinates at the floor sits at the cadence
/// floor and reads a standing figure that scales with the interval rather than
/// with the width. It is conservative in direction, since the covariance is
/// wider than the inverse exactly along the directions the model has no
/// evidence for, so it is not a fault; it is a prior the reading cannot tell
/// apart from rounding.
///
/// ´claim:bayes:the-clamps-own-contribution-to-the-synchronisation-reading-is-the-clamp-mass-times-the-covariance´
/// ´test:crate:the-clamp-mass-predicts-its-own-contribution-to-the-reading´
#[test]
fn the_clamp_mass_predicts_its_own_contribution_to_the_reading() {
    use crate::model::recompute::synchronisation_error;
    use crate::testing::assert_near;

    let p = 6;
    let prior = 0.1;
    let lambda_floor = 0.05;
    let gamma = 0.9;
    let labels = 50;

    let mut model = BayesianLinearModel::new(p, prior, 1_000_000);
    let phi_hat = vec_to_col(&vec![0.0; p]);
    for _ in 0..labels {
        let (v, h) = model.covariance().quadratic_form_with_product(&phi_hat);
        model.apply_leverage_bounded_update(&phi_hat, &v, h, 0.0, 0.0, gamma, lambda_floor);
    }

    // The clamp bound on every coordinate, so every coordinate carries mass.
    // The monitor reads the mass since the last rebuild, and this model has
    // not rebuilt, so here that is the whole of what the clamp has added.
    let mass = model.clamp_mass_since_rebuild().to_vec();
    assert_eq!(mass, model.clamp_mass(), "before a rebuild the two masses are the same mass");
    assert!(mass.iter().all(|&m| m > 0.0), "the clamp bound on every coordinate");

    // The prediction: ‖diag(c)·Σ‖_F, the clamp's accumulated contribution seen
    // through the covariance. Under decay alone the precision matrix is its
    // decayed self plus exactly that mass, so the identity is exact rather
    // than asymptotic.
    let mut sum_sq = 0.0_f64;
    for (i, &row_mass) in mass.iter().enumerate() {
        for j in 0..p {
            let entry = row_mass * model.covariance().as_inner()[(i, j)];
            sum_sq = entry.mul_add(entry, sum_sq);
        }
    }
    let predicted = sum_sq.sqrt();

    let measured = synchronisation_error(model.precision(), model.covariance());
    assert!(predicted > 1.0, "the contribution is large at this floor and interval");
    assert_near(
        measured,
        predicted,
        predicted * 1e-9,
        "the reading is the clamp mass seen through the covariance",
    );
}

/// The two masses resolve to a share along a direction, and the share is a
/// division rather than an inference. A model whose floors have never acted
/// reports no share at all along any direction, so nothing downstream of the
/// reading can move on a healthy model. Once the clamp has bound, the share
/// along a coordinate is that coordinate's own clamp mass over that
/// coordinate's own posterior precision, which is a number a reader can work
/// out by hand from the two published quantities — here a fifth along the
/// coordinate an observation has since sharpened, and a half along the three
/// it has not.
///
/// The direction is what makes the reading useful and what makes it partial.
/// An aggregate degradation figure says a model is propped up somewhere; this
/// says whether it is propped up *here*, along the features of the request in
/// hand, which is the only form in which the fact can be attached to a verdict.
/// What it does not say is anything about the subspace the direction lies in,
/// because a Rayleigh quotient reads one direction and reports one number.
///
/// ´claim:bayes:the-borrowed-share-divides-the-tracked-masses-by-the-precision-along-the-direction-asked-about´
/// ´test:crate:the-borrowed-share-resolves-the-two-masses-to-a-direction´
#[test]
fn the_borrowed_share_resolves_the_two_masses_to_a_direction() {
    use crate::testing::{DEFAULT_TOLERANCES, assert_near};

    let p = 4;
    let prior = 1.0;
    let mut model = BayesianLinearModel::new(p, prior, 1_000_000);

    // A model whose floors have never acted holds no prior mass beyond its own
    // construction, so every direction reads zero — including directions that
    // are not coordinates.
    for index in 0..p {
        let mut axis = vec![0.0; p];
        axis[index] = 1.0;
        assert!(
            model.borrowed_share(&axis).abs() < f64::EPSILON,
            "a healthy model borrowed nothing along coordinate {index}"
        );
    }
    assert!(
        model.borrowed_share(&[1.0, -2.0, 0.5, 3.0]).abs() < f64::EPSILON,
        "and nothing along an oblique direction either"
    );

    // One update at unit decay: the clamp binds on every coordinate at a floor
    // above the prior, and the rank-one term then sharpens the first
    // coordinate alone. The precision is diagonal throughout, so the reading
    // along each coordinate is that coordinate's arithmetic and no other's.
    let binding_floor = 2.0;
    let sharpening = 3.0;
    let mut phi = vec![0.0; p];
    phi[0] = 1.0;
    let phi_hat = vec_to_col(&phi);
    let (v, h) = model.covariance().quadratic_form_with_product(&phi_hat);
    model.apply_leverage_bounded_update(&phi_hat, &v, h, sharpening, 0.0, 1.0, binding_floor);

    let clamp = model.clamp_mass().to_vec();
    let added = binding_floor - prior;
    for (index, &mass) in clamp.iter().enumerate() {
        assert_near(
            mass,
            added,
            DEFAULT_TOLERANCES.bit_identical,
            &format!("the clamp lifted coordinate {index} from the prior to the floor"),
        );
    }
    assert!(
        model.spectral_floor_mass().abs() < f64::EPSILON,
        "and no rebuild has run, so the identity-shaped mass is still nothing"
    );

    // Along the sharpened coordinate the clamp holds a fifth of a posterior
    // precision of five; along the others it holds half of two.
    let mut sharpened = vec![0.0; p];
    sharpened[0] = 1.0;
    assert_near(
        model.borrowed_share(&sharpened),
        added / (binding_floor + sharpening),
        1e-15,
        "the share along the sharpened coordinate is its clamp over its precision",
    );
    for index in 1..p {
        let mut axis = vec![0.0; p];
        axis[index] = 1.0;
        assert_near(
            model.borrowed_share(&axis),
            added / binding_floor,
            1e-15,
            &format!("and along coordinate {index} the evidence has added nothing to divide by"),
        );
    }

    // The reading is scale-free, because both of its terms are quadratic in
    // the direction: a direction is a direction whatever its length.
    let scaled = vec![-7.5, 0.0, 0.0, 0.0];
    assert_near(
        model.borrowed_share(&scaled),
        model.borrowed_share(&sharpened),
        DEFAULT_TOLERANCES.bit_identical,
        "the share depends on the direction and not on the vector's length",
    );

    // A vector of the wrong width is not a direction in this model's space.
    assert!(
        model.borrowed_share(&[1.0, 0.0]).abs() < f64::EPSILON,
        "a direction of the wrong width reads zero rather than a truncation"
    );
}

/// The repaired count counts the floor's repairs and nothing else: a rebuild whose least eigenvalue was short of the floor increments it, and a rebuild on a matrix the evidence already holds above the floor leaves it where it was. The counter is the one the retired repair cascade used to fill, and its meaning is unchanged — this model's precision needed help — because the help it now records is the floor's rather than a factorisation shift's. A counter left on the retired device could hold only zero, which is a constant reported where a measurement is promised, so the move is what keeps the reading honest rather than merely keeping a field alive.
///
/// ´claim:bayes:the-repaired-count-counts-the-floors-repairs-and-only-those´
/// ´test:crate:the-repaired-count-counts-the-floors-repairs´
#[test]
fn the_repaired_count_counts_the_floors_repairs() {
    use crate::model::recompute::{VisitInputs, synchronisation_visit};

    /// One rebuild of `precision`, applied to a fresh model; returns the
    /// model's repaired count and whether the rebuild found a deficit.
    fn rebuild(precision: &SymmetricMatrix) -> (u64, bool) {
        let p = precision.dim();
        let covariance = SymmetricMatrix::identity_scaled(p, 1.0);
        let inputs = VisitInputs {
            labels_since_recompute: 1000,
            diagonal_ratio: 1.0,
            dimensions_at_floor: 0,
            sync_error_threshold: synchronisation_error_threshold(p),
            spectral_floor_mass: 0.0,
            clamp_mass: &[],
        };
        let (outcome, rebuilt, baseline) = synchronisation_visit(precision, &covariance, None, true, &inputs);
        let repaired = baseline.floor_repaired();
        assert!(rebuilt.is_some(), "both matrices here rebuild cleanly and are adopted");
        let mut model = BayesianLinearModel::new(p, 0.1, 1_000_000);
        assert_eq!(model.floored_rebuilds(), 0, "a fresh model has repaired nothing");
        model.apply_recompute_outcome(&outcome, rebuilt, Some(baseline), 1_000_000);
        (model.floored_rebuilds(), repaired)
    }

    let p = 6;

    // A weakest direction far below anything the coordinatewise clamp bounds:
    // the floor has a deficit to apply, and the rebuild is a repair.
    let short = SymmetricMatrix::from_computation(faer::Mat::from_fn(p, p, |i, j| {
        if i != j {
            0.0
        } else if i == 0 {
            300.0
        } else if i == 1 {
            1e-18
        } else {
            1.0
        }
    }));
    let (counted, repaired) = rebuild(&short);
    assert!(repaired, "the weakest direction was short of the floor");
    assert_eq!(counted, 1, "a rebuild the floor repaired is counted");

    // The same shape with the weakest direction well clear of the floor: the
    // reading is taken, the deficit is nothing, and nothing is counted.
    let ample = SymmetricMatrix::from_computation(faer::Mat::from_fn(p, p, |i, j| {
        if i == j { if i == 0 { 300.0 } else { 1.0 } } else { 0.0 }
    }));
    let (uncounted, unrepaired) = rebuild(&ample);
    assert!(!unrepaired, "the evidence holds every direction above the floor");
    assert_eq!(uncounted, 0, "a rebuild with nothing to repair is not counted");
}
