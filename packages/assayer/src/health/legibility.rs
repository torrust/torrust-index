// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`a_block_with_no_evidence_reads_absent_on_both_readings`] | wellness | A block nothing has been folded into reports neither reading rather than reporting a number computed from an empty accumulator. The two readings are ratios whose denominators are accumulated quantities, and a block that has seen nothing has zero in both, so a producer that answered anyway would be publishing the shape of its own division. |
//! | [`a_coordinate_carrying_the_outcome_reads_far_above_the_null_floor`] | wellness | A coordinate that moves with the outcome earns an association many times the floor a coordinate telling the outcome nothing earns, and one drawn independently of the outcome stays beside that floor. Calibrating against the floor is what makes the two readable against each other at all: the raw correlation of a null coordinate is not zero and shrinks with the window, so only the multiple is comparable across blocks and across ages. |
//! | [`a_coordinate_that_never_moves_leaves_the_mean_rather_than_entering_it_as_zero`] | wellness | A coordinate that held one value throughout is left out of the block's mean instead of averaging in as a zero. A block of mostly-silent positions would otherwise read quieter than its live positions are, which is the reading saying the block is uninformative when what is true is that most of it is unoccupied. |
//! | [`the_effective_sample_size_settles_at_the_forgetting_rates_own_window`] | wellness | Under a constant stream the effective sample size climbs to the window the forgetting rate implies and stays there. The null floor is a function of that number alone, so a reading calibrated against it is calibrated against the stretch of traffic the accumulator actually remembers rather than against every label the deployment ever saw. |
//! | [`removing_a_block_the_score_leans_on_costs_loss_and_removing_an_idle_one_does_not`] | wellness | A block the score leans on shows a contribution well above zero, and a block whose weights are zero shows none. The reading is the removal cost itself rather than a proxy for it, so what it reports about retiring the block is what retiring the block would do. |
//! | [`a_block_whose_width_changes_starts_its_evidence_again`] | wellness | Evidence accumulated over a block of one width is discarded when the layout gives that block another. The accumulator's positions are the block's own coordinates, so carrying them across a re-widening would fold two different coordinates' histories into one position and publish the sum as a reading of neither. |

//! The two legibility readings and the evidence they are computed from
//! (´def:monitoring:slot-association´), (´def:monitoring:slot-contribution´).
//!
//! One accumulator per entity block — a Sentinel's own slot or an identity
//! dimension's own block — folded prequentially on the model owner's thread at
//! label time, before the label reaches the models. Everything the two
//! readings need is in hand at that moment: the standardised feature vector
//! the update is about to consume, the outcome the label carries, the
//! balancing weight the operational model will weigh it by, and the weights
//! the model held when it scored the request. Nothing here reads a frozen
//! window, and nothing here is recomputed from stored requests.
//!
//! The evidence is small and bounded by the block: three accumulator vectors
//! of the block's own width and five scalars. It is deliberately not the
//! stream — a producer that kept the labels themselves could compute anything
//! and would cost the deployment its label history to do it.
//!
//! # Cross-References
//!
//! - (´def:monitoring:slot-association´) — the screening reading, and the null
//!   floor it is expressed as a multiple of
//! - (´def:monitoring:slot-contribution´) — the confirmation reading, and why
//!   it does not compose across blocks
//! - (´def:weighting:balancing-weights´) — the weight each observation is
//!   folded under
//! - (´inv:monitoring:report-only´) — both readings report and never act

use crate::numerics::stable_sigmoid;

/// The variance below which a coordinate counts as having held one value.
///
/// A coordinate that never moved over the accumulator's window carries no
/// correlation with anything, and the sample correlation it would nominally
/// earn is a quotient of two vanishing quantities. The floor is placed far
/// below the variance of any standardised coordinate that varies at all and
/// far above the rounding of the moment arithmetic that produces it. Leaving
/// such a coordinate out of the block's mean rather than averaging a zero in
/// for it is the definition's own rule (´def:monitoring:slot-association´);
/// this is the boundary that decides which coordinates that rule applies to.
///
/// ´const:assayer:association-variance-floor´ (´alg:const:scalar´)
/// ´const:assayer:association-variance-floor-scalar-1en12´
const ASSOCIATION_VARIANCE_FLOOR: f64 = 1e-12;

/// The effective sample size below which neither reading is published.
///
/// The null floor is an asymptotic statement about the absolute value of a
/// sample correlation, and a handful of observations is not where it holds.
/// Thirty is the same evidence floor the calibration path applies to its own
/// first fit, and it is used here for the same reason: below it the quantity
/// exists and does not yet mean what its name says. Both readings are absent
/// rather than published beneath it (´def:monitoring:slot-association´),
/// (´def:monitoring:slot-contribution´).
///
/// ´const:assayer:legibility-evidence-floor´ (´alg:const:scalar´)
/// ´const:assayer:legibility-evidence-floor-scalar-30p0´
const LEGIBILITY_EVIDENCE_FLOOR: f64 = 30.0;

/// The mean absolute sample correlation a coordinate telling the outcome
/// nothing earns from a window of `n_eff` observations.
///
/// A null coordinate's sample correlation is approximately normal about zero
/// with standard deviation `1/√n`, so the expectation of its absolute value is
/// `√(2/πn)`. The figure depends on the window and on nothing else — not on
/// the block's width, not on the model, not on what the block is a block of —
/// which is what lets one number calibrate every block on the surface
/// (´def:monitoring:slot-association´).
#[must_use]
pub fn null_association_floor(effective_sample_size: f64) -> f64 {
    (2.0 / (std::f64::consts::PI * effective_sample_size)).sqrt()
}

/// Binary cross-entropy of the logistic read of one score against one outcome.
///
/// The operational model regresses a sign onto the standardised vector, so its
/// output is not a probability and the logistic is the monotone read that
/// turns it into one. The link is shared by the full score and the ablated
/// one, so it moves both by the same monotone map and cannot manufacture a
/// difference between them (´def:monitoring:slot-contribution´).
fn log_loss(score: f64, outcome: f64) -> f64 {
    let probability = stable_sigmoid(score).clamp(1e-12, 1.0 - 1e-12);
    let target = f64::midpoint(outcome, 1.0);
    -target.mul_add(probability.ln(), (1.0 - target) * (1.0 - probability).ln())
}

/// What one entity block's two legibility readings come to.
///
/// Each is absent rather than zero where the block has not carried enough
/// evidence to answer, because a block nothing has been measured on and a
/// block measured at nothing are different facts and a deployment acts
/// differently on each.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LegibilityReadings {
    /// The block's association with the outcome, as a multiple of the null
    /// floor (´def:monitoring:slot-association´).
    pub association: Option<f64>,
    /// What removing the block from the score would cost, as a fraction of
    /// the score's own loss (´def:monitoring:slot-contribution´).
    pub contribution: Option<f64>,
}

/// The stored evidence behind one entity block's two legibility readings.
///
/// Three vectors of the block's own width and five scalars, decayed at the
/// operational model's own per-label forgetting rate so that the readings
/// describe the same stretch of traffic the operational weights describe. The
/// decay is what makes the effective sample size a window rather than a count
/// of every label the deployment ever saw, and the window is what the null
/// floor is a function of.
///
/// The block's own weight accumulation serves both readings. Both fold at one
/// site under one guard, so a second copy of it kept for the contribution
/// reading alone could only ever differ from this one by a defect.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
// Justified: the shared suffix is the type's whole content. Every field is a
// decayed sum over the same stream, and a name that dropped the word would
// read as the quantity itself rather than as an accumulation of it — which is
// exactly the confusion the readings below exist to undo.
#[allow(clippy::struct_field_names)]
pub struct BlockLegibilityEvidence {
    /// Decayed sum of the observation weights, `Σ v`.
    weight_sum: f64,
    /// Decayed sum of their squares, `Σ v²`, which with the sum above gives
    /// the effective sample size.
    weight_square_sum: f64,
    /// Decayed weighted sum of the outcomes, `Σ v y`.
    outcome_sum: f64,
    /// Decayed weighted first moment of each of the block's coordinates.
    feature_sum: Vec<f64>,
    /// Decayed weighted second moment of each of the block's coordinates.
    feature_square_sum: Vec<f64>,
    /// Decayed weighted cross moment of each coordinate with the outcome.
    feature_outcome_sum: Vec<f64>,
    /// Decayed weighted excess loss of scoring without the block.
    excess_loss_sum: f64,
    /// Decayed weighted loss of scoring with it.
    full_loss_sum: f64,
}

impl BlockLegibilityEvidence {
    /// Folds one label into the block's evidence.
    ///
    /// `block_phi` is the block's own slice of the standardised feature
    /// vector, `block_score` the block's own contribution to the score the
    /// model gave the request, `full_score` the whole score, `outcome` the
    /// label's sign and `weight` the balancing weight the operational update
    /// will carry. `forgetting` is applied to everything already accumulated
    /// before the new observation lands, which is what makes each accumulator
    /// a decayed sum rather than a lifetime one.
    pub fn observe(&mut self, block_phi: &[f64], block_score: f64, full_score: f64, outcome: f64, weight: f64, forgetting: f64) {
        if self.feature_sum.len() != block_phi.len() {
            // The layout has given the block another width, so the positions
            // the vectors describe are not the positions arriving. Carrying
            // them across would fold two coordinates' histories into one
            // accumulator and publish the sum as a reading of neither.
            self.restart(block_phi.len());
        }

        self.weight_sum = forgetting.mul_add(self.weight_sum, weight);
        self.weight_square_sum = (forgetting * forgetting).mul_add(self.weight_square_sum, weight * weight);
        self.outcome_sum = forgetting.mul_add(self.outcome_sum, weight * outcome);

        let moments = self
            .feature_sum
            .iter_mut()
            .zip(self.feature_square_sum.iter_mut())
            .zip(self.feature_outcome_sum.iter_mut());
        for (((sum, square), cross), &value) in moments.zip(block_phi) {
            *sum = forgetting.mul_add(*sum, weight * value);
            *square = forgetting.mul_add(*square, weight * value * value);
            *cross = forgetting.mul_add(*cross, weight * value * outcome);
        }

        let full_loss = log_loss(full_score, outcome);
        let ablated_loss = log_loss(full_score - block_score, outcome);
        self.full_loss_sum = forgetting.mul_add(self.full_loss_sum, weight * full_loss);
        self.excess_loss_sum = forgetting.mul_add(self.excess_loss_sum, weight * (ablated_loss - full_loss));
    }

    /// Discards everything accumulated and sizes the vectors to a new width.
    fn restart(&mut self, width: usize) {
        self.weight_sum = 0.0;
        self.weight_square_sum = 0.0;
        self.outcome_sum = 0.0;
        self.feature_sum = vec![0.0; width];
        self.feature_square_sum = vec![0.0; width];
        self.feature_outcome_sum = vec![0.0; width];
        self.excess_loss_sum = 0.0;
        self.full_loss_sum = 0.0;
    }

    /// The window the decayed accumulators remember, `(Σ v)² / Σ v²`.
    ///
    /// Zero for an accumulator nothing has been folded into. Under a constant
    /// stream at forgetting rate `γ` it settles at `(1 + γ) / (1 − γ)`, which
    /// is the sense in which the rate names a window.
    #[must_use]
    pub fn effective_sample_size(&self) -> f64 {
        if self.weight_square_sum <= 0.0 {
            return 0.0;
        }
        self.weight_sum * self.weight_sum / self.weight_square_sum
    }

    /// Both readings, each absent where the evidence does not support it.
    #[must_use]
    pub fn readings(&self) -> LegibilityReadings {
        LegibilityReadings {
            association: self.association(),
            contribution: self.contribution(),
        }
    }

    /// The block's mean absolute correlation with the outcome, expressed as a
    /// multiple of the null floor (´def:monitoring:slot-association´).
    ///
    /// Absent where the window is too short for the floor to mean what it
    /// says, where every outcome folded in carried the same sign — an outcome
    /// that never varied has no correlation with anything — and where no
    /// coordinate of the block varied.
    #[must_use]
    pub fn association(&self) -> Option<f64> {
        let effective = self.effective_sample_size();
        if effective < LEGIBILITY_EVIDENCE_FLOOR || self.weight_sum <= 0.0 {
            return None;
        }
        let outcome_mean = self.outcome_sum / self.weight_sum;
        // The outcome is a sign, so its second moment is its weight and its
        // variance is one less the square of its mean.
        let outcome_variance = outcome_mean.mul_add(-outcome_mean, 1.0);
        if outcome_variance <= ASSOCIATION_VARIANCE_FLOOR {
            return None;
        }

        let mut total = 0.0;
        let mut varying = 0.0_f64;
        let moments = self
            .feature_sum
            .iter()
            .zip(self.feature_square_sum.iter())
            .zip(self.feature_outcome_sum.iter());
        for ((&sum, &square), &cross) in moments {
            let mean = sum / self.weight_sum;
            let variance = mean.mul_add(-mean, square / self.weight_sum);
            if variance <= ASSOCIATION_VARIANCE_FLOOR {
                continue;
            }
            let covariance = mean.mul_add(-outcome_mean, cross / self.weight_sum);
            total += (covariance / (variance * outcome_variance).sqrt()).abs();
            varying += 1.0;
        }
        if varying == 0.0 {
            return None;
        }
        Some(total / varying / null_association_floor(effective))
    }

    /// What removing the block from the score costs, as a fraction of the
    /// score's own loss (´def:monitoring:slot-contribution´).
    ///
    /// Absent where the window is too short and where the score carried no
    /// loss to take a fraction of. Negative where the block's coordinates were
    /// making the score worse, which is a reading and not a defect.
    #[must_use]
    pub fn contribution(&self) -> Option<f64> {
        if self.effective_sample_size() < LEGIBILITY_EVIDENCE_FLOOR || self.full_loss_sum <= 0.0 {
            return None;
        }
        Some(self.excess_loss_sum / self.full_loss_sum)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    /// The operational model's own per-label forgetting rate, which is the
    /// rate the producers fold at.
    const FORGETTING: f64 = crate::model::update::GAMMA_LABEL_OPERATIONAL;

    /// A deterministic bit from an avalanche over one index under one salt.
    ///
    /// Stands in for a coordinate stream that tells the outcome nothing: the
    /// index says nothing about the bit, two salts give two streams that say
    /// nothing about each other, and the same index gives the same bit on
    /// every run.
    fn mixed_bit(index: u64, salt: u64) -> bool {
        let mut word = index ^ salt;
        word ^= word >> 30;
        word = word.wrapping_mul(0xbf58_476d_1ce4_e5b9);
        word ^= word >> 27;
        word = word.wrapping_mul(0x94d0_49bb_1331_11eb);
        word ^= word >> 31;
        word >> 40 & 1 == 1
    }

    /// Drives a stream of labels whose outcomes alternate in a mixed order
    /// through one accumulator, with the first coordinate carrying the outcome
    /// and the second drawn from an independent stream.
    fn drive(labels: u64, block_score: impl Fn(f64) -> f64) -> BlockLegibilityEvidence {
        let mut evidence = BlockLegibilityEvidence::default();
        for index in 0..labels {
            let adverse = mixed_bit(index, 0xb7e1_5162_8aed_2a6b);
            let outcome = if adverse { 1.0 } else { -1.0 };
            let noise = if mixed_bit(index, 0x243f_6a88_85a3_08d3) { 1.0 } else { -1.0 };
            let phi = [outcome, noise, 1.0];
            evidence.observe(&phi, block_score(outcome), block_score(outcome), outcome, 1.0, FORGETTING);
        }
        evidence
    }

    /// A block nothing has been folded into reports neither reading rather
    /// than reporting a number computed from an empty accumulator. The two
    /// readings are ratios whose denominators are accumulated quantities, and
    /// a block that has seen nothing has zero in both, so a producer that
    /// answered anyway would be publishing the shape of its own division.
    ///
    /// ´claim:wellness:a-block-with-no-evidence-reports-neither-legibility-reading´
    /// ´test:unit:a-block-with-no-evidence-reads-absent-on-both-readings´
    #[test]
    fn a_block_with_no_evidence_reads_absent_on_both_readings() {
        let evidence = BlockLegibilityEvidence::default();

        assert_eq!(evidence.effective_sample_size().to_bits(), 0.0f64.to_bits());
        assert_eq!(evidence.readings(), LegibilityReadings::default());
    }

    /// A coordinate that moves with the outcome earns an association many
    /// times the floor a coordinate telling the outcome nothing earns, and one
    /// drawn independently of the outcome stays beside that floor. Calibrating
    /// against the floor is what makes the two readable against each other at
    /// all: the raw correlation of a null coordinate is not zero and shrinks
    /// with the window, so only the multiple is comparable across blocks and
    /// across ages.
    ///
    /// ´claim:wellness:the-association-reading-separates-a-carrying-coordinate-from-a-null-one´
    /// ´test:unit:a-coordinate-carrying-the-outcome-reads-far-above-the-null-floor´
    #[test]
    fn a_coordinate_carrying_the_outcome_reads_far_above_the_null_floor() {
        let mut carrying = BlockLegibilityEvidence::default();
        let mut null = BlockLegibilityEvidence::default();
        for index in 0..2_000_u64 {
            let adverse = mixed_bit(index, 0xb7e1_5162_8aed_2a6b);
            let outcome = if adverse { 1.0 } else { -1.0 };
            let noise = if mixed_bit(index, 0x243f_6a88_85a3_08d3) { 1.0 } else { -1.0 };
            carrying.observe(&[outcome], 0.0, 0.0, outcome, 1.0, FORGETTING);
            null.observe(&[noise], 0.0, 0.0, outcome, 1.0, FORGETTING);
        }

        let carried = carrying.association().expect("the carrying block reads");
        let unrelated = null.association().expect("the null block reads");
        assert!(carried > 15.0, "a carrying coordinate stands well above the floor: {carried}");
        assert!(unrelated < 3.0, "a null coordinate stays beside the floor: {unrelated}");
    }

    /// A coordinate that held one value throughout is left out of the block's
    /// mean instead of averaging in as a zero. A block of mostly-silent
    /// positions would otherwise read quieter than its live positions are,
    /// which is the reading saying the block is uninformative when what is
    /// true is that most of it is unoccupied.
    ///
    /// ´claim:wellness:a-coordinate-that-never-varied-leaves-the-association-mean´
    /// ´test:unit:a-coordinate-that-never-moves-leaves-the-mean-rather-than-entering-it-as-zero´
    #[test]
    fn a_coordinate_that_never_moves_leaves_the_mean_rather_than_entering_it_as_zero() {
        let mut alone = BlockLegibilityEvidence::default();
        let mut padded = BlockLegibilityEvidence::default();
        for index in 0..2_000_u64 {
            let adverse = mixed_bit(index, 0xb7e1_5162_8aed_2a6b);
            let outcome = if adverse { 1.0 } else { -1.0 };
            alone.observe(&[outcome], 0.0, 0.0, outcome, 1.0, FORGETTING);
            padded.observe(&[outcome, 1.0, 1.0, 1.0], 0.0, 0.0, outcome, 1.0, FORGETTING);
        }

        let one = alone.association().expect("the single-coordinate block reads");
        let four = padded.association().expect("the padded block reads");
        assert!(
            (one - four).abs() < 1e-9,
            "three constant coordinates left the mean untouched: {one} against {four}"
        );
    }

    /// Under a constant stream the effective sample size climbs to the window
    /// the forgetting rate implies and stays there. The null floor is a
    /// function of that number alone, so a reading calibrated against it is
    /// calibrated against the stretch of traffic the accumulator actually
    /// remembers rather than against every label the deployment ever saw.
    ///
    /// ´claim:wellness:the-legibility-window-settles-at-the-forgetting-rates-own-length´
    /// ´test:unit:the-effective-sample-size-settles-at-the-forgetting-rates-own-window´
    #[test]
    fn the_effective_sample_size_settles_at_the_forgetting_rates_own_window() {
        let settled = (1.0 + FORGETTING) / (1.0 - FORGETTING);

        let short = drive(1_000, |_| 0.0).effective_sample_size();
        let long = drive(60_000, |_| 0.0).effective_sample_size();
        let longer = drive(120_000, |_| 0.0).effective_sample_size();

        assert!(short < settled, "a short stream has not filled the window: {short}");
        assert!(
            (long - settled).abs() / settled < 0.01,
            "a long stream sits at the window: {long} against {settled}"
        );
        assert!(
            (longer - long).abs() / settled < 0.01,
            "the window stops growing: {longer} against {long}"
        );
    }

    /// A block the score leans on shows a contribution well above zero, and a
    /// block whose weights are zero shows none. The reading is the removal
    /// cost itself rather than a proxy for it, so what it reports about
    /// retiring the block is what retiring the block would do.
    ///
    /// ´claim:wellness:the-contribution-reading-is-the-removal-cost-itself´
    /// ´test:unit:removing-a-block-the-score-leans-on-costs-loss-and-removing-an-idle-one-does-not´
    #[test]
    fn removing_a_block_the_score_leans_on_costs_loss_and_removing_an_idle_one_does_not() {
        let carrying = drive(4_000, |outcome| 2.0 * outcome);
        let idle = drive(4_000, |_| 0.0);

        let cost = carrying.contribution().expect("the carrying block reads");
        let none = idle.contribution().expect("the idle block reads");
        assert!(cost > 0.58, "removing a block the score leans on costs loss: {cost}");
        assert!(none.abs() < 1e-12, "removing an idle block costs nothing: {none}");
    }

    /// Evidence accumulated over a block of one width is discarded when the
    /// layout gives that block another. The accumulator's positions are the
    /// block's own coordinates, so carrying them across a re-widening would
    /// fold two different coordinates' histories into one position and publish
    /// the sum as a reading of neither.
    ///
    /// ´claim:wellness:a-block-whose-width-changes-restarts-its-legibility-evidence´
    /// ´test:unit:a-block-whose-width-changes-starts-its-evidence-again´
    #[test]
    fn a_block_whose_width_changes_starts_its_evidence_again() {
        let mut evidence = BlockLegibilityEvidence::default();
        for index in 0..2_000_u64 {
            let outcome = if mixed_bit(index, 0xb7e1_5162_8aed_2a6b) { 1.0 } else { -1.0 };
            evidence.observe(&[outcome], 1.0, 1.0, outcome, 1.0, FORGETTING);
        }
        assert!(evidence.association().is_some(), "the narrow block had a reading");

        evidence.observe(&[1.0, 1.0], 1.0, 1.0, 1.0, 1.0, FORGETTING);

        assert_eq!(evidence.effective_sample_size().to_bits(), 1.0f64.to_bits());
        assert_eq!(evidence.readings(), LegibilityReadings::default());
    }
}
