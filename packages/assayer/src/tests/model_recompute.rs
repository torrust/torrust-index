// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`counter_trigger`] | bayes | Once as many labels have been absorbed as the effective interval allows, recomputation is called for on the strength of the count alone, and the decision carries both the count that reached the threshold and the conditioning estimate observed at the time. A perfectly conditioned matrix is still recomputed on schedule: the count exists to bound drift that conditioning does not reveal. |
//! | [`not_needed`] | bayes | Neither alarm sounding means no recomputation: a well-conditioned matrix part way through its interval is left alone, with the conditioning estimate still reported. Recomputation is a cubic-cost operation, so the default has to be to decline it and say why. |
//! | [`clean_recomputation`] | bayes | A precision that factorises without help yields a clean outcome: a fresh covariance is produced, no interval halving is called for, the cascade did not reach its terminus, and the mismatch measured beforehand was already negligible. Clean is the outcome the adaptive machinery treats as evidence that the model is healthy, so it must mean the unassisted factorisation genuinely succeeded rather than merely that nothing threw. |
//! | [`interval_floor_on_terminus`] | bayes | A refused factorisation takes the interval to its floor in one step rather than halving it, and clears the recovery streak with it. Halving is the proportional response to drift that is being removed; a refusal removes none, and it only happens now on a model whose measurement said the drift was worth acting on. The model is looked at as often as the cadence permits until a measurement comes back good. |
//! | [`needed_rebuild_shortens_interval`] | bayes | A rebuild that was needed halves the interval, resets recovery and is counted; a measurement that found nothing to do leaves all three alone. The comparison against the threshold no longer happens here — it happens at the measurement, and the outcome that reaches the cadence names the answer — so what this reads is that the cadence acts on the visit's verdict rather than re-deriving it. |
//! | [`interval_recovery`] | bayes | A shortened interval is not permanent: the good measurement that completes the required streak restores the configured interval outright. Without a way back, a single bad patch would leave the model paying for frequent visits for the rest of its life — and the recovery is now spent out of measurements, which is what lets a model that needed one rebuild climb back to its configured interval without paying for three more (´dec:posterior:adaptive-cadence´). |
//! | [`interval_floor`] | bayes | Repeated shortening stops at the floor rather than continuing down: a run of needed rebuilds takes the interval there, and the next reports no change at all. Below the floor a cubic-cost factorisation would be amortised over too few labels to be worth its price, so a persistently drifting model recomputes often but not without limit. |
//! | [`sync_error_identity`] | bayes | A precision and covariance that are exactly inverse register no disagreement at all. The measure has to read zero on a healthy pair, or every recomputation decision it feeds would be made against a standing offset rather than against real drift. |
//! | [`precision_sync_ceiling_scales_with_the_dimension`] | bayes | The synchronisation ceiling the harness holds a model to is the specification's threshold at that model's own width — the coefficient times the dimension — and not a flat figure. The distinction has teeth at both ends of the range the suite exercises: a five-wide model is held twenty times tighter than the flat ceiling that preceded it and a fifty-wide model twice as tight, while the two coincide only at a width of one hundred, which is what made the flat figure look adequate (´def:monitoring:synchronisation-error´). |
//! | [`sync_error_drift`] | bayes | The measure is quantitative, not merely a flag: a covariance stretched by a tenth on one coordinate registers a disagreement of that same tenth. Because the number tracks the size of the departure, thresholds on it mean something and a slowly worsening model can be watched rather than only caught. On a departure confined to one diagonal entry the norm and a maximum over the diagonal agree exactly, which is why this test held under both and why the interchange needed the test below to become visible. |
//! | [`sync_error_sees_off_diagonal_departure`] | bayes | A pair whose product has an exact unit diagonal and a large off-diagonal departure registers that departure. The measure is a norm over the whole product, and the norm and a maximum over the diagonal are different quantities that a restatement can silently interchange: here the diagonal reads perfect health while the two representations disagree by a half on four entries. Since the reading is what a recomputation interval would be shortened on, a measure blind to off-diagonal growth would leave the model on its long interval exactly while it drifted. |
//! | [`clean_increments_consecutive`] | bayes | Each good measurement lengthens the run of clean visits by exactly one, and a rebuild ends the run whatever its own result was. The streak counts consecutive visits that found nothing to do rather than merely recording that the last one did, and recovery of the interval is spent out of it — so a model that needed rebuilding is not one whose condition has passed, even where the rebuild succeeded. |
//! | [`outcome_accessors`] | bayes | Each of the three outcomes answers for itself whether a factorisation was paid for, whether the alarm was raised, whether the factorisation ran out of options, and what residual the measurement found. Callers branch on these answers rather than on the shape of the outcome, so the answers have to be right for all three and not only the common one — and the common one is now the measurement that rebuilt nothing. |
//! | [`the_spectral_floor_is_the_least_the_success_condition_admits`] | bayes | The floor a rebuild derives is exactly the least eigenvalue the Cholesky success condition demands at that width and that largest eigenvalue: a matrix held at the floor sits precisely at the condition's boundary, and a floor a shade lower puts it outside. Deriving the least sufficient value is what keeps the floor an arithmetic requirement rather than a modelling assertion, so a floor merely sufficient would be claiming confidence no factorisation needs. |
//! | [`the_spectral_floor_follows_the_evidence_down`] | bayes | Halving the largest eigenvalue halves the floor derived from it, and the derivation reads nothing else about the spectrum. The floor tracks the evidence in both directions because a floor that could only rise would assert a confidence that returning evidence has already refuted. |
//! | [`the_floor_share_is_dimensionless_monotone_and_tends_to_zero`] | bayes | Scaling the floor and the least eigenvalue together leaves the degradation reading unchanged, arriving evidence drives it toward zero, a least eigenvalue equal to the floor reads one, and a direction outside the definite cone reads one as well. The reading is meant to be comparable across models of different scales, so cancelling the units is the property that makes a maximum across models mean anything. |
//! | [`the_deficit_is_what_the_least_eigenvalue_is_short_of_the_floor`] | bayes | The deficit a reading hands the rebuild is exactly what the least eigenvalue is short of the floor, and nothing where the evidence already clears it. Adding that much of the identity shifts every eigenvalue by it, so the least lands on the floor and no eigenvalue is raised further than the least one needed — which is what makes the floor the smallest anti-conservative claim the arithmetic admits rather than a convenient one. |
//! | [`an_addition_the_covariance_does_not_follow_is_measured_as_drift`] | bayes | An addition to the precision matrix that the covariance does not follow is read by the synchronisation monitor as drift, and the reading is exactly the size of the addition scaled by the covariance. This is the arithmetic that decides where the spectral floor is applied: a per-label increment to the precision matrix alone would enter the monitor's reading at `δ·‖Σ‖`, which at the floor's own magnitude is of order one per label, so the floor would manufacture the very drift the monitor exists to detect and the cadence would shorten the interval in response to it. Applying the floor where the covariance is rebuilt costs nothing extra and leaves the pair consistent (´dec:posterior:spectral-floor´). |
//! | [`a_rebuild_that_is_not_better_is_refused`] | bayes | An indefinite matrix reaches the rebuild and the rebuild offers nothing: the factorisation refuses it, the baseline records the refusal, no covariance comes back, and the model keeps what it had. The retired second phase answered here with a success carrying a replacement count, and the rebuild had to refuse that answer on its own measurement; there is no answer to refuse now. The floor declines to act, which is the other half of the reading — a matrix outside the definite cone is a verdict and not a condition to floor. |
//! | [`a_rebuild_not_adopted_shortens_the_interval`] | bayes | A rebuild the model declined takes the interval to its floor in one step and clears the recovery streak with it, where a good measurement leaves both alone. The decline is the alarm now: a rebuild runs only where the measurement found drift worth acting on, so a rebuild whose answer was not an improvement is a model whose drift is real and whose fresh inverse is worse than the pair it holds. Halving would be a proportional response to drift being removed, and none is being removed (´rep:assayer:declined-rebuild-cadence´). |
//! | [`conditioning_alone_never_calls_for_a_rebuild`] | bayes | A diagonal ratio of a million million, on a matrix whose counter is nowhere near its interval, calls for nothing. The retired arm tested that ratio against a fixed threshold and the replenishment clamp pins the ratio's denominator at the floor, so once any dimension rested there the arm was satisfied on every label for ever — and the rebuild it called for cannot change the precision matrix the ratio is read from, so the arm asked for a cubic factorisation per label to no effect. Nothing absolute about the conditioning fires now, whether or not a baseline exists. |
//! | [`an_adopted_rebuild_leaves_the_check_quiet`] | bayes | Immediately after a rebuild the model adopted, the check on the very same matrix says no rebuild is needed. This is the property the retired arm did not have and the whole reason the check is relative: every quantity it tests is one the rebuild has just reset, so a rebuild cannot leave the engine asking for another. A check that could would spend a cubic factorisation per label for ever, which is what the shipped engine did. |
//! | [`the_relative_arms_fire_on_their_own_quantities`] | bayes | The three relative arms each fire on their own quantity and on nothing else: the ratio doubling past its baseline, the floored-dimension count moving off the baseline's, and the label counter reaching the interval. A growth arm that fired on a ratio below its factor, or a floored-count arm that fired on an unchanged count, would put the engine back on the cadence the fixed threshold produced. |
//! | [`the_floor_and_the_reading_never_ratchet`] | bayes | Two rebuilds in succession, the second on a matrix whose evidence has returned: the floor the second derives is below the floor the first derived, it asks for nothing where the first had to act, and the degradation reading falls with both. Nothing carries the earlier floor forward, because a floor that could only rise would keep asserting a confidence the returning evidence has already refuted — and the reading published beside it would go on reporting a degradation the model no longer has. |
//! | [`a_floored_geometry_does_not_have_its_rebuilds_refused`] | bayes | A rebuild on the shape the drift study drove — a definite matrix with a band of coordinates resting at the replenishment floor and a conditioning the floor itself permits — is adopted, and the numbers it was judged on are carried in the failure message so that a refusal here is a diagnosis rather than a surprise. This is the adoption test's own falsifier: a measurement that refuses a healthy rebuild leaves the maintained covariance in place for ever, and a model whose covariance is never replaced is worse off than one that rebuilds too often. The geometry matters. The floor bounds the condition number at the largest value a floating-point factorisation is assured to survive, and the residual a correct inverse of a matrix at that conditioning carries is itself proportional to that conditioning. So the two bounds have to be compatible: if the drift threshold a rebuild is judged against is tighter than the residual the arithmetic can deliver at the conditioning the floor permits, every rebuild at that conditioning is refused however healthy it is. |
//! | [`the_prior_induced_component_is_the_mass_accumulated_since_the_rebuild`] | bayes | The component the prior put into the reading is the clamp mass accumulated since the last rebuild, and the mass the model tracks over its life is a different and larger number once a rebuild has happened. The model here is driven under decay alone, so the clamp is the only thing adding to the precision matrix and every disagreement in the tracked pair is the clamp's doing. Before any rebuild the two masses coincide and the closed form is exact against the reading. The rebuild then takes the covariance from the precision matrix the clamp has already raised, which absorbs everything accumulated up to that point and leaves the pair consistent; the reading restarts from zero while the tracked mass does not. Driven on from there, the reading is predicted exactly by what the clamp has added since — and the lifetime mass overstates it by half again at this decay and this interval, which is the error a monitor subtracting the wrong mass would make. The consequence is the reason to pin it: the residual the cadence acts on is the reading minus this component, so a component that overstates drives the residual to zero and the cadence stops reacting to rounding at all on exactly the models that carry a clamp. |
//! | [`a_model_whose_drift_is_all_prior_keeps_its_interval`] | bayes | A model whose drift is entirely the prior's doing keeps its interval and pays for no factorisation at all, and the rebuild it is made to take anyway leaves both components at zero. This is the behaviour the separation exists to produce, and the visit is what completes it. Driven under decay alone, every coordinate rests at the replenishment floor and the clamp is the only thing adding to the precision matrix, so the reading the monitor takes is large and none of it is rounding. The measurement takes that reading, subtracts the prior's part, finds the residual under the threshold, and stops — the model keeps its interval and its pair, and no spectrum is read and no matrix factored. The arrangement this replaces did the subtraction too and reached the same verdict about the interval, but reached it *after* paying for the factorisation the verdict said was unnecessary (´rep:assayer:declined-rebuild-cadence´). The forced rebuilds are the other half of the claim. Once a rebuild has been adopted the covariance is the inverse of the precision matrix the clamp has already raised, so there is nothing left for either component to be: the measured reading, the part attributed to the prior, and the residual are all at zero together. That is what makes the attribution safe to publish — it does not leave a standing figure behind on a model that has just been rebuilt. |
//! | [`a_model_whose_drift_is_all_rounding_shortens_as_before`] | bayes | A model whose drift is entirely rounding has a residual equal to its whole reading, and the cadence shortens on it exactly as it did before. The falsifier for the test above. Separating the components is only worth doing if it leaves the case the monitor exists for untouched, so here the covariance is stretched off the inverse by a tenth on one coordinate with no clamp mass anywhere: the prior's component is exactly zero, the residual is the whole reading, and the interval halves. Had the subtraction been wrong in the other direction — attributing to the prior something the arithmetic put there — this is where it would show, and the cost would be a model drifting on a long interval with nothing to say so. |
//! | [`the_studys_reading_is_the_prior_and_its_residual_is_rounding`] | bayes | At the drift study's own geometry the reading a floored model publishes is the prior's in full, its residual is rounding, and the closed form predicts the study's measured figures at both cadences from the interval alone. This is the reproduction that makes the study's numbers readable. The geometry is the one the sweep drives: twenty-six coordinates resting at the replenishment floor with no evidence arriving on them, the rest taking evidence, at the outcome-axis model's own forgetting rate. Because the unevidenced coordinates form their own block, the covariance over that block is exactly the reciprocal of the floor and the arithmetic closes in hand: the clamp contributes `λ_floor·(1 − γ^k)` per coordinate since the rebuild, the covariance over the block has grown by `γ^{-k}` since it, the floor cancels between the two, and the reading is `√n · (1 − γ^k) · γ^{-k}` for `n` floored coordinates. That expression is the whole content of the study's readings. At an interval of a hundred labels it gives 8.83, which is what the sweep measured on the outcome-axis model while its cadence was pinned at the floor; at four hundred it gives 278, which is what the same model measures once the cadence stops shortening on a quantity it cannot remove. The reading grew because the interval did, and both figures are the prior's. The floor's own value does not appear, which is why sweeping the floor across two decimal orders left the error where it was. The direction matters as much as the size. A large figure here is a model leaning on its prior along directions no evidence has reached, which is conservative — the covariance is wider than the inverse exactly there — and it is not drift. What an operator reads for drift is the residual, and at this geometry the residual is the accumulated rounding of the maintenance and sits below the threshold by orders. A monitor publishing only the measured figure cannot tell the two apart, which is the whole reason the components are separated. |
//! | [`an_isolated_model_at_the_schedules_conditioning_adopts_every_rebuild`] | bayes | An isolated model driven to a synchronised state at the dense schedule's own conditioning adopts every rebuild, and the fresh inverse is the better of the two readings by a factor of three. This is the falsifier for the reading the dense schedule gives, and it falsifies the easy explanation of it. That schedule shows its operational and sister models declining nine of every ten rebuilds while the drift they measured beforehand sits three orders under the threshold the cadence tests, which pins their interval at the adaptive floor and throws away a cubic factorisation every hundred labels. The obvious reading is that declines are what a model at that conditioning does: the maintained pair's drift and a fresh inverse's own rounding residual arrive at the same magnitude, the comparison between them is decided by rounding noise, and roughly half of them are lost. That reading is wrong, and this is where it is shown to be wrong. The model here is driven through the real update path at the operational family's own forgetting rate, with a four-decade geometric spread across the feature magnitudes chosen so that the top of the precision diagonal settles near `1e5` and the bottom just above the replenishment floor. That is a diagonal ratio of `8.776e7` at a width of four hundred, which is the conditioning the schedule reports. Ten rebuilds at the adaptive floor's own interval follow, and every one of them is adopted: the drift standing before the last is `9.220e-12`, the drift the rebuilt pair carries is `4.096e-12`, the mean of the ratio over the ten is `0.318`, and the threshold at this width is `4.000e-4` — so both readings are under it by seven orders and the fresh inverse is the better one every time. The same measurement at widths two hundred, eight hundred and twelve hundred gives the same verdict, so the width is not what separates the two cases either. What that leaves is the finding. The declines are a property of the schedule's matrices rather than of the conditioning figure they publish, and the figure is the reason the two can come apart: the diagonal ratio is a lower bound on the condition number and not a measurement of it (´dec:posterior:recomputation-trigger´), so a matrix built from a competitive geometry — where a cell and the parent it was split from fire together, where the width grows between one rebuild and the next, and where coordinates are added at the prior while the schedule runs — can carry a true conditioning far above the ratio it reports while an isolated model at the same ratio does not. A rebuild's inverse is accurate to the true conditioning and not to the reported one, which is what puts the schedule's `after` above its `before` and this model's below. The measurement also says what the instrument cannot yet say. The adoption test declines on either of two conditions — the drift after the rebuild is not at or below the drift before it, or it is not under the threshold — and the published precision detail carries the drift before the last rebuild and not the drift after it, so a decline in the schedule's output cannot be attributed to one condition or the other. The model's own baseline holds both figures. |
//! | [`a_readings_resolution_is_read_off_its_operands`] | bayes | A reading's resolution is read off the operands and not off the answer, so a well-scaled pair reports a resolution no meaningful reading approaches while a cancelling pair reports one that swallows its own reading whole. This is the derivation's content. The bound on a floating-point matrix product is `\|fl(BΣ) − BΣ\| ≤ γ_p·\|B\|\|Σ\|` entrywise, so what limits the reading is the size of the products that were summed, not the size of what they summed to. Scaling the answer would give the same figure for both pairs here and would be wrong for both. |
//! | [`a_residual_at_its_readings_resolution_is_good`] | bayes | A residual at the resolution of the reading it was taken out of is published as such and is *not* thereby good: the flag is a reading and the threshold is the verdict. The refinement was proposed the other way round — an at-resolution residual would be good however far above the threshold it stood — and the falsifier vetoed that half of it. The resolution is computed from the operands, so a pair that has diverged by orders reports a resolution large enough to excuse any residual at all; the schedule's outcome-axis model reached a residual of `3.0e2` against a threshold of `1.2e-3` with the flag still true, and every measurement in that state called for nothing while the model's pair ceased to be an inverse pair (´rep:assayer:declined-rebuild-cadence´). What survives is the reading, which is true and worth publishing: this residual really is the rounding of the subtraction that produced it. |
//! | [`a_clamp_contribution_over_the_threshold_calls_for_a_rebuild`] | bayes | A clamp contribution over the model's threshold calls for a rebuild even where the residual is spotless, and a model carrying no clamp is untouched by that arm. The prior-induced component is the replenishment clamp's contribution since the last rebuild, and an adopted rebuild is the only thing that removes it. A measurement that looked only at the residual would therefore report a healthy model while the pair it holds drifted apart without limit — which is what the falsifier measured before this arm existed: a reading running from `1.7e2` to `2.0e10` across a single rebuild, no spectral floor taken for thousands of labels, and a refused factorisation that stopped the label path (´rep:assayer:declined-rebuild-cadence´). |
//! | [`a_good_measurement_rebuilds_nothing`] | bayes | A good measurement rebuilds nothing, records what it read, and leaves the interval where it was; a bad one rebuilds and its adopted result shortens the interval. The two halves are the restructure. The counters separate them — a measurement is not a recompute — and the baseline separates them too: a measurement writes no rebuild record, so the readings a reader sees beside one are the readings of the visit itself. |
//! | [`a_rebuild_that_is_no_better_raises_the_alarm`] | bayes | A rebuild that was needed and produced no improvement raises the alarm, keeps the pair the model already held, and takes the interval to its floor. The threshold arm of the adoption test is what fails here, and it is made to fail by asking the rebuild for a drift under zero — a demand no factorisation can meet. What the reading is about is not the arithmetic of the demand but the disposition: the model keeps its pair, the alarm counts, and the interval goes to the floor in one step rather than halving (´dec:posterior:measured-adoption´). |
//! | [`a_refused_factorisation_raises_the_alarm`] | bayes | A refused factorisation raises the alarm and is a terminus with it. The witness is a matrix outside the definite cone whose diagonal is positive, so the fast check does not divert it to the rebuild and the measurement is what sends it there: the drift on this pair is real, large, and nowhere near its own resolution. The factorisation then refuses, the model keeps its pair, and both readings are published — the alarm because nothing improved, the terminus because nothing was offered. |
//! | [`the_interval_recovers_under_measurements`] | bayes | The interval recovers under a run of good measurements and returns to the floor the moment one alarm fires. Recovery spent out of measurements is what lets a model that needed one rebuild climb back without paying for three more, and it is the half of the cadence the restructure changes most: under the arrangement it replaces the run was spent out of rebuilds, so a model could only earn its interval back by paying the cost the interval exists to avoid (´dec:posterior:adaptive-cadence´). |

//! Tests for the Cholesky recomputation strategy (´dec:posterior:recomputation-trigger´).
//!
//! # Cross-References
//!
//! - (´dec:posterior:recomputation-trigger´) — recomputation fires on a counter or on conditioning
//! - (´dec:posterior:always-refactor´) — and the covariance is refactored every time

use faer::Mat;

use crate::linalg::symmetric::SymmetricMatrix;
use crate::model::recompute::{
    CHOLESKY_SUCCESS_CONSTANT, DEFAULT_KAPPA_GROWTH_FACTOR, ModelPrecisionHealth, N_RECOMPUTE_FLOOR, N_RECOVERY_CLEAN,
    RebuildRecord, RebuildVerdict, RecomputeOutcome, RecomputeTrigger, SpectralReading, VisitInputs, compute_new_interval,
    floor_share, measure_synchronisation, product_error_growth, should_recompute, spectral_floor, synchronisation_error,
    synchronisation_error_threshold, synchronisation_reading, synchronisation_visit,
};
use crate::model::update::DEFAULT_LAMBDA_FLOOR;
use crate::testing::{DEFAULT_TOLERANCES, assert_below, assert_near};

// ═══════════════════════════════════════════════════════════════════════════════
// should_recompute Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// Once as many labels have been absorbed as the effective interval allows,
/// recomputation is called for on the strength of the count alone, and the
/// decision carries both the count that reached the threshold and the
/// conditioning estimate observed at the time. A perfectly conditioned matrix
/// is still recomputed on schedule: the count exists to bound drift that
/// conditioning does not reveal.
///
/// ´claim:bayes:the-label-count-reaching-the-effective-interval-calls-for-recomputation´
/// ´test:crate:counter-trigger´
#[test]
fn counter_trigger() {
    // 5×5 identity matrix (well-conditioned, κ = 1)
    let precision = SymmetricMatrix::identity_scaled(5, 1.0);

    // labels_since = 1000, n_effective = 1000 → should trigger
    let trigger = should_recompute(
        &precision,
        1000,
        1000,
        None,
        DEFAULT_KAPPA_GROWTH_FACTOR,
        DEFAULT_LAMBDA_FLOOR,
    );

    match trigger {
        RecomputeTrigger::Counter {
            labels, diagonal_ratio, ..
        } => {
            assert_eq!(labels, 1000);
            assert_near(
                diagonal_ratio,
                1.0,
                DEFAULT_TOLERANCES.bit_identical,
                "counter diagonal ratio at identity",
            );
        }
        _ => panic!("Expected Counter trigger, got {:?}", trigger),
    }
}

/// Neither alarm sounding means no recomputation: a well-conditioned matrix
/// part way through its interval is left alone, with the conditioning estimate
/// still reported. Recomputation is a cubic-cost operation, so the default has
/// to be to decline it and say why.
///
/// ´claim:bayes:a-well-conditioned-precision-part-way-through-its-interval-needs-no-recomputation´
/// ´test:crate:not-needed´
#[test]
fn not_needed() {
    let precision = SymmetricMatrix::identity_scaled(5, 1.0);

    // labels = 500, n_effective = 1000, κ = 1.0 < 1e5
    let trigger = should_recompute(&precision, 500, 1000, None, DEFAULT_KAPPA_GROWTH_FACTOR, DEFAULT_LAMBDA_FLOOR);

    match trigger {
        RecomputeTrigger::NotNeeded { diagonal_ratio, .. } => {
            assert_near(
                diagonal_ratio,
                1.0,
                DEFAULT_TOLERANCES.bit_identical,
                "not-needed diagonal ratio at identity",
            );
        }
        _ => panic!("Expected NotNeeded, got {:?}", trigger),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// synchronisation_visit Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// A precision that factorises without help yields a clean outcome: a fresh
/// covariance is produced, no interval halving is called for, the cascade did
/// not reach its terminus, and the mismatch measured beforehand was already
/// negligible. Clean is the outcome the adaptive machinery treats as evidence
/// that the model is healthy, so it must mean the unassisted factorisation
/// genuinely succeeded rather than merely that nothing threw.
///
/// ´claim:bayes:an-unassisted-factorisation-that-succeeds-yields-a-clean-outcome-with-a-rebuilt-covariance´
/// ´test:crate:clean-recomputation´
#[test]
fn clean_recomputation() {
    let precision = SymmetricMatrix::identity_scaled(5, 1.0);
    let covariance = SymmetricMatrix::identity_scaled(5, 1.0);

    let inputs = VisitInputs {
        labels_since_recompute: 1000,
        diagonal_ratio: 1.0,
        dimensions_at_floor: 0,
        sync_error_threshold: synchronisation_error_threshold(5),
        spectral_floor_mass: 0.0,
        // No clamp mass: this rebuild is about rounding, and a model with
        // nothing at the floor attributes none of its drift to the prior.
        clamp_mass: &[],
    };
    let (outcome, new_cov, baseline) = synchronisation_visit(&precision, &covariance, None, true, &inputs);

    assert!(outcome.is_rebuild(), "the verdict was skipped, so the visit rebuilt");
    assert!(!outcome.is_alarm());
    assert!(!outcome.is_cascade_terminus());
    assert!(new_cov.is_some());
    assert!(baseline.adopted().expect("the visit rebuilt"), "a healthy rebuild is adopted");
    assert_eq!(baseline.verdict().expect("the visit rebuilt"), RebuildVerdict::Clean);
    assert_eq!(baseline.labels_at_record, 1000);

    // After clean recompute, sync error should be near machine epsilon —
    // well below the specification's threshold at this width, which is the
    // coefficient times the dimension rather than a flat ceiling
    // (´def:monitoring:synchronisation-error´).
    let sync_err = outcome.sync_error_residual();
    assert_below(
        sync_err,
        DEFAULT_TOLERANCES.precision_sync_ceiling(5),
        "clean recompute sync error",
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// ModelPrecisionHealth Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// A refused factorisation takes the interval to its floor in one step rather
/// than halving it, and clears the recovery streak with it. Halving is the
/// proportional response to drift that is being removed; a refusal removes
/// none, and it only happens now on a model whose measurement said the drift
/// was worth acting on. The model is looked at as often as the cadence permits
/// until a measurement comes back good.
///
/// ´claim:bayes:a-refused-factorisation-takes-the-interval-to-its-floor´
/// ´test:crate:interval-floor-on-terminus´
#[test]
fn interval_floor_on_terminus() {
    let mut health = ModelPrecisionHealth::with_interval(1000);

    let outcome = RecomputeOutcome::Alarm {
        sync_error_residual: 0.01,
        sync_error_after: 0.03,
        refused_pivot: Some(0),
    };

    health.record_outcome(&outcome);
    let changed = health.update_interval(1000, &outcome);

    assert!(changed);
    assert_eq!(health.n_recompute_effective, N_RECOMPUTE_FLOOR);
    assert_eq!(health.consecutive_clean_recomputes, 0);
}

/// A rebuild that was needed halves the interval, resets recovery and is
/// counted; a measurement that found nothing to do leaves all three alone.
/// The comparison against the threshold no longer happens here — it happens at
/// the measurement, and the outcome that reaches the cadence names the answer
/// — so what this reads is that the cadence acts on the visit's verdict rather
/// than re-deriving it.
///
/// ´claim:bayes:a-needed-rebuild-shortens-the-interval-and-restarts-recovery´
/// ´test:crate:needed-rebuild-shortens-interval´
#[test]
fn needed_rebuild_shortens_interval() {
    let monitoring = crate::MonitoringConfig {
        sync_error_threshold_per_dimension: 0.2,
        ..Default::default()
    };
    let threshold = monitoring.synchronisation_error_threshold(5);

    let mut healthy = ModelPrecisionHealth::with_interval(1000);
    let measured = RecomputeOutcome::Measured {
        sync_error_residual: threshold,
        at_resolution: false,
    };
    healthy.record_outcome(&measured);
    assert!(!healthy.update_interval(1000, &measured));
    assert_eq!(healthy.n_recompute_effective, 1000);
    assert_eq!(healthy.sync_error_shortenings, 0);

    let mut health = ModelPrecisionHealth::with_interval(1000);
    health.consecutive_clean_recomputes = N_RECOVERY_CLEAN - 1;
    let outcome = RecomputeOutcome::Rebuilt {
        sync_error_residual: threshold + f64::EPSILON,
        drift_was_real: true,
    };

    health.record_outcome(&outcome);
    let changed = health.update_interval(1000, &outcome);

    assert!(changed);
    assert_eq!(health.n_recompute_effective, 500);
    assert_eq!(health.consecutive_clean_recomputes, 0);
    assert_eq!(health.sync_error_shortenings, 1);
}

/// A shortened interval is not permanent: the good measurement that completes
/// the required streak restores the configured interval outright. Without a
/// way back, a single bad patch would leave the model paying for frequent
/// visits for the rest of its life — and the recovery is now spent out of
/// measurements, which is what lets a model that needed one rebuild climb back
/// to its configured interval without paying for three more
/// (´dec:posterior:adaptive-cadence´).
///
/// ´claim:bayes:a-run-of-good-measurements-restores-the-configured-interval´
/// ´test:crate:interval-recovery´
#[test]
fn interval_recovery() {
    let mut health = ModelPrecisionHealth::with_interval(500); // Already shortened
    health.consecutive_clean_recomputes = N_RECOVERY_CLEAN - 1;

    let outcome = RecomputeOutcome::Measured {
        sync_error_residual: 1e-14,
        at_resolution: false,
    };

    health.record_outcome(&outcome);
    let changed = health.update_interval(1000, &outcome);

    assert!(changed);
    assert_eq!(health.n_recompute_effective, 1000);
    assert_eq!(health.consecutive_clean_recomputes, N_RECOVERY_CLEAN);
}

/// Repeated shortening stops at the floor rather than continuing down: a run
/// of needed rebuilds takes the interval there, and the next reports no change
/// at all. Below the floor a cubic-cost factorisation would be amortised over
/// too few labels to be worth its price, so a persistently drifting model
/// recomputes often but not without limit.
///
/// ´claim:bayes:the-interval-never-shortens-below-its-floor´
/// ´test:crate:interval-floor´
#[test]
fn interval_floor() {
    let mut health = ModelPrecisionHealth::with_interval(200);

    let outcome = RecomputeOutcome::Rebuilt {
        sync_error_residual: 0.001,
        drift_was_real: true,
    };

    // First halving: 200 → 100
    health.record_outcome(&outcome);
    health.update_interval(1000, &outcome);
    assert_eq!(health.n_recompute_effective, N_RECOMPUTE_FLOOR);

    // Second "halving" should stay at floor
    health.record_outcome(&outcome);
    let changed = health.update_interval(1000, &outcome);
    assert!(!changed);
    assert_eq!(health.n_recompute_effective, N_RECOMPUTE_FLOOR);
}

// ═══════════════════════════════════════════════════════════════════════════════
// synchronisation_error Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// A precision and covariance that are exactly inverse register no
/// disagreement at all. The measure has to read zero on a healthy pair, or
/// every recomputation decision it feeds would be made against a standing
/// offset rather than against real drift.
///
/// ´claim:bayes:an-exactly-inverse-pair-registers-no-synchronisation-error´
/// ´test:crate:sync-error-identity´
#[test]
fn sync_error_identity() {
    let precision = SymmetricMatrix::identity_scaled(5, 1.0);
    let covariance = SymmetricMatrix::identity_scaled(5, 1.0);

    let error = synchronisation_error(&precision, &covariance);

    assert_below(error, DEFAULT_TOLERANCES.precision_sync_ceiling(5), "identity BΣ sync error");
}

/// The synchronisation ceiling the harness holds a model to is the
/// specification's threshold at that model's own width — the coefficient times
/// the dimension — and not a flat figure. The distinction has teeth at both
/// ends of the range the suite exercises: a five-wide model is held twenty
/// times tighter than the flat ceiling that preceded it and a fifty-wide model
/// twice as tight, while the two coincide only at a width of one hundred, which
/// is what made the flat figure look adequate
/// (´def:monitoring:synchronisation-error´).
///
/// ´claim:bayes:the-synchronisation-ceiling-scales-with-the-model-width-the-specification-scales-it-by´
/// ´test:crate:precision-sync-ceiling-scales-with-the-dimension´
#[test]
fn precision_sync_ceiling_scales_with_the_dimension() {
    let tol = DEFAULT_TOLERANCES;

    // The specification's own figure at each width the consuming tests use.
    assert_near(tol.precision_sync_ceiling(5), 5e-6, 1e-18, "threshold at width 5");
    assert_near(tol.precision_sync_ceiling(50), 5e-5, 1e-18, "threshold at width 50");

    // The flat ceiling this replaced, and the single width at which it agreed.
    assert_near(tol.precision_sync_ceiling(100), 1e-4, 1e-18, "threshold at width 100");

    // Strictly tighter than the flat figure everywhere the suite consumes it.
    assert!(
        tol.precision_sync_ceiling(5) < 1e-4 && tol.precision_sync_ceiling(50) < 1e-4,
        "both consuming widths must be held tighter than the flat ceiling",
    );

    // And it scales, rather than merely differing.
    assert_near(
        tol.precision_sync_ceiling(50),
        10.0 * tol.precision_sync_ceiling(5),
        1e-18,
        "the ceiling is linear in the width",
    );
}

/// The measure is quantitative, not merely a flag: a covariance stretched by a
/// tenth on one coordinate registers a disagreement of that same tenth. Because
/// the number tracks the size of the departure, thresholds on it mean something
/// and a slowly worsening model can be watched rather than only caught. On a
/// departure confined to one diagonal entry the norm and a maximum over the
/// diagonal agree exactly, which is why this test held under both and why the
/// interchange needed the test below to become visible.
///
/// ´claim:bayes:the-synchronisation-error-reports-how-far-the-pairs-product-has-departed-from-the-identity´
/// ´test:crate:sync-error-drift´
#[test]
fn sync_error_drift() {
    let precision = SymmetricMatrix::identity_scaled(5, 1.0);
    // Create covariance with drift: diag(1.1, 1.0, 1.0, 1.0, 1.0)
    let cov_mat = Mat::from_fn(5, 5, |i, j| if i == j { if i == 0 { 1.1 } else { 1.0 } } else { 0.0 });
    let covariance = SymmetricMatrix::from_computation(cov_mat);

    let error = synchronisation_error(&precision, &covariance);

    // BΣ should have (0,0) = 1.1, so error ~ 0.1.
    assert_near(error, 0.1, DEFAULT_TOLERANCES.default, "drifted BΣ sync error");
}

/// A pair whose product has an exact unit diagonal and a large off-diagonal
/// departure registers that departure. The measure is a norm over the whole
/// product, and the norm and a maximum over the diagonal are different
/// quantities that a restatement can silently interchange: here the diagonal
/// reads perfect health while the two representations disagree by a half on
/// four entries. Since the reading is what a recomputation interval would be
/// shortened on, a measure blind to off-diagonal growth would leave the model
/// on its long interval exactly while it drifted.
///
/// ´claim:bayes:off-diagonal-departure-is-measured-even-when-the-diagonal-is-exact´
/// ´test:crate:sync-error-sees-off-diagonal-departure´
#[test]
fn sync_error_sees_off_diagonal_departure() {
    // B = I, so BΣ = Σ: a unit diagonal with ±0.5 off the diagonal.
    let precision = SymmetricMatrix::identity_scaled(3, 1.0);
    let cov_mat = Mat::from_fn(3, 3, |i, j| if i == j { 1.0 } else { 0.5 });
    let covariance = SymmetricMatrix::from_computation(cov_mat);

    let error = synchronisation_error(&precision, &covariance);

    // Six off-diagonal entries at 0.5: √(6 × 0.25) = √1.5.
    assert_near(
        error,
        1.5_f64.sqrt(),
        DEFAULT_TOLERANCES.default,
        "off-diagonal BΣ sync error",
    );
}

/// Each good measurement lengthens the run of clean visits by exactly one, and
/// a rebuild ends the run whatever its own result was. The streak counts
/// consecutive visits that found nothing to do rather than merely recording
/// that the last one did, and recovery of the interval is spent out of it — so
/// a model that needed rebuilding is not one whose condition has passed, even
/// where the rebuild succeeded.
///
/// ´claim:bayes:each-good-measurement-lengthens-the-clean-streak-and-a-rebuild-ends-it´
/// ´test:crate:clean-increments-consecutive´
#[test]
fn clean_increments_consecutive() {
    let mut health = ModelPrecisionHealth::default();
    assert_eq!(health.consecutive_clean_recomputes, 0);

    let outcome = RecomputeOutcome::Measured {
        sync_error_residual: 1e-14,
        at_resolution: false,
    };

    health.record_outcome(&outcome);
    assert_eq!(health.consecutive_clean_recomputes, 1);

    health.record_outcome(&outcome);
    assert_eq!(health.consecutive_clean_recomputes, 2);

    // An adopted rebuild is a success and still ends the run.
    health.record_outcome(&RecomputeOutcome::Rebuilt {
        sync_error_residual: 1e-3,
        drift_was_real: true,
    });
    assert_eq!(health.consecutive_clean_recomputes, 0);
}

/// Each of the three outcomes answers for itself whether a factorisation was
/// paid for, whether the alarm was raised, whether the factorisation ran out
/// of options, and what residual the measurement found. Callers branch on
/// these answers rather than on the shape of the outcome, so the answers have
/// to be right for all three and not only the common one — and the common one
/// is now the measurement that rebuilt nothing.
///
/// ´claim:bayes:every-visit-outcome-reports-whether-it-rebuilt-and-what-it-measured´
/// ´test:crate:outcome-accessors´
#[test]
fn outcome_accessors() {
    let measured = RecomputeOutcome::Measured {
        sync_error_residual: 0.001,
        at_resolution: true,
    };
    assert!(measured.is_measurement());
    assert!(!measured.is_rebuild());
    assert!(!measured.is_alarm());
    assert!(!measured.is_cascade_terminus());
    assert_eq!(measured.refused_pivot(), None);
    assert_near(
        measured.sync_error_residual(),
        0.001,
        DEFAULT_TOLERANCES.bit_identical,
        "measured.sync_error_residual",
    );

    let rebuilt = RecomputeOutcome::Rebuilt {
        sync_error_residual: 0.002,
        drift_was_real: true,
    };
    assert!(!rebuilt.is_measurement());
    assert!(rebuilt.is_rebuild());
    assert!(!rebuilt.is_alarm());
    assert!(!rebuilt.is_cascade_terminus());

    // The alarm on a factorisation that succeeded and was declined: a rebuild,
    // an alarm, and not a terminus.
    let declined = RecomputeOutcome::Alarm {
        sync_error_residual: 0.003,
        sync_error_after: 0.009,
        refused_pivot: None,
    };
    assert!(declined.is_rebuild());
    assert!(declined.is_alarm());
    assert!(!declined.is_cascade_terminus());
    assert_eq!(declined.refused_pivot(), None);

    // And the alarm on a refusal: the terminus is the refusal, not the alarm.
    let terminus = RecomputeOutcome::Alarm {
        sync_error_residual: 0.003,
        sync_error_after: 0.003,
        refused_pivot: Some(2),
    };
    assert!(terminus.is_alarm());
    assert!(terminus.is_cascade_terminus());
    assert_eq!(terminus.refused_pivot(), Some(2));
    assert_near(
        terminus.sync_error_residual(),
        0.003,
        DEFAULT_TOLERANCES.bit_identical,
        "terminus.sync_error_residual",
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Spectral floor and degradation reading
// ═══════════════════════════════════════════════════════════════════════════════

/// The floor a rebuild derives is exactly the least eigenvalue the Cholesky
/// success condition demands at that width and that largest eigenvalue: a
/// matrix held at the floor sits precisely at the condition's boundary, and a
/// floor a shade lower puts it outside. Deriving the least sufficient value is
/// what keeps the floor an arithmetic requirement rather than a modelling
/// assertion, so a floor merely sufficient would be claiming confidence no
/// factorisation needs.
///
/// ´claim:bayes:the-spectral-floor-is-the-least-eigenvalue-the-success-condition-admits´
/// ´test:crate:the-spectral-floor-is-the-least-the-success-condition-admits´
#[test]
#[allow(clippy::cast_precision_loss)] // Test widths are far below f64's exact integer range.
fn the_spectral_floor_is_the_least_the_success_condition_admits() {
    let unit_roundoff = f64::EPSILON / 2.0;

    for &(p, lambda_max) in &[(5_usize, 1.0_f64), (100, 3.0), (1126, 300.0)] {
        let floor = spectral_floor(lambda_max, p);

        // The derivation, recomputed from the condition rather than from the
        // implementation: λ_min ≥ c · p^(3/2) · u · λ_max.
        let width = p as f64;
        let expected = CHOLESKY_SUCCESS_CONSTANT * width.powf(1.5) * unit_roundoff * lambda_max;
        assert_near(
            floor,
            expected,
            expected * 1e-12,
            "the floor is the success condition rearranged",
        );

        // A matrix held at the floor sits at the boundary: the condition's
        // left-hand side is one.
        let condition_at_floor = CHOLESKY_SUCCESS_CONSTANT * width.powf(1.5) * unit_roundoff * (lambda_max / floor);
        assert_near(condition_at_floor, 1.0, 1e-9, "the floor is the condition's boundary");

        // A floor below it puts the matrix outside the condition.
        let condition_below = CHOLESKY_SUCCESS_CONSTANT * width.powf(1.5) * unit_roundoff * (lambda_max / (floor * 0.5));
        assert!(condition_below > 1.0, "a lower floor leaves the success condition");
    }

    // No spectrum, no floor.
    assert_near(spectral_floor(1.0, 0), 0.0, DEFAULT_TOLERANCES.bit_identical, "empty width");
    assert_near(
        spectral_floor(f64::NAN, 10),
        0.0,
        DEFAULT_TOLERANCES.bit_identical,
        "no finite largest eigenvalue",
    );
    assert_near(
        spectral_floor(-1.0, 10),
        0.0,
        DEFAULT_TOLERANCES.bit_identical,
        "no positive largest eigenvalue",
    );
}

/// Halving the largest eigenvalue halves the floor derived from it, and the
/// derivation reads nothing else about the spectrum. The floor tracks the
/// evidence in both directions because a floor that could only rise would
/// assert a confidence that returning evidence has already refuted.
///
/// ´claim:bayes:the-spectral-floor-falls-with-the-largest-eigenvalue-it-is-derived-from´
/// ´test:crate:the-spectral-floor-follows-the-evidence-down´
#[test]
fn the_spectral_floor_follows_the_evidence_down() {
    let wide = spectral_floor(300.0, 512);
    let narrower = spectral_floor(150.0, 512);

    assert!(narrower < wide, "a smaller largest eigenvalue means a smaller floor");
    assert_near(
        narrower * 2.0,
        wide,
        wide * 1e-12,
        "the floor is linear in the largest eigenvalue",
    );
}

/// Scaling the floor and the least eigenvalue together leaves the degradation
/// reading unchanged, arriving evidence drives it toward zero, a least
/// eigenvalue equal to the floor reads one, and a direction outside the
/// definite cone reads one as well. The reading is meant to be comparable
/// across models of different scales, so cancelling the units is the property
/// that makes a maximum across models mean anything.
///
/// ´claim:bayes:the-degradation-reading-is-dimensionless-monotone-and-tends-to-zero-with-evidence´
/// ´test:crate:the-floor-share-is-dimensionless-monotone-and-tends-to-zero´
#[test]
fn the_floor_share_is_dimensionless_monotone_and_tends_to_zero() {
    // Dimensionless: the reading survives a change of scale in both arguments.
    let at_unit_scale = floor_share(1e-8, 1e-4);
    let at_thousand_scale = floor_share(1e-5, 1e-1);
    assert_near(
        at_unit_scale,
        at_thousand_scale,
        1e-15,
        "the reading cancels the units it is built from",
    );

    // Monotone, and tending to zero as evidence arrives in the weakest
    // direction: a rising least eigenvalue lowers the share the floor carries.
    let mut previous = f64::INFINITY;
    for exponent in 0..6 {
        let lambda_min = 1e-8 * 10.0_f64.powi(exponent);
        let share = floor_share(1e-8, lambda_min);
        assert!(share < previous, "arriving evidence lowers the share the floor carries");
        previous = share;
    }
    assert!(previous < 1e-4, "the share tends to zero as evidence accumulates");

    // Monotone in the other argument too: a rising floor raises the share.
    assert!(
        floor_share(2e-8, 1e-4) > floor_share(1e-8, 1e-4),
        "a higher floor carries a larger share"
    );

    // A least eigenvalue at the floor is entirely the floor's doing.
    assert_near(floor_share(1e-8, 1e-8), 1.0, 1e-15, "the floor carrying the whole of it");

    // Outside the definite cone there is no evidence holding the direction up
    // at all, and the reading says so rather than reporting a negative share.
    assert_near(
        floor_share(1e-8, -1.0),
        1.0,
        DEFAULT_TOLERANCES.bit_identical,
        "an indefinite direction",
    );
    assert_near(
        floor_share(1e-8, 0.0),
        1.0,
        DEFAULT_TOLERANCES.bit_identical,
        "a singular direction",
    );

    // No floor in force, nothing to attribute to it.
    assert_near(
        floor_share(0.0, 1e-4),
        0.0,
        DEFAULT_TOLERANCES.bit_identical,
        "no floor in force",
    );
}

/// The deficit a reading hands the rebuild is exactly what the least
/// eigenvalue is short of the floor, and nothing where the evidence already
/// clears it. Adding that much of the identity shifts every eigenvalue by it,
/// so the least lands on the floor and no eigenvalue is raised further than
/// the least one needed — which is what makes the floor the smallest
/// anti-conservative claim the arithmetic admits rather than a convenient one.
///
/// ´claim:bayes:the-deficit-is-what-the-least-eigenvalue-is-short-of-the-floor´
/// ´test:crate:the-deficit-is-what-the-least-eigenvalue-is-short-of-the-floor´
#[test]
fn the_deficit_is_what_the_least_eigenvalue_is_short_of_the_floor() {
    let diagonal = |top: f64, bottom: f64| {
        let mat = Mat::from_fn(8, 8, |i, j| {
            if i == j {
                if i == 0 {
                    top
                } else if i == 1 {
                    bottom
                } else {
                    1.0
                }
            } else {
                0.0
            }
        });
        SymmetricMatrix::from_computation(mat)
    };

    // Short of the floor: the deficit closes exactly the gap.
    let short = diagonal(300.0, 1e-18);
    let reading = SpectralReading::measure(&short, 0.0).expect("a spectrum");
    assert!(reading.deficit > 0.0, "the least eigenvalue is short of the floor");
    assert_near(
        reading.lambda_min + reading.deficit,
        reading.lambda_floor,
        reading.lambda_floor * 1e-12,
        "the deficit lands the least eigenvalue on the floor",
    );
    assert_near(
        reading.lambda_min_floored(),
        reading.lambda_floor,
        reading.lambda_floor * 1e-12,
        "and the reading says so",
    );

    // Clear of it: nothing is added, and a measured least eigenvalue above the
    // floor is not raised to meet anything.
    let clear = diagonal(300.0, 1.0);
    let ample = SpectralReading::measure(&clear, 0.0).expect("a spectrum");
    assert!(ample.lambda_min > ample.lambda_floor, "the evidence clears the floor");
    assert_near(
        ample.deficit,
        0.0,
        DEFAULT_TOLERANCES.bit_identical,
        "a spectrum above the floor asks for nothing",
    );
    assert!(
        ample.floor_share < 1e-9,
        "and the reading reports the floor holding almost none of it, got {}",
        ample.floor_share
    );
}

/// An addition to the precision matrix that the covariance does not follow is
/// read by the synchronisation monitor as drift, and the reading is exactly
/// the size of the addition scaled by the covariance. This is the arithmetic
/// that decides where the spectral floor is applied: a per-label increment to
/// the precision matrix alone would enter the monitor's reading at
/// `δ·‖Σ‖`, which at the floor's own magnitude is of order one per label, so
/// the floor would manufacture the very drift the monitor exists to detect and
/// the cadence would shorten the interval in response to it. Applying the
/// floor where the covariance is rebuilt costs nothing extra and leaves the
/// pair consistent (´dec:posterior:spectral-floor´).
///
/// ´claim:bayes:an-addition-the-covariance-does-not-follow-is-measured-as-drift´
/// ´test:crate:an-addition-the-covariance-does-not-follow-is-measured-as-drift´
#[test]
fn an_addition_the_covariance_does_not_follow_is_measured_as_drift() {
    // A geometry with a weak direction, so that the covariance is large along
    // it and the addition's effect on the reading is the one the floor's own
    // magnitude would produce.
    let mat = Mat::from_fn(6, 6, |i, j| if i == j { if i == 0 { 100.0 } else { 1e-6 } } else { 0.0 });
    let precision = SymmetricMatrix::from_computation(mat);
    let covariance = SymmetricMatrix::from_computation(Mat::from_fn(6, 6, |i, j| {
        if i == j { if i == 0 { 1.0 / 100.0 } else { 1e6 } } else { 0.0 }
    }));

    // The pair starts consistent.
    let paired = synchronisation_error(&precision, &covariance);
    assert!(paired < 1e-9, "the pair starts as inverses, got {paired:e}");

    // Add a multiple of the identity to the precision matrix alone.
    let delta = 1e-8;
    let shifted = precision.add_scaled_identity(delta);
    let unpaired = synchronisation_error(&shifted, &covariance);

    // The reading is the addition times the covariance's own norm, because
    // (B + δI)Σ − I = (BΣ − I) + δΣ and the first term is negligible here.
    let covariance_norm = {
        let mut sum_sq = 0.0_f64;
        for j in 0..6 {
            for i in 0..6 {
                let entry = covariance.as_inner()[(i, j)];
                sum_sq = entry.mul_add(entry, sum_sq);
            }
        }
        sum_sq.sqrt()
    };
    let predicted = delta * covariance_norm;
    assert_near(
        unpaired,
        predicted,
        predicted * 1e-6,
        "the unfollowed addition is measured as drift of exactly its own size",
    );
    assert!(
        unpaired > paired * 1e6,
        "and it dwarfs the drift the consistent pair carried: {paired:e} against {unpaired:e}"
    );
}

/// An indefinite matrix reaches the rebuild and the rebuild offers nothing: the factorisation refuses it, the baseline records the refusal, no covariance comes back, and the model keeps what it had. The retired second phase answered here with a success carrying a replacement count, and the rebuild had to refuse that answer on its own measurement; there is no answer to refuse now. The floor declines to act, which is the other half of the reading — a matrix outside the definite cone is a verdict and not a condition to floor.
///
/// ´claim:bayes:a-rebuild-of-an-indefinite-matrix-is-refused-and-offers-nothing´
/// ´test:crate:a-rebuild-that-is-not-better-is-refused´
#[test]
fn a_rebuild_that_is_not_better_is_refused() {
    // The witness the suite already carries for the diagonal and the spectrum
    // disagreeing: ones on the diagonal, twos off it, spectrum three and minus
    // one. Its diagonal test finds nothing wrong; the factorisation refuses it.
    let mat = Mat::from_fn(2, 2, |i, j| if i == j { 1.0 } else { 2.0 });
    let precision = SymmetricMatrix::from_computation(mat);
    let covariance = SymmetricMatrix::identity_scaled(2, 1.0);

    let inputs = VisitInputs {
        labels_since_recompute: 1000,
        diagonal_ratio: 1.0,
        dimensions_at_floor: 0,
        sync_error_threshold: synchronisation_error_threshold(2),
        spectral_floor_mass: 0.0,
        // No clamp mass: this rebuild is about rounding, and a model with
        // nothing at the floor attributes none of its drift to the prior.
        clamp_mass: &[],
    };
    let (_outcome, new_cov, baseline) = synchronisation_visit(&precision, &covariance, None, true, &inputs);

    assert_eq!(
        baseline.verdict().expect("the visit rebuilt"),
        RebuildVerdict::Refused,
        "the factorisation refuses the indefinite witness"
    );
    assert!(
        !baseline.adopted().expect("the visit rebuilt"),
        "a refused rebuild is not adopted"
    );
    assert!(new_cov.is_none(), "a refused rebuild hands back no covariance");
    assert!(
        baseline.sync_error_after().expect("the visit rebuilt") > inputs.sync_error_threshold,
        "the answer is not the inverse of the matrix in hand: {:e} against a threshold of {:e}",
        baseline.sync_error_after().expect("the visit rebuilt"),
        inputs.sync_error_threshold
    );

    // The spectrum the rebuild read is the witness's own, the floor declined to
    // act on it — a matrix outside the definite cone is a verdict and not a
    // condition to floor — and the degradation reading says the weakest
    // direction is held up by nothing.
    let spectrum = baseline.spectrum().expect("the witness has a spectrum");
    assert_near(
        spectrum.deficit,
        0.0,
        DEFAULT_TOLERANCES.bit_identical,
        "the floor does not carry an indefinite matrix into the cone",
    );
    assert_near(spectrum.lambda_max, 3.0, 1e-12, "the witness's largest eigenvalue");
    assert_near(spectrum.lambda_min, -1.0, 1e-12, "the witness's least eigenvalue");
    assert_near(
        spectrum.floor_share,
        1.0,
        DEFAULT_TOLERANCES.bit_identical,
        "no evidence holds it up",
    );
}

/// A rebuild the model declined takes the interval to its floor in one step
/// and clears the recovery streak with it, where a good measurement leaves
/// both alone. The decline is the alarm now: a rebuild runs only where the
/// measurement found drift worth acting on, so a rebuild whose answer was not
/// an improvement is a model whose drift is real and whose fresh inverse is
/// worse than the pair it holds. Halving would be a proportional response to
/// drift being removed, and none is being removed
/// (´rep:assayer:declined-rebuild-cadence´).
///
/// ´claim:bayes:a-rebuild-that-was-not-adopted-takes-the-interval-to-its-floor´
/// ´test:crate:a-rebuild-not-adopted-shortens-the-interval´
#[test]
fn a_rebuild_not_adopted_shortens_the_interval() {
    let measured = RecomputeOutcome::Measured {
        sync_error_residual: 1e-14,
        at_resolution: false,
    };

    // A measurement that found nothing to do: the interval holds.
    let mut healthy = ModelPrecisionHealth::with_interval(1000);
    healthy.record_outcome(&measured);
    assert!(!healthy.update_interval(1000, &measured));
    assert_eq!(healthy.n_recompute_effective, 1000);

    // The same reading, but a rebuild ran on it and its answer was declined:
    // the interval goes to the floor and the streak goes with it.
    let declined = RecomputeOutcome::Alarm {
        sync_error_residual: 1e-14,
        sync_error_after: 1e-13,
        refused_pivot: None,
    };
    let mut alarmed = ModelPrecisionHealth::with_interval(1000);
    alarmed.consecutive_clean_recomputes = N_RECOVERY_CLEAN;
    alarmed.record_outcome(&declined);
    assert!(alarmed.update_interval(1000, &declined));
    assert_eq!(alarmed.n_recompute_effective, N_RECOMPUTE_FLOOR);
    assert_eq!(alarmed.consecutive_clean_recomputes, 0);

    // And the policy function says the same on its own.
    assert_eq!(
        compute_new_interval(1000, 1000, N_RECOVERY_CLEAN, &declined),
        N_RECOMPUTE_FLOOR
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// The fast check, relative to the baseline (´dec:posterior:recomputation-trigger´)
// ═══════════════════════════════════════════════════════════════════════════════

/// A diagonal ratio of a million million, on a matrix whose counter is nowhere
/// near its interval, calls for nothing. The retired arm tested that ratio
/// against a fixed threshold and the replenishment clamp pins the ratio's
/// denominator at the floor, so once any dimension rested there the arm was
/// satisfied on every label for ever — and the rebuild it called for cannot
/// change the precision matrix the ratio is read from, so the arm asked for a
/// cubic factorisation per label to no effect. Nothing absolute about the
/// conditioning fires now, whether or not a baseline exists.
///
/// ´claim:bayes:no-absolute-conditioning-reading-calls-for-a-rebuild´
/// ´test:crate:conditioning-alone-never-calls-for-a-rebuild´
#[test]
fn conditioning_alone_never_calls_for_a_rebuild() {
    let mat = Mat::from_fn(5, 5, |i, j| if i == j { if i == 0 { 1e12 } else { 1.0 } } else { 0.0 });
    let precision = SymmetricMatrix::from_computation(mat);

    // No baseline: only the counter and the degenerate arm are available, and
    // neither applies.
    let fresh = should_recompute(&precision, 500, 1000, None, DEFAULT_KAPPA_GROWTH_FACTOR, DEFAULT_LAMBDA_FLOOR);
    assert!(
        matches!(fresh, RecomputeTrigger::NotNeeded { .. }),
        "a ratio of 1e12 with no baseline calls for nothing, got {fresh:?}"
    );
    assert!(!fresh.is_needed());
    assert!(fresh.diagonal_ratio() >= 1e12, "the ratio is still read and still reported");

    // A baseline recording the same ratio: the growth arm has nothing to fire
    // on either, however large the absolute reading is.
    let baseline = baseline_at(1e12, 0);
    let settled = should_recompute(
        &precision,
        500,
        1000,
        Some(&baseline),
        DEFAULT_KAPPA_GROWTH_FACTOR,
        DEFAULT_LAMBDA_FLOOR,
    );
    assert!(
        matches!(settled, RecomputeTrigger::NotNeeded { .. }),
        "a ratio equal to its baseline calls for nothing, got {settled:?}"
    );
}

/// Immediately after a rebuild the model adopted, the check on the very same
/// matrix says no rebuild is needed. This is the property the retired arm did
/// not have and the whole reason the check is relative: every quantity it
/// tests is one the rebuild has just reset, so a rebuild cannot leave the
/// engine asking for another. A check that could would spend a cubic
/// factorisation per label for ever, which is what the shipped engine did.
///
/// ´claim:bayes:a-rebuild-leaves-the-fast-check-asking-for-nothing´
/// ´test:crate:an-adopted-rebuild-leaves-the-check-quiet´
#[test]
fn an_adopted_rebuild_leaves_the_check_quiet() {
    // A matrix with a dimension resting at the replenishment floor, which is
    // the state that pinned the retired arm above its threshold for ever.
    let mat = Mat::from_fn(6, 6, |i, j| {
        if i == j {
            if i == 0 { DEFAULT_LAMBDA_FLOOR } else { 300.0 }
        } else {
            0.0
        }
    });
    let precision = SymmetricMatrix::from_computation(mat);
    let covariance = SymmetricMatrix::identity_scaled(6, 1.0);

    // Drive the real rebuild, so the baseline is the one the engine writes
    // rather than one the test composes.
    let first = should_recompute(
        &precision,
        1000,
        1000,
        None,
        DEFAULT_KAPPA_GROWTH_FACTOR,
        DEFAULT_LAMBDA_FLOOR,
    );
    assert!(
        matches!(first, RecomputeTrigger::Counter { .. }),
        "the counter calls it, got {first:?}"
    );

    let inputs = VisitInputs::from_trigger(&first, 1000, synchronisation_error_threshold(6), 0.0, &[]);
    let (_outcome, new_cov, baseline) = synchronisation_visit(&precision, &covariance, None, true, &inputs);
    assert!(
        baseline.adopted().expect("the visit rebuilt"),
        "the rebuild of a definite matrix is adopted"
    );
    assert!(new_cov.is_some());

    // The counter is reset by the rebuild, and the matrix has not moved.
    let after = should_recompute(
        &precision,
        0,
        baseline.labels_at_record.max(1),
        Some(&baseline),
        DEFAULT_KAPPA_GROWTH_FACTOR,
        DEFAULT_LAMBDA_FLOOR,
    );
    assert!(
        matches!(after, RecomputeTrigger::NotNeeded { .. }),
        "the check is quiet on the matrix the rebuild just answered for, got {after:?}"
    );

    // And the ratio it read is the one the baseline carries, which is what
    // makes the growth arm's comparison a comparison with itself.
    assert_near(
        after.diagonal_ratio(),
        baseline.diagonal_ratio,
        DEFAULT_TOLERANCES.bit_identical,
        "the ratio the check reads is the ratio the baseline recorded",
    );
}

/// The three relative arms each fire on their own quantity and on nothing
/// else: the ratio doubling past its baseline, the floored-dimension count
/// moving off the baseline's, and the label counter reaching the interval. A
/// growth arm that fired on a ratio below its factor, or a floored-count arm
/// that fired on an unchanged count, would put the engine back on the cadence
/// the fixed threshold produced.
///
/// ´claim:bayes:each-relative-arm-of-the-fast-check-fires-on-its-own-quantity´
/// ´test:crate:the-relative-arms-fire-on-their-own-quantities´
#[test]
fn the_relative_arms_fire_on_their_own_quantities() {
    let ratio_at = |top: f64| {
        let mat = Mat::from_fn(4, 4, |i, j| if i == j { if i == 0 { top } else { 1.0 } } else { 0.0 });
        SymmetricMatrix::from_computation(mat)
    };

    // Growth: the baseline recorded a ratio of one hundred; two hundred is at
    // the factor and does not fire, two hundred and one is past it and does.
    let baseline = baseline_at(100.0, 0);
    let at_factor = should_recompute(&ratio_at(200.0), 0, 1000, Some(&baseline), 2.0, DEFAULT_LAMBDA_FLOOR);
    assert!(
        matches!(at_factor, RecomputeTrigger::NotNeeded { .. }),
        "growth exactly at the factor is not growth past it, got {at_factor:?}"
    );

    let past_factor = should_recompute(&ratio_at(201.0), 0, 1000, Some(&baseline), 2.0, DEFAULT_LAMBDA_FLOOR);
    match past_factor {
        RecomputeTrigger::ConditioningGrowth {
            diagonal_ratio,
            baseline_ratio,
            ..
        } => {
            assert_near(baseline_ratio, 100.0, DEFAULT_TOLERANCES.bit_identical, "the baseline ratio");
            assert!(diagonal_ratio > 200.0, "the ratio that outgrew it");
        }
        other => panic!("expected ConditioningGrowth, got {other:?}"),
    }

    // The floored count: unchanged is quiet, moved calls for a rebuild. The
    // matrix carries one dimension at the floor and the baseline claims three.
    let floored = Mat::from_fn(4, 4, |i, j| {
        if i == j {
            if i == 0 { DEFAULT_LAMBDA_FLOOR } else { 1.0 }
        } else {
            0.0
        }
    });
    let floored = SymmetricMatrix::from_computation(floored);
    let agreeing = baseline_at(1.0 / DEFAULT_LAMBDA_FLOOR, 1);
    let quiet = should_recompute(&floored, 0, 1000, Some(&agreeing), 2.0, DEFAULT_LAMBDA_FLOOR);
    assert!(
        matches!(quiet, RecomputeTrigger::NotNeeded { .. }),
        "an unchanged floored count is quiet, got {quiet:?}"
    );

    let disagreeing = baseline_at(1.0 / DEFAULT_LAMBDA_FLOOR, 3);
    let moved = should_recompute(&floored, 0, 1000, Some(&disagreeing), 2.0, DEFAULT_LAMBDA_FLOOR);
    match moved {
        RecomputeTrigger::FlooredCountChanged {
            dimensions_at_floor,
            baseline_dimensions_at_floor,
            ..
        } => {
            assert_eq!(dimensions_at_floor, 1);
            assert_eq!(baseline_dimensions_at_floor, 3);
        }
        other => panic!("expected FlooredCountChanged, got {other:?}"),
    }

    // The counter still reaches past both, which is the arm that bounds drift
    // the conditioning does not reveal.
    let counted = should_recompute(&floored, 1000, 1000, Some(&agreeing), 2.0, DEFAULT_LAMBDA_FLOOR);
    assert!(matches!(counted, RecomputeTrigger::Counter { .. }), "got {counted:?}");

    // A non-positive diagonal entry is the one absolute reading left.
    let broken = Mat::from_fn(3, 3, |i, j| if i == j { if i == 1 { -1.0 } else { 1.0 } } else { 0.0 });
    let broken = SymmetricMatrix::from_computation(broken);
    let degenerate = should_recompute(&broken, 0, 1000, Some(&agreeing), 2.0, DEFAULT_LAMBDA_FLOOR);
    assert!(
        matches!(degenerate, RecomputeTrigger::Degenerate { .. }),
        "got {degenerate:?}"
    );
}

/// A baseline carrying the given diagonal ratio and floored count, with
/// everything else at a healthy adopted rebuild's shape. The fast check reads
/// only those two fields, so the rest is scaffolding and is written once here
/// rather than at each call.
fn baseline_at(diagonal_ratio: f64, dimensions_at_floor: usize) -> crate::model::recompute::RecomputeBaseline {
    crate::model::recompute::RecomputeBaseline {
        labels_at_record: 1000,
        sync_error_before: 0.0,
        sync_error_prior_induced: 0.0,
        sync_error_residual: 0.0,
        sync_error_resolution: 0.0,
        at_resolution: false,
        diagonal_ratio,
        dimensions_at_floor,
        wall_nanos: 0,
        rebuild: Some(RebuildRecord {
            spectrum: None,
            sync_error_after: 0.0,
            verdict: RebuildVerdict::Clean,
            adopted: true,
            wall_nanos: 0,
        }),
    }
}

/// Two rebuilds in succession, the second on a matrix whose evidence has
/// returned: the floor the second derives is below the floor the first
/// derived, it asks for nothing where the first had to act, and the
/// degradation reading falls with both. Nothing carries the earlier floor
/// forward, because a floor that could only rise would keep asserting a
/// confidence the returning evidence has already refuted — and the reading
/// published beside it would go on reporting a degradation the model no longer
/// has.
///
/// ´claim:bayes:a-later-rebuild-lowers-the-floor-and-the-reading-rather-than-ratcheting-them´
/// ´test:crate:the-floor-and-the-reading-never-ratchet´
#[test]
fn the_floor_and_the_reading_never_ratchet() {
    let diagonal = |top: f64, bottom: f64| {
        let mat = Mat::from_fn(8, 8, |i, j| {
            if i == j {
                if i == 0 {
                    top
                } else if i == 1 {
                    bottom
                } else {
                    1.0
                }
            } else {
                0.0
            }
        });
        SymmetricMatrix::from_computation(mat)
    };
    let sigma = SymmetricMatrix::identity_scaled(8, 1.0);
    let inputs_at = |mass: f64| VisitInputs {
        labels_since_recompute: 1000,
        diagonal_ratio: 1.0,
        dimensions_at_floor: 0,
        sync_error_threshold: synchronisation_error_threshold(8),
        spectral_floor_mass: mass,
        // No clamp mass: this rebuild is about rounding, and a model with
        // nothing at the floor attributes none of its drift to the prior.
        clamp_mass: &[],
    };

    // A stretched matrix whose weakest direction has fallen through the floor:
    // the rebuild has to act, and the reading says the floor is holding almost
    // the whole of that direction up.
    let stretched = diagonal(300.0, 1e-20);
    let (_o1, _c1, first) = synchronisation_visit(&stretched, &sigma, None, true, &inputs_at(0.0));
    let first_spectrum = first.spectrum().expect("the first rebuild read a spectrum");
    assert!(first_spectrum.deficit > 0.0, "the first rebuild had a floor to apply");
    let first_share = first.floor_share().expect("the first reading");
    assert!(
        first_share > 0.99,
        "a direction the floor is holding up reads near one, got {first_share}"
    );

    // Evidence has returned: the weakest direction is far stronger and the
    // largest is an order smaller. The floor derived is lower, it asks for
    // nothing, and the reading falls to what the earlier deficit still
    // contributes.
    let recovered = diagonal(30.0, 1e-3);
    let carried = first_spectrum.deficit;
    let (_o2, _c2, second) = synchronisation_visit(&recovered, &sigma, None, true, &inputs_at(carried));
    let second_spectrum = second.spectrum().expect("the second rebuild read a spectrum");
    assert!(
        second_spectrum.lambda_min > first_spectrum.lambda_min,
        "the second rebuild measures a larger least eigenvalue"
    );
    assert!(
        second.lambda_floor() < first.lambda_floor(),
        "the floor falls with the evidence rather than ratcheting: {:e} then {:e}",
        first.lambda_floor(),
        second.lambda_floor()
    );
    assert_near(
        second_spectrum.deficit,
        0.0,
        DEFAULT_TOLERANCES.bit_identical,
        "and asks for nothing where the evidence clears it",
    );
    let second_share = second.floor_share().expect("the second reading");
    assert!(
        second_share < first_share,
        "the degradation reading falls with the floor: {first_share:e} then {second_share:e}"
    );

    // And nothing remembers the earlier floor: rebuilding the stretched matrix
    // again derives exactly the floor it derived the first time.
    let (_o3, _c3, again) = synchronisation_visit(&stretched, &sigma, None, true, &inputs_at(0.0));
    assert_near(
        again.lambda_floor(),
        first.lambda_floor(),
        DEFAULT_TOLERANCES.bit_identical,
        "the floor is a function of the spectrum in hand and of nothing else",
    );
}

/// A rebuild on the shape the drift study drove — a definite matrix with a
/// band of coordinates resting at the replenishment floor and a conditioning
/// the floor itself permits — is adopted, and the numbers it was judged on are
/// carried in the failure message so that a refusal here is a diagnosis rather
/// than a surprise. This is the adoption test's own falsifier: a measurement
/// that refuses a healthy rebuild leaves the maintained covariance in place
/// for ever, and a model whose covariance is never replaced is worse off than
/// one that rebuilds too often.
///
/// The geometry matters. The floor bounds the condition number at the largest
/// value a floating-point factorisation is assured to survive, and the
/// residual a correct inverse of a matrix at that conditioning carries is
/// itself proportional to that conditioning. So the two bounds have to be
/// compatible: if the drift threshold a rebuild is judged against is tighter
/// than the residual the arithmetic can deliver at the conditioning the floor
/// permits, every rebuild at that conditioning is refused however healthy it
/// is.
///
/// ´claim:bayes:a-rebuild-on-a-floored-ill-conditioned-but-definite-matrix-is-adopted´
/// ´test:crate:a-floored-geometry-does-not-have-its-rebuilds-refused´
#[test]
fn a_floored_geometry_does_not_have_its_rebuilds_refused() {
    let p = 60;
    let floored = 26;
    let lambda_floor = 1e-3;

    // Coordinates at the replenishment floor, coordinates carrying evidence,
    // and a coupling between them that leaves the matrix definite while
    // pushing its least eigenvalue well below the smallest diagonal entry —
    // which is the state the floor's own requirement warns is compatible with
    // a healthy diagonal.
    #[allow(clippy::cast_precision_loss)] // Test widths are exact in f64.
    let mat = Mat::from_fn(p, p, |i, j| {
        if i == j {
            if i < floored { lambda_floor } else { 1.0 + (i as f64) }
        } else if i < floored && j < floored {
            // A near-singular block on the exhausted coordinates.
            lambda_floor * 0.999
        } else {
            0.0
        }
    });
    let precision = SymmetricMatrix::from_computation(mat);
    let covariance = SymmetricMatrix::identity_scaled(p, 1.0);
    let threshold = synchronisation_error_threshold(p);
    let inputs = VisitInputs {
        labels_since_recompute: 1000,
        diagonal_ratio: 1.0,
        dimensions_at_floor: floored,
        sync_error_threshold: threshold,
        spectral_floor_mass: 0.0,
        // No clamp mass: this rebuild is about rounding, and a model with
        // nothing at the floor attributes none of its drift to the prior.
        clamp_mass: &[],
    };

    let (_outcome, rebuilt, baseline) = synchronisation_visit(&precision, &covariance, None, true, &inputs);
    let spectrum = baseline.spectrum().expect("the geometry has a spectrum");

    assert!(
        spectrum.lambda_min > 0.0,
        "the geometry is definite before the floor is considered: lambda_min {:e}",
        spectrum.lambda_min
    );
    assert!(
        baseline.adopted().expect("the visit rebuilt"),
        "a rebuild on a floored, ill-conditioned, definite matrix is adopted; \
         lambda_max {:e} lambda_min {:e} floor {:e} deficit {:e} kappa {:e} \
         before {:e} after {:e} threshold {threshold:e} verdict {:?}",
        spectrum.lambda_max,
        spectrum.lambda_min,
        spectrum.lambda_floor,
        spectrum.deficit,
        spectrum.kappa(),
        baseline.sync_error_before,
        baseline.sync_error_after().expect("the visit rebuilt"),
        baseline.verdict().expect("the visit rebuilt")
    );
    assert!(rebuilt.is_some(), "and it hands back its pair");
}

/// The component the prior put into the reading is the clamp mass accumulated
/// since the last rebuild, and the mass the model tracks over its life is a
/// different and larger number once a rebuild has happened.
///
/// The model here is driven under decay alone, so the clamp is the only thing
/// adding to the precision matrix and every disagreement in the tracked pair
/// is the clamp's doing. Before any rebuild the two masses coincide and the
/// closed form is exact against the reading. The rebuild then takes the
/// covariance from the precision matrix the clamp has already raised, which
/// absorbs everything accumulated up to that point and leaves the pair
/// consistent; the reading restarts from zero while the tracked mass does not.
/// Driven on from there, the reading is predicted exactly by what the clamp
/// has added since — and the lifetime mass overstates it by half again at this
/// decay and this interval, which is the error a monitor subtracting the wrong
/// mass would make. The consequence is the reason to pin it: the residual the
/// cadence acts on is the reading minus this component, so a component that
/// overstates drives the residual to zero and the cadence stops reacting to
/// rounding at all on exactly the models that carry a clamp.
///
/// ´claim:bayes:the-prior-induced-component-is-the-clamp-mass-accumulated-since-the-last-rebuild´
/// ´test:crate:the-prior-induced-component-is-the-mass-accumulated-since-the-rebuild´
#[test]
fn the_prior_induced_component_is_the_mass_accumulated_since_the_rebuild() {
    use crate::linalg::convert::vec_to_col;
    use crate::model::bayesian::BayesianLinearModel;
    use crate::model::recompute::prior_induced_synchronisation_error;

    let p = 6;
    let lambda_floor = 0.05;
    let gamma = 0.9;

    let mut model = BayesianLinearModel::new(p, 0.1, 1_000_000);
    let phi_hat = vec_to_col(&vec![0.0; p]);
    let decay_only = |model: &mut BayesianLinearModel, labels: usize| {
        for _ in 0..labels {
            let (v, h) = model.covariance().quadratic_form_with_product(&phi_hat);
            model.apply_leverage_bounded_update(&phi_hat, &v, h, 0.0, 0.0, gamma, lambda_floor);
        }
    };

    // Before any rebuild the lifetime mass is the mass since the rebuild,
    // because there has not been one. This is the condition under which the
    // closed form was first pinned, and it is exact.
    decay_only(&mut model, 50);
    let measured_first = synchronisation_error(model.precision(), model.covariance());
    let predicted_first = prior_induced_synchronisation_error(model.covariance(), model.clamp_mass());
    assert!(predicted_first > 1.0, "the contribution is large at this floor and interval");
    assert_near(
        measured_first,
        predicted_first,
        predicted_first * 1e-9,
        "before a rebuild the reading is the lifetime clamp mass seen through the covariance",
    );

    // The rebuild takes the covariance from the precision matrix the clamp has
    // raised, so it absorbs the disagreement rather than leaving it standing.
    let trigger = should_recompute(
        model.precision(),
        50,
        50,
        model.baseline(),
        DEFAULT_KAPPA_GROWTH_FACTOR,
        lambda_floor,
    );
    let inputs = VisitInputs::from_trigger(
        &trigger,
        50,
        synchronisation_error_threshold(p),
        model.spectral_floor_mass(),
        model.clamp_mass_since_rebuild(),
    );
    let (outcome, rebuilt, baseline) =
        synchronisation_visit(model.precision(), model.covariance(), model.baseline(), true, &inputs);
    assert!(
        baseline.adopted().expect("the visit rebuilt"),
        "the rebuild on this geometry is adopted"
    );
    let clamp_mass_at_rebuild = model.clamp_mass().to_vec();
    model.apply_recompute_outcome(&outcome, rebuilt, Some(baseline), 1000);
    assert_below(
        synchronisation_error(model.precision(), model.covariance()),
        1e-12,
        "the rebuild leaves the pair consistent, so the reading restarts from zero",
    );

    // Driven on, the reading is what the clamp has added *since* the rebuild,
    // decayed at the same rate the covariance grew at, and the two decays
    // cancel exactly.
    let further = 10;
    decay_only(&mut model, further);

    // Derived independently of the model: what the clamp added since the
    // rebuild is the lifetime figure less what stood at the rebuild, decayed
    // over the labels since. The model tracks this itself, and the two agree.
    let carried: Vec<f64> = clamp_mass_at_rebuild
        .iter()
        .map(|&m| m * gamma.powi(i32::try_from(further).expect("a small label count")))
        .collect();
    let since_rebuild: Vec<f64> = model
        .clamp_mass()
        .iter()
        .zip(&carried)
        .map(|(&now, &absorbed)| now - absorbed)
        .collect();
    for (coordinate, (&tracked, &derived)) in model.clamp_mass_since_rebuild().iter().zip(&since_rebuild).enumerate() {
        assert_near(
            tracked,
            derived,
            derived * 1e-12,
            &format!("the tracked mass since the rebuild at coordinate {coordinate}"),
        );
    }

    let measured = synchronisation_error(model.precision(), model.covariance());
    let predicted_since = prior_induced_synchronisation_error(model.covariance(), model.clamp_mass_since_rebuild());
    let predicted_lifetime = prior_induced_synchronisation_error(model.covariance(), model.clamp_mass());

    assert_near(
        measured,
        predicted_since,
        predicted_since * 1e-9,
        "after a rebuild the reading is the clamp mass accumulated since it",
    );
    assert!(
        predicted_lifetime > predicted_since * 1.4,
        "the lifetime mass overstates the component it is standing in for: \
         lifetime {predicted_lifetime:e} against {predicted_since:e} since the rebuild, \
         measured {measured:e}"
    );
}

/// A model whose drift is entirely the prior's doing keeps its interval and
/// pays for no factorisation at all, and the rebuild it is made to take
/// anyway leaves both components at zero.
///
/// This is the behaviour the separation exists to produce, and the visit is
/// what completes it. Driven under decay alone, every coordinate rests at the
/// replenishment floor and the clamp is the only thing adding to the precision
/// matrix, so the reading the monitor takes is large and none of it is
/// rounding. The measurement takes that reading, subtracts the prior's part,
/// finds the residual under the threshold, and stops — the model keeps its
/// interval and its pair, and no spectrum is read and no matrix factored. The
/// arrangement this replaces did the subtraction too and reached the same
/// verdict about the interval, but reached it *after* paying for the
/// factorisation the verdict said was unnecessary
/// (´rep:assayer:declined-rebuild-cadence´).
///
/// The forced rebuilds are the other half of the claim. Once a rebuild has
/// been adopted the covariance is the inverse of the precision matrix the
/// clamp has already raised, so there is nothing left for either component to
/// be: the measured reading, the part attributed to the prior, and the
/// residual are all at zero together. That is what makes the attribution safe
/// to publish — it does not leave a standing figure behind on a model that has
/// just been rebuilt.
///
/// ´claim:bayes:a-model-whose-drift-is-all-prior-keeps-its-interval-and-rebuilds-nothing´
/// ´test:crate:a-model-whose-drift-is-all-prior-keeps-its-interval´
#[test]
fn a_model_whose_drift_is_all_prior_keeps_its_interval() {
    use crate::linalg::convert::vec_to_col;
    use crate::model::bayesian::BayesianLinearModel;

    let p = 6;
    let lambda_floor = 0.05;
    let gamma = 0.9;
    let configured = 1000;
    let threshold = synchronisation_error_threshold(p);

    let mut model = BayesianLinearModel::new(p, 0.1, configured);
    let phi_hat = vec_to_col(&vec![0.0; p]);
    for _ in 0..50 {
        let (v, h) = model.covariance().quadratic_form_with_product(&phi_hat);
        model.apply_leverage_bounded_update(&phi_hat, &v, h, 0.0, 0.0, gamma, lambda_floor);
    }

    // `force` is the disposition the degenerate arm sets: go to the rebuild
    // whatever the measurement says. False is the ordinary visit.
    let visit = |model: &BayesianLinearModel, labels: u32, force: bool| {
        let trigger = should_recompute(
            model.precision(),
            labels,
            labels,
            model.baseline(),
            DEFAULT_KAPPA_GROWTH_FACTOR,
            lambda_floor,
        );
        let inputs = VisitInputs::from_trigger(
            &trigger,
            labels,
            threshold,
            model.spectral_floor_mass(),
            model.clamp_mass_since_rebuild(),
        );
        synchronisation_visit(model.precision(), model.covariance(), model.baseline(), force, &inputs)
    };

    let (outcome, rebuilt, baseline) = visit(&model, 50, false);
    // The clamp has put more into the reading than the threshold allows, and
    // an adopted rebuild is the only thing that removes it — so the visit
    // rebuilds. What it does *not* do is shorten, which is the whole of the
    // claim (´dec:posterior:adaptive-cadence´).
    assert!(outcome.is_rebuild(), "the clamp's own contribution calls for a rebuild");
    assert!(
        matches!(
            outcome,
            RecomputeOutcome::Rebuilt {
                drift_was_real: false,
                ..
            }
        ),
        "and it is the clamp that called for it, not drift: {outcome:?}"
    );
    assert!(rebuilt.is_some(), "so the pair reaches the caller");
    assert!(baseline.rebuild.is_some(), "and a rebuild record is written");

    // The whole reading is large, and the prior accounts for all of it.
    assert!(
        baseline.sync_error_before > 1.0,
        "the reading is large at this floor and interval: {:e}",
        baseline.sync_error_before
    );
    assert_near(
        baseline.sync_error_prior_induced,
        baseline.sync_error_before,
        baseline.sync_error_before * 1e-9,
        "the prior accounts for the whole reading on a model driven by decay alone",
    );
    assert_below(
        baseline.sync_error_residual,
        threshold,
        "and what is left of it is below the threshold the cadence tests",
    );

    let interval_before = model.n_recompute_effective();
    model.apply_recompute_outcome(&outcome, rebuilt, Some(baseline), configured);
    assert_eq!(
        model.n_recompute_effective(),
        interval_before,
        "a reading the cadence can attribute to the prior does not shorten the interval"
    );
    assert_eq!(model.sync_error_shortenings(), 0, "and is not counted as a shortening");
    assert_eq!(model.total_recomputes(), 1, "the rebuild that absorbed the clamp is counted");
    assert_eq!(model.alarms(), 0, "and it was adopted, so nothing is alarming");
    assert_eq!(
        model.consecutive_clean_recomputes(),
        1,
        "a rebuild that found no drift still counts toward recovery",
    );

    // The measured reading is still published whole, which is what the
    // specification defines the quantity to be.
    assert_near(
        model.last_sync_error(),
        baseline.sync_error_before,
        baseline.sync_error_before * 1e-12,
        "the published reading is the measured one, both components in it",
    );

    // And measured again, neither component has anything left in it: the
    // rebuild above absorbed the clamp mass into the pair it installed.
    let (_second_outcome, _second_pair, second) = visit(&model, 1, true);
    assert_below(second.sync_error_before, 1e-12, "the rebuilt pair measures no drift");
    assert_below(
        second.sync_error_prior_induced,
        1e-12,
        "the adopted rebuild absorbed the clamp mass, so the prior's component is gone",
    );
    assert_below(second.sync_error_residual, 1e-12, "and so is the residual");
}

/// A model whose drift is entirely rounding has a residual equal to its whole
/// reading, and the cadence shortens on it exactly as it did before.
///
/// The falsifier for the test above. Separating the components is only worth
/// doing if it leaves the case the monitor exists for untouched, so here the
/// covariance is stretched off the inverse by a tenth on one coordinate with
/// no clamp mass anywhere: the prior's component is exactly zero, the residual
/// is the whole reading, and the interval halves. Had the subtraction been
/// wrong in the other direction — attributing to the prior something the
/// arithmetic put there — this is where it would show, and the cost would be a
/// model drifting on a long interval with nothing to say so.
///
/// ´claim:bayes:a-model-whose-drift-is-all-rounding-has-a-residual-equal-to-its-reading-and-shortens-as-before´
/// ´test:crate:a-model-whose-drift-is-all-rounding-shortens-as-before´
#[test]
fn a_model_whose_drift_is_all_rounding_shortens_as_before() {
    let p = 5;
    let precision = SymmetricMatrix::identity_scaled(p, 1.0);
    let mut stretched = Mat::<f64>::identity(p, p);
    stretched[(0, 0)] = 1.1;
    let covariance = SymmetricMatrix::from_computation(stretched);
    let threshold = synchronisation_error_threshold(p);

    let inputs = VisitInputs {
        labels_since_recompute: 1000,
        diagonal_ratio: 1.0,
        dimensions_at_floor: 0,
        sync_error_threshold: threshold,
        spectral_floor_mass: 0.0,
        // Nothing at the floor, so the clamp has added nothing and there is
        // nothing to attribute to the prior.
        clamp_mass: &[],
    };
    let (outcome, rebuilt, baseline) = synchronisation_visit(&precision, &covariance, None, true, &inputs);

    assert!(baseline.sync_error_before > threshold, "the stretch is real drift");
    assert_eq!(
        baseline.sync_error_prior_induced, 0.0,
        "a model with no clamp mass attributes nothing to the prior"
    );
    assert_eq!(
        baseline.sync_error_residual, baseline.sync_error_before,
        "so the residual is the whole reading"
    );
    assert!(baseline.adopted().expect("the visit rebuilt"), "and the rebuild is adopted");
    assert!(rebuilt.is_some(), "so the pair it produced is handed back");

    let mut health = ModelPrecisionHealth::with_interval(1000);
    assert!(
        health.update_interval(1000, &outcome),
        "the cadence shortens on a residual above the threshold"
    );
    assert_eq!(health.n_recompute_effective, 500);
    assert_eq!(health.sync_error_shortenings, 1);
}

/// At the drift study's own geometry the reading a floored model publishes is
/// the prior's in full, its residual is rounding, and the closed form predicts
/// the study's measured figures at both cadences from the interval alone.
///
/// This is the reproduction that makes the study's numbers readable. The
/// geometry is the one the sweep drives: twenty-six coordinates resting at the
/// replenishment floor with no evidence arriving on them, the rest taking
/// evidence, at the outcome-axis model's own forgetting rate. Because the
/// unevidenced coordinates form their own block, the covariance over that
/// block is exactly the reciprocal of the floor and the arithmetic closes in
/// hand: the clamp contributes `λ_floor·(1 − γ^k)` per coordinate since the
/// rebuild, the covariance over the block has grown by `γ^{-k}` since it, the
/// floor cancels between the two, and the reading is
/// `√n · (1 − γ^k) · γ^{-k}` for `n` floored coordinates.
///
/// That expression is the whole content of the study's readings. At an
/// interval of a hundred labels it gives 8.83, which is what the sweep
/// measured on the outcome-axis model while its cadence was pinned at the
/// floor; at four hundred it gives 278, which is what the same model measures
/// once the cadence stops shortening on a quantity it cannot remove. The
/// reading grew because the interval did, and both figures are the prior's.
/// The floor's own value does not appear, which is why sweeping the floor
/// across two decimal orders left the error where it was.
///
/// The direction matters as much as the size. A large figure here is a model
/// leaning on its prior along directions no evidence has reached, which is
/// conservative — the covariance is wider than the inverse exactly there — and
/// it is not drift. What an operator reads for drift is the residual, and at
/// this geometry the residual is the accumulated rounding of the maintenance
/// and sits below the threshold by orders. A monitor publishing only the
/// measured figure cannot tell the two apart, which is the whole reason the
/// components are separated.
///
/// ´claim:bayes:at-the-studys-geometry-the-published-reading-is-the-prior-in-full-and-the-residual-is-rounding´
/// ´test:crate:the-studys-reading-is-the-prior-and-its-residual-is-rounding´
#[test]
fn the_studys_reading_is_the_prior_and_its_residual_is_rounding() {
    use crate::linalg::convert::vec_to_col;
    use crate::model::bayesian::BayesianLinearModel;
    use crate::model::recompute::{prior_induced_synchronisation_error, residual_synchronisation_error};

    /// The three figures this reproduction exists to publish. Captured by the
    /// harness on a normal run and visible when it is asked for them.
    #[allow(clippy::print_stdout)] // Justified: the decomposition of a figure a reader would otherwise misread is this test's output, and libtest captures it unless a reader asks.
    fn report(line: &str) {
        println!("{line}");
    }

    let floored = 26;
    let evidenced = 14;
    let p = floored + evidenced;
    let lambda_floor = 0.001;
    let gamma = 0.99;
    let interval: u32 = 400;
    let threshold = synchronisation_error_threshold(p);

    let mut model = BayesianLinearModel::new(p, 0.1, interval);

    // Evidence reaches the first `evidenced` coordinates and never the rest,
    // so the rest decay to the floor and the clamp holds them there. The two
    // groups do not mix, which is what makes the covariance over the floored
    // block exactly diagonal and the arithmetic closed.
    let label = |model: &mut BayesianLinearModel, at: usize| {
        let mut values = vec![0.0; p];
        values[at % evidenced] = 1.0;
        let phi_hat = vec_to_col(&values);
        let (v, h) = model.covariance().quadratic_form_with_product(&phi_hat);
        model.apply_leverage_bounded_update(&phi_hat, &v, h, 1.0, 1.0, gamma, lambda_floor);
    };

    // Burn in until the unevidenced coordinates have reached the floor.
    for i in 0..600 {
        label(&mut model, i);
    }

    let rebuild = |model: &BayesianLinearModel, labels: u32| {
        let trigger = should_recompute(
            model.precision(),
            labels,
            labels,
            model.baseline(),
            DEFAULT_KAPPA_GROWTH_FACTOR,
            lambda_floor,
        );
        let inputs = VisitInputs::from_trigger(
            &trigger,
            labels,
            threshold,
            model.spectral_floor_mass(),
            model.clamp_mass_since_rebuild(),
        );
        (
            trigger,
            synchronisation_visit(model.precision(), model.covariance(), model.baseline(), true, &inputs),
        )
    };

    let (trigger, (outcome, rebuilt, baseline)) = rebuild(&model, 600);
    assert_eq!(
        trigger.dimensions_at_floor(),
        floored,
        "the geometry is the study's: {floored} coordinates resting at the floor"
    );
    assert!(
        baseline.adopted().expect("the visit rebuilt"),
        "the rebuild on this geometry is adopted"
    );
    model.apply_recompute_outcome(&outcome, rebuilt, Some(baseline), interval);

    // One interval on from the rebuild, which is where the monitor reads.
    let since = usize::try_from(interval).expect("a small interval");
    for i in 600..600 + since {
        label(&mut model, i);
    }

    let measured = synchronisation_error(model.precision(), model.covariance());
    let prior_induced = prior_induced_synchronisation_error(model.covariance(), model.clamp_mass_since_rebuild());
    let residual = residual_synchronisation_error(measured, prior_induced);

    // The closed form, in hand: √n · (1 − γ^k) · γ^{-k}. The floor cancels.
    let k = i32::try_from(interval).expect("a small interval");
    let decayed = gamma.powi(k);
    #[allow(clippy::cast_precision_loss)]
    let closed_form = (floored as f64).sqrt() * (1.0 - decayed) / decayed;

    report(&format!(
        "STUDY GEOMETRY floored={floored} width={p} gamma={gamma} interval={interval}\n  \
         measured      = {measured:.6e}\n  \
         prior_induced = {prior_induced:.6e}\n  \
         residual      = {residual:.6e}\n  \
         closed_form   = {closed_form:.6e}   threshold = {threshold:.6e}\n  \
         residual/measured = {:.3e}",
        residual / measured
    ));

    assert_near(
        measured,
        closed_form,
        closed_form * 1e-6,
        "the reading at this geometry is what the closed form predicts from the interval alone",
    );
    assert_near(
        prior_induced,
        measured,
        measured * 1e-6,
        "and the prior accounts for the whole of it",
    );
    assert_below(
        residual,
        threshold,
        "so what is left for the cadence to act on is below the threshold",
    );
    assert!(
        residual < measured * 1e-9,
        "the residual is orders below the reading it is taken out of: residual {residual:e} against measured {measured:e}"
    );
}

/// An isolated model driven to a synchronised state at the dense schedule's
/// own conditioning adopts every rebuild, and the fresh inverse is the better
/// of the two readings by a factor of three.
///
/// This is the falsifier for the reading the dense schedule gives, and it
/// falsifies the easy explanation of it. That schedule shows its operational
/// and sister models declining nine of every ten rebuilds while the drift
/// they measured beforehand sits three orders under the threshold the cadence
/// tests, which pins their interval at the adaptive floor and throws away a
/// cubic factorisation every hundred labels. The obvious reading is that
/// declines are what a model at that conditioning does: the maintained pair's
/// drift and a fresh inverse's own rounding residual arrive at the same
/// magnitude, the comparison between them is decided by rounding noise, and
/// roughly half of them are lost. That reading is wrong, and this is where it
/// is shown to be wrong.
///
/// The model here is driven through the real update path at the operational
/// family's own forgetting rate, with a four-decade geometric spread across
/// the feature magnitudes chosen so that the top of the precision diagonal
/// settles near `1e5` and the bottom just above the replenishment floor. That
/// is a diagonal ratio of `8.776e7` at a width of four hundred, which is the
/// conditioning the schedule reports. Ten rebuilds at the adaptive floor's
/// own interval follow, and every one of them is adopted: the drift standing
/// before the last is `9.220e-12`, the drift the rebuilt pair carries is
/// `4.096e-12`, the mean of the ratio over the ten is `0.318`, and the
/// threshold at this width is `4.000e-4` — so both readings are under it by
/// seven orders and the fresh inverse is the better one every time. The same
/// measurement at widths two hundred, eight hundred and twelve hundred gives
/// the same verdict, so the width is not what separates the two cases either.
///
/// What that leaves is the finding. The declines are a property of the
/// schedule's matrices rather than of the conditioning figure they publish,
/// and the figure is the reason the two can come apart: the diagonal ratio is
/// a lower bound on the condition number and not a measurement of it
/// (´dec:posterior:recomputation-trigger´), so a matrix built from a
/// competitive geometry — where a cell and the parent it was split from fire
/// together, where the width grows between one rebuild and the next, and
/// where coordinates are added at the prior while the schedule runs — can
/// carry a true conditioning far above the ratio it reports while an isolated
/// model at the same ratio does not. A rebuild's inverse is accurate to the
/// true conditioning and not to the reported one, which is what puts the
/// schedule's `after` above its `before` and this model's below.
///
/// The measurement also says what the instrument cannot yet say. The adoption
/// test declines on either of two conditions — the drift after the rebuild is
/// not at or below the drift before it, or it is not under the threshold —
/// and the published precision detail carries the drift before the last
/// rebuild and not the drift after it, so a decline in the schedule's output
/// cannot be attributed to one condition or the other. The model's own
/// baseline holds both figures.
///
/// ´claim:bayes:an-isolated-model-at-the-schedules-conditioning-adopts-every-rebuild´
/// ´test:crate:an-isolated-model-at-the-schedules-conditioning-adopts-every-rebuild´
#[test]
fn an_isolated_model_at_the_schedules_conditioning_adopts_every_rebuild() {
    /// The table this measurement exists to publish.
    #[allow(clippy::print_stdout)] // Justified: this test's finding is a table of measured pairs, and a reading no one sees is not one; libtest captures it unless a reader asks.
    fn report(line: &str) {
        println!("{line}");
    }

    let width = 400;
    let reading = measure_rebuilds_at_the_fixed_point(width, 3000, 10);
    report(&format!(
        "ISOLATED MODEL width={width} gamma=0.9995 spread=4-decades burn=3000\n  \
         kappa-diagonal = {:.3e}   threshold = {:.3e}\n  \
         before = {:.6e}   after = {:.6e}   mean after/before = {:.6}\n  \
         adopted {}/{}   declined {}",
        reading.diagonal_ratio,
        reading.threshold,
        reading.last_before,
        reading.last_after,
        reading.mean_ratio,
        reading.adopted,
        reading.attempts,
        reading.attempts - reading.adopted,
    ));

    assert!(
        reading.diagonal_ratio > 1e7,
        "the conditioning is the schedule's own, not a well-behaved one: {:e}",
        reading.diagonal_ratio
    );
    assert_eq!(
        reading.adopted, reading.attempts,
        "every rebuild at this conditioning is adopted: {} of {} against a mean ratio of {:.6}",
        reading.adopted, reading.attempts, reading.mean_ratio
    );
    assert!(
        reading.mean_ratio < 1.0,
        "the fresh inverse is the better of the two readings on average: {:.6}",
        reading.mean_ratio
    );
    assert!(
        reading.last_after < reading.last_before,
        "and on the last rebuild in particular: after {:e} against before {:e}",
        reading.last_after,
        reading.last_before
    );
    assert_below(
        reading.last_before,
        reading.threshold * 1e-4,
        "the drift standing before the rebuild is orders under the threshold the cadence tests",
    );
    assert_below(
        reading.last_after,
        reading.threshold * 1e-4,
        "and so is the drift the rebuilt pair carries",
    );
}

/// What one run of the measurement above read off a model.
struct FixedPointReading {
    /// Rebuilds the model's own measurement carried
    /// (´dec:posterior:measured-adoption´).
    adopted: u32,
    /// Rebuilds attempted, so that the two together are the fraction the
    /// finding is stated in.
    attempts: u32,
    /// The mean over the attempts of the drift after the rebuild divided by
    /// the drift before it: the quantity the adoption test's first condition
    /// compares against one.
    mean_ratio: f64,
    /// The drift standing against the maintained pair at the last rebuild.
    last_before: f64,
    /// The drift the last rebuild's own pair carries, measured against the
    /// matrix the model would hold if it took it.
    last_after: f64,
    /// The cheap conditioning estimate at the last rebuild: a lower bound on
    /// the condition number, and the figure the instrument publishes.
    diagonal_ratio: f64,
    /// The width-scaled threshold both readings are held to
    /// (´def:monitoring:synchronisation-error´).
    threshold: f64,
}

/// Drives a model of the given width to a synchronised state through the real
/// update path and then calls the rebuild at successive intervals, recording
/// what the adoption test was handed each time.
fn measure_rebuilds_at_the_fixed_point(p: usize, burn_in: usize, rebuilds: usize) -> FixedPointReading {
    use crate::linalg::convert::vec_to_col;
    use crate::model::bayesian::BayesianLinearModel;
    use crate::model::update::GAMMA_LABEL_OPERATIONAL;

    let prior_precision = DEFAULT_LAMBDA_FLOOR;
    let lambda_floor = DEFAULT_LAMBDA_FLOOR;
    let gamma = GAMMA_LABEL_OPERATIONAL;
    let weight = 150.0;
    let decades = 4.0;
    let interval: u32 = N_RECOMPUTE_FLOOR;
    let per_interval = usize::try_from(interval).expect("a small interval");
    let threshold = synchronisation_error_threshold(p);

    // A geometric spread across the coordinates. Under this decay every
    // coordinate settles at the evidence it receives divided by one less the
    // rate, so a spread of four decades in the feature magnitudes is eight in
    // the precision diagonal and the weight sets where the band sits: the top
    // lands near 1e5 and the bottom just above the replenishment floor, which
    // is a conditioning near 1e8 with nothing resting at the floor.
    let last = f64::from(u32::try_from(p - 1).expect("a width in the hundreds"));
    let scale: Vec<f64> = (0..p)
        .map(|j| {
            let coordinate = f64::from(u32::try_from(j).expect("a width in the hundreds"));
            10.0_f64.powf(-decades * coordinate / last)
        })
        .collect();

    // A deterministic stream, so the geometry is the same on every run and on
    // every host: a measurement that moves between runs is not one.
    let mut state: u64 = 0x2545_F491_4F6C_DD1D;
    let mut uniform = move || {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let bits = u32::try_from(state >> 40).expect("twenty-four bits");
        f64::from(bits) / f64::from(1_u32 << 23) - 1.0
    };

    let mut model = BayesianLinearModel::new(p, prior_precision, interval);
    let mut drive = |model: &mut BayesianLinearModel, labels: usize| {
        for _ in 0..labels {
            let values: Vec<f64> = scale.iter().map(|&s| s * uniform()).collect();
            let phi_hat = vec_to_col(&values);
            let (v, h) = model.covariance().quadratic_form_with_product(&phi_hat);
            model.apply_leverage_bounded_update(&phi_hat, &v, h, weight, 1.0, gamma, lambda_floor);
        }
    };

    drive(&mut model, burn_in);

    let mut adopted = 0_u32;
    let mut ratio_sum = 0.0_f64;
    let mut last_before = 0.0_f64;
    let mut last_after = 0.0_f64;
    let mut diagonal_ratio = 0.0_f64;
    for _ in 0..rebuilds {
        drive(&mut model, per_interval);
        let (trigger, outcome, rebuilt, baseline) = {
            let trigger = should_recompute(
                model.precision(),
                interval,
                interval,
                model.baseline(),
                DEFAULT_KAPPA_GROWTH_FACTOR,
                lambda_floor,
            );
            let inputs = VisitInputs::from_trigger(
                &trigger,
                interval,
                threshold,
                model.spectral_floor_mass(),
                model.clamp_mass_since_rebuild(),
            );
            let (outcome, rebuilt, baseline) =
                synchronisation_visit(model.precision(), model.covariance(), model.baseline(), true, &inputs);
            (trigger, outcome, rebuilt, baseline)
        };
        adopted += u32::from(baseline.adopted().expect("the visit rebuilt"));
        ratio_sum += baseline.sync_error_after().expect("the visit rebuilt") / baseline.sync_error_before;
        last_before = baseline.sync_error_before;
        last_after = baseline.sync_error_after().expect("the visit rebuilt");
        diagonal_ratio = trigger.diagonal_ratio();
        model.apply_recompute_outcome(&outcome, rebuilt, Some(baseline), interval);
    }

    let attempts = u32::try_from(rebuilds).expect("a small rebuild count");
    FixedPointReading {
        adopted,
        attempts,
        mean_ratio: ratio_sum / f64::from(attempts),
        last_before,
        last_after,
        diagonal_ratio,
        threshold,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// The visit: the measurement, the rebuild it calls for, and the alarm
// (´dec:posterior:recomputation-trigger´), (´dec:posterior:measured-adoption´)
// ═══════════════════════════════════════════════════════════════════════════════

/// Inputs for a visit at one threshold, with nothing at the replenishment
/// floor. The tests below vary the threshold deliberately, because the
/// threshold is what separates a measurement the cadence acts on from one it
/// does not.
fn visit_inputs(threshold: f64) -> VisitInputs<'static> {
    VisitInputs {
        labels_since_recompute: 1000,
        diagonal_ratio: 1.0,
        dimensions_at_floor: 0,
        sync_error_threshold: threshold,
        spectral_floor_mass: 0.0,
        clamp_mass: &[],
    }
}

/// A matrix whose fresh inverse is dominated by cancellation, and that
/// inverse in closed form.
///
/// `B = [[1, 1], [1, 1 + δ]]` has determinant exactly `δ`, so its inverse is
/// `(1/δ)·[[1 + δ, −1], [−1, 1]]` and both matrices are written down rather
/// than factored. At `δ = 10⁻¹⁰` the inverse's entries are of order `10¹⁰`
/// while their products cancel down to the identity, which is the structure a
/// competitive geometry produces and the structure the resolution exists to
/// read (´rep:assayer:declined-rebuild-cadence´).
fn cancelling_pair(delta: f64) -> (SymmetricMatrix, SymmetricMatrix) {
    let precision =
        SymmetricMatrix::from_computation(Mat::from_fn(2, 2, |i, j| if i == 1 && j == 1 { 1.0 + delta } else { 1.0 }));
    let covariance = SymmetricMatrix::from_computation(Mat::from_fn(2, 2, |i, j| {
        let entry = if i == j {
            if i == 0 { 1.0 + delta } else { 1.0 }
        } else {
            -1.0
        };
        entry / delta
    }));
    (precision, covariance)
}

/// A reading's resolution is read off the operands and not off the answer, so
/// a well-scaled pair reports a resolution no meaningful reading approaches
/// while a cancelling pair reports one that swallows its own reading whole.
///
/// This is the derivation's content. The bound on a floating-point matrix
/// product is `|fl(BΣ) − BΣ| ≤ γ_p·|B||Σ|` entrywise, so what limits the
/// reading is the size of the products that were summed, not the size of what
/// they summed to. Scaling the answer would give the same figure for both
/// pairs here and would be wrong for both.
///
/// ´claim:bayes:a-synchronisation-readings-resolution-is-read-off-its-operands´
/// ´test:crate:a-readings-resolution-is-read-off-its-operands´
#[test]
fn a_readings_resolution_is_read_off_its_operands() {
    // Well scaled: the pair is its own inverse and nothing cancels.
    let identity = SymmetricMatrix::identity_scaled(2, 1.0);
    let plain = synchronisation_reading(&identity, &identity);
    assert_below(
        plain.resolution,
        1e-14,
        "a pair with no cancellation in it can be read to nearly the last bit",
    );

    // Cancelling: the same width, and a resolution eight orders larger.
    let (precision, covariance) = cancelling_pair(1e-10);
    let cancelling = synchronisation_reading(&precision, &covariance);
    assert!(
        cancelling.resolution > plain.resolution * 1e8,
        "the cancelling pair's reading is limited by its operands, not its answer: \
         {:e} against {:e}",
        cancelling.resolution,
        plain.resolution
    );

    // And the reading it produces is itself under that level, which is what
    // makes the figure a resolution rather than a tolerance: there is nothing
    // in this reading but the rounding of the product.
    assert!(
        cancelling.measured <= cancelling.resolution,
        "the whole reading is at its own resolution: {:e} against {:e}",
        cancelling.measured,
        cancelling.resolution
    );

    // The growth factor is the width's own, with no constant beside it.
    assert_near(
        product_error_growth(2),
        2.0 * (f64::EPSILON / 2.0) / 2.0f64.mul_add(-(f64::EPSILON / 2.0), 1.0),
        DEFAULT_TOLERANCES.bit_identical,
        "the growth factor is gamma_p and nothing else",
    );
}

/// A residual at the resolution of the reading it was taken out of is
/// published as such and is *not* thereby good: the flag is a reading and the
/// threshold is the verdict.
///
/// The refinement was proposed the other way round — an at-resolution residual
/// would be good however far above the threshold it stood — and the falsifier
/// vetoed that half of it. The resolution is computed from the operands, so a
/// pair that has diverged by orders reports a resolution large enough to
/// excuse any residual at all; the schedule's outcome-axis model reached a
/// residual of `3.0e2` against a threshold of `1.2e-3` with the flag still
/// true, and every measurement in that state called for nothing while the
/// model's pair ceased to be an inverse pair
/// (´rep:assayer:declined-rebuild-cadence´). What survives is the reading,
/// which is true and worth publishing: this residual really is the rounding of
/// the subtraction that produced it.
///
/// ´claim:bayes:an-at-resolution-residual-is-published-and-does-not-excuse-the-threshold´
/// ´test:crate:a-residual-at-its-readings-resolution-is-good´
#[test]
fn a_residual_at_its_readings_resolution_is_good() {
    // The cancelling pair: its whole reading is rounding.
    let (precision, covariance) = cancelling_pair(1e-10);
    let reading = synchronisation_reading(&precision, &covariance);
    assert!(reading.measured > 0.0, "the premise is a reading that is not exactly zero");

    // A threshold under the residual, so the threshold arm calls it bad.
    let inputs = visit_inputs(reading.measured / 2.0);
    let measurement = measure_synchronisation(&precision, &covariance, &inputs);
    assert!(
        measurement.residual > inputs.sync_error_threshold,
        "the residual is over the threshold"
    );
    assert!(
        measurement.at_resolution,
        "and it is at the resolution of its own reading, which is published"
    );
    assert!(
        !measurement.good,
        "but the flag does not excuse the threshold, which is the vetoed half"
    );

    // Under the threshold the same pair is good and still reads at its
    // resolution: the flag is orthogonal to the verdict.
    let generous = visit_inputs(reading.measured * 2.0);
    let under = measure_synchronisation(&precision, &covariance, &generous);
    assert!(under.at_resolution);
    assert!(under.good, "a residual under its threshold calls for nothing");

    // The control: a pair stretched off its inverse, whose reading is real
    // drift and can be read exactly.
    let identity = SymmetricMatrix::identity_scaled(2, 1.0);
    let stretched = SymmetricMatrix::identity_scaled(2, 1.1);
    let drift = synchronisation_reading(&identity, &stretched);
    let control_inputs = visit_inputs(drift.measured / 2.0);
    let control = measure_synchronisation(&identity, &stretched, &control_inputs);
    assert!(
        !control.at_resolution,
        "real drift stands far above the resolution of a pair that does not cancel: {:e} against {:e}",
        control.residual, control.reading.resolution
    );
    assert!(!control.good, "so the measurement calls for a rebuild");
}

/// A clamp contribution over the model's threshold calls for a rebuild even
/// where the residual is spotless, and a model carrying no clamp is untouched
/// by that arm.
///
/// The prior-induced component is the replenishment clamp's contribution since
/// the last rebuild, and an adopted rebuild is the only thing that removes it.
/// A measurement that looked only at the residual would therefore report a
/// healthy model while the pair it holds drifted apart without limit — which is
/// what the falsifier measured before this arm existed: a reading running from
/// `1.7e2` to `2.0e10` across a single rebuild, no spectral floor taken for
/// thousands of labels, and a refused factorisation that stopped the label path
/// (´rep:assayer:declined-rebuild-cadence´).
///
/// ´claim:bayes:a-clamp-contribution-over-the-threshold-calls-for-a-rebuild´
/// ´test:crate:a-clamp-contribution-over-the-threshold-calls-for-a-rebuild´
#[test]
fn a_clamp_contribution_over_the_threshold_calls_for_a_rebuild() {
    // A consistent pair, so the residual is nothing at all.
    let identity = SymmetricMatrix::identity_scaled(4, 1.0);
    let threshold = synchronisation_error_threshold(4);

    // No clamp mass: the component is exactly zero and the measurement is
    // good, which is the case the restructure exists for.
    let clean = measure_synchronisation(&identity, &identity, &visit_inputs(threshold));
    assert_eq!(clean.prior_induced, 0.0, "a model with no clamp attributes nothing");
    assert!(clean.good, "so nothing is rebuilt");

    // A clamp contribution far over the threshold, with the residual still at
    // zero because the reading is entirely the clamp's.
    let mass = [1.0_f64, 1.0, 1.0, 1.0];
    let inputs = VisitInputs {
        labels_since_recompute: 1000,
        diagonal_ratio: 1.0,
        dimensions_at_floor: 4,
        sync_error_threshold: threshold,
        spectral_floor_mass: 0.0,
        clamp_mass: &mass,
    };
    let clamped = measure_synchronisation(&identity, &identity, &inputs);
    assert!(
        clamped.prior_induced > threshold,
        "the clamp has put more into the reading than the threshold allows: {:e} against {:e}",
        clamped.prior_induced,
        threshold
    );
    assert!(
        !clamped.good,
        "so the measurement calls for the rebuild that is the only thing which removes it"
    );
}

/// A good measurement rebuilds nothing, records what it read, and leaves the
/// interval where it was; a bad one rebuilds and its adopted result shortens
/// the interval.
///
/// The two halves are the restructure. The counters separate them — a
/// measurement is not a recompute — and the baseline separates them too: a
/// measurement writes no rebuild record, so the readings a reader sees beside
/// one are the readings of the visit itself.
///
/// ´claim:bayes:a-good-measurement-rebuilds-nothing-and-a-bad-one-rebuilds-and-shortens´
/// ´test:crate:a-good-measurement-rebuilds-nothing´
#[test]
fn a_good_measurement_rebuilds_nothing() {
    use crate::model::bayesian::BayesianLinearModel;

    // Good: the model's own pair, which a fresh model holds exactly.
    let mut healthy = BayesianLinearModel::new(5, 0.1, 1000);
    let inputs = visit_inputs(synchronisation_error_threshold(5));
    let (outcome, rebuilt, baseline) =
        synchronisation_visit(healthy.precision(), healthy.covariance(), healthy.baseline(), false, &inputs);
    assert!(outcome.is_measurement(), "a consistent pair calls for nothing");
    assert!(rebuilt.is_none(), "so no pair is handed back");
    assert!(baseline.rebuild.is_none(), "and no rebuild record is written");
    assert_eq!(baseline.labels_at_record, 1000, "the record places its readings");

    healthy.apply_recompute_outcome(&outcome, rebuilt, Some(baseline), 1000);
    assert_eq!(healthy.n_recompute_effective(), 1000, "the interval is untouched");
    assert_eq!(healthy.measurements(), 1, "and the visit is counted as a measurement");
    assert_eq!(healthy.total_recomputes(), 0, "no factorisation was paid for");
    assert_eq!(healthy.alarms(), 0);

    // Bad: the same precision matrix against a covariance stretched off it.
    let precision = SymmetricMatrix::identity_scaled(5, 1.0);
    let stretched = SymmetricMatrix::identity_scaled(5, 1.1);
    let (outcome, rebuilt, baseline) = synchronisation_visit(&precision, &stretched, None, false, &inputs);
    assert!(outcome.is_rebuild(), "real drift calls for a rebuild");
    assert!(!outcome.is_alarm(), "which improves on the pair and is adopted");
    assert!(rebuilt.is_some(), "so the pair reaches the caller");
    assert!(
        baseline.adopted().expect("the visit rebuilt"),
        "and the record says it was adopted"
    );

    let mut drifting = BayesianLinearModel::new(5, 0.1, 1000);
    drifting.apply_recompute_outcome(&outcome, rebuilt, Some(baseline), 1000);
    assert_eq!(drifting.n_recompute_effective(), 500, "a needed rebuild shortens");
    assert_eq!(drifting.total_recomputes(), 1);
    assert_eq!(drifting.measurements(), 0);
    assert_eq!(drifting.alarms(), 0);
}

/// A rebuild that was needed and produced no improvement raises the alarm,
/// keeps the pair the model already held, and takes the interval to its floor.
///
/// The threshold arm of the adoption test is what fails here, and it is made
/// to fail by asking the rebuild for a drift under zero — a demand no
/// factorisation can meet. What the reading is about is not the arithmetic of
/// the demand but the disposition: the model keeps its pair, the alarm counts,
/// and the interval goes to the floor in one step rather than halving
/// (´dec:posterior:measured-adoption´).
///
/// ´claim:bayes:a-rebuild-that-is-no-better-raises-the-alarm-and-keeps-the-pair´
/// ´test:crate:a-rebuild-that-is-no-better-raises-the-alarm´
#[test]
fn a_rebuild_that_is_no_better_raises_the_alarm() {
    use crate::model::bayesian::BayesianLinearModel;

    let precision = SymmetricMatrix::identity_scaled(5, 1.0);
    let stretched = SymmetricMatrix::identity_scaled(5, 1.1);

    // A threshold of zero: the measurement is bad, the factorisation is
    // clean, and no drift after it can come in under the bar.
    let inputs = visit_inputs(0.0);
    let (outcome, rebuilt, baseline) = synchronisation_visit(&precision, &stretched, None, false, &inputs);

    assert!(outcome.is_alarm(), "a rebuild that cannot improve enough is the alarm");
    assert!(
        !outcome.is_cascade_terminus(),
        "and it is not a terminus: the factorisation succeeded"
    );
    assert_eq!(outcome.refused_pivot(), None);
    assert!(rebuilt.is_none(), "the pair the model holds is kept");
    assert_eq!(
        baseline.verdict().expect("the visit rebuilt"),
        RebuildVerdict::Clean,
        "the factorisation's own verdict is recorded beside the refusal to adopt"
    );
    assert!(!baseline.adopted().expect("the visit rebuilt"));

    let mut model = BayesianLinearModel::new(5, 0.1, 1000);
    let held = model.covariance().clone();
    model.apply_recompute_outcome(&outcome, rebuilt, Some(baseline), 1000);

    assert_eq!(
        model.n_recompute_effective(),
        N_RECOMPUTE_FLOOR,
        "the alarm goes to the floor in one step"
    );
    assert_eq!(model.alarms(), 1);
    assert_eq!(model.total_recomputes(), 1, "the factorisation was paid for");
    assert_eq!(model.measurements(), 0);
    assert_eq!(model.cascade_terminus_events(), 0);
    for i in 0..model.dim() {
        assert_eq!(
            model.covariance().diagonal_element(i).to_bits(),
            held.diagonal_element(i).to_bits(),
            "the model kept the pair it had at coordinate {i}"
        );
    }
}

/// A refused factorisation raises the alarm and is a terminus with it.
///
/// The witness is a matrix outside the definite cone whose diagonal is
/// positive, so the fast check does not divert it to the rebuild and the
/// measurement is what sends it there: the drift on this pair is real, large,
/// and nowhere near its own resolution. The factorisation then refuses, the
/// model keeps its pair, and both readings are published — the alarm because
/// nothing improved, the terminus because nothing was offered.
///
/// ´claim:bayes:a-refused-factorisation-raises-the-alarm-and-is-a-terminus´
/// ´test:crate:a-refused-factorisation-raises-the-alarm´
#[test]
fn a_refused_factorisation_raises_the_alarm() {
    use crate::model::bayesian::BayesianLinearModel;

    let mat = Mat::from_fn(2, 2, |i, j| if i == j { 1.0 } else { 2.0 });
    let precision = SymmetricMatrix::from_computation(mat);
    let covariance = SymmetricMatrix::identity_scaled(2, 1.0);
    let inputs = visit_inputs(synchronisation_error_threshold(2));

    let (outcome, rebuilt, baseline) = synchronisation_visit(&precision, &covariance, None, false, &inputs);

    assert!(outcome.is_alarm());
    assert!(outcome.is_cascade_terminus(), "a refusal is a terminus");
    assert!(outcome.refused_pivot().is_some(), "and it says where");
    assert!(rebuilt.is_none());
    assert_eq!(baseline.verdict().expect("the visit rebuilt"), RebuildVerdict::Refused);
    assert_eq!(
        baseline.sync_error_after().expect("the visit rebuilt"),
        baseline.sync_error_before,
        "a refusal offered nothing, so the reading that stands after it is the one before"
    );

    let mut model = BayesianLinearModel::new(2, 0.1, 1000);
    model.apply_recompute_outcome(&outcome, rebuilt, Some(baseline), 1000);
    assert_eq!(model.alarms(), 1);
    assert_eq!(model.cascade_terminus_events(), 1);
    assert_eq!(model.n_recompute_effective(), N_RECOMPUTE_FLOOR);
}

/// The interval recovers under a run of good measurements and returns to the
/// floor the moment one alarm fires.
///
/// Recovery spent out of measurements is what lets a model that needed one
/// rebuild climb back without paying for three more, and it is the half of the
/// cadence the restructure changes most: under the arrangement it replaces the
/// run was spent out of rebuilds, so a model could only earn its interval back
/// by paying the cost the interval exists to avoid
/// (´dec:posterior:adaptive-cadence´).
///
/// ´claim:bayes:the-interval-recovers-under-measurements-and-returns-to-the-floor-on-an-alarm´
/// ´test:crate:the-interval-recovers-under-measurements´
#[test]
fn the_interval_recovers_under_measurements() {
    use crate::model::bayesian::BayesianLinearModel;

    let mut model = BayesianLinearModel::new(5, 0.1, 1000);
    let alarm = RecomputeOutcome::Alarm {
        sync_error_residual: 1.0,
        sync_error_after: 2.0,
        refused_pivot: None,
    };
    let measured = RecomputeOutcome::Measured {
        sync_error_residual: 1e-14,
        at_resolution: false,
    };

    model.apply_recompute_outcome(&alarm, None, None, 1000);
    assert_eq!(model.n_recompute_effective(), N_RECOMPUTE_FLOOR);

    // The run has to be earned: two measurements are not enough.
    model.apply_recompute_outcome(&measured, None, None, 1000);
    model.apply_recompute_outcome(&measured, None, None, 1000);
    assert_eq!(
        model.n_recompute_effective(),
        N_RECOMPUTE_FLOOR,
        "two of the three do not restore it"
    );

    model.apply_recompute_outcome(&measured, None, None, 1000);
    assert_eq!(
        model.n_recompute_effective(),
        1000,
        "the third completes the run and the configured interval returns"
    );
    assert_eq!(model.measurements(), 3);
    assert_eq!(model.alarms(), 1);

    // And one alarm puts it back at the floor from the top.
    model.apply_recompute_outcome(&alarm, None, None, 1000);
    assert_eq!(model.n_recompute_effective(), N_RECOMPUTE_FLOOR);
}
