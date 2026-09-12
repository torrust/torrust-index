// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`first_observation_seeds_rather_than_smooths`] | wellness | The first sample of a stage becomes that stage's reading outright instead of being averaged against the zero an empty accumulator would otherwise hold. A smoothing rate near one takes hundreds of observations to climb out of a false zero, so a seeded start is the difference between a first reading that is the measurement and one that is an artefact of the accumulator's age. |
//! | [`later_observations_smooth_towards_the_samples_seen`] | wellness | Once seeded, a stage moves towards new samples without jumping to them: a run of larger samples lifts the reading above the seed and leaves it below the samples themselves. Feedback latency is dominated by a human queue whose individual waits vary enormously, so a reading that tracked the last sample would be noise and one that ignored it would never reflect a real change. |
//! | [`stages_are_independent_and_absent_until_observed`] | wellness | Each stage is absent until something is measured for it and observing one leaves the others untouched. The four stages are measured at four different boundaries and a label can reach some and not others — a replayed label has no arrival — so a tracker that seeded all four together would report figures for boundaries nothing had crossed. |
//! | [`the_total_is_the_sum_of_what_is_present`] | wellness | The total adds the stages that have readings and is absent only when none has. It is honest about being partial rather than refusing to answer: an operator whose report stage is unmeasured still needs the three stages that are, and a total that vanished whenever one stage was missing would hide the other three. |

//! The four stages of end-to-end feedback latency, smoothed as they are
//! observed (´def:monitoring:feedback-latency´).
//!
//! One label's journey crosses four boundaries, each measured by a different
//! clock reading at a different place in the process. This tracker is where
//! the four meet. It accumulates on the model owner's thread, one observation
//! per published label, so the stages it reports are all measured over the
//! same stream of labels and their sum is the sum of the same journey rather
//! than four averages taken over four different populations.
//!
//! # Cross-References
//!
//! - (´def:monitoring:feedback-latency´) — the four-stage decomposition this
//!   realises, and the residual the first stage does not include
//! - (´dec:health:independent-publication´) — the summary these readings ride
//! - (´inv:monitoring:report-only´) — the readings report and never act

#![allow(dead_code)]

/// Exponentially smoothed readings of the four feedback-latency stages.
///
/// Every field is `None` until that stage is observed, because a stage nothing
/// has measured and a stage measured at zero are different facts and a
/// deployment acts differently on each.
///
/// The first stage is a **lower bound** and never the stage itself. It is
/// built from the age a Sentinel measured on its own clock, which stops when
/// the Sentinel emitted the report; what happened between that emission and
/// the report reaching this process is an explicitly unmeasured residual, so
/// the true first stage is at least this and possibly more. Every surface that
/// carries the figure says so.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
// The shared suffix is the unit, and the unit is the point: the producing
// Sentinel measures in microseconds and this surface reports in seconds, so a
// field that did not name its unit would invite the conversion to be skipped.
#[allow(clippy::struct_field_names)]
pub struct FeedbackLatencyStages {
    /// A lower bound on the time from a Sentinel's observation to the report
    /// carrying it reaching this process, in seconds.
    pub report_lower_bound_seconds: Option<f64>,
    /// Time from that report's reception to the host's label arriving, in
    /// seconds. The human in the loop is normally the whole of it.
    pub label_seconds: Option<f64>,
    /// Time the label then waited for the model owner to take it up, in
    /// seconds.
    pub queue_seconds: Option<f64>,
    /// Time the model owner took to turn the label into a published snapshot,
    /// in seconds.
    pub publish_seconds: Option<f64>,
}

impl FeedbackLatencyStages {
    /// The sum of the stages that have readings, or `None` when none has.
    ///
    /// A lower bound whenever the first stage contributes, and partial
    /// whenever a stage is absent. Both qualifications are inherited from the
    /// parts rather than hidden by the sum: an operator reads the total beside
    /// the stages that produced it.
    #[must_use]
    pub fn total_seconds(&self) -> Option<f64> {
        let parts = [
            self.report_lower_bound_seconds,
            self.label_seconds,
            self.queue_seconds,
            self.publish_seconds,
        ];
        parts
            .iter()
            .filter_map(|part| *part)
            .fold(None, |acc: Option<f64>, part| Some(acc.unwrap_or(0.0) + part))
    }
}

/// Accumulator for the four stages, updated once per published label.
///
/// Held beside the model owner's other health counters and read at
/// publication. It deliberately does not cross a restart: every stage is a
/// difference between two `Instant`s in this process's monotonic domain, and a
/// figure carried over from a domain that no longer exists would describe a
/// journey no live label took.
#[derive(Clone, Copy, Debug, Default)]
pub struct FeedbackLatencyTracker {
    /// The smoothed stages as they stand.
    stages: FeedbackLatencyStages,
}

impl FeedbackLatencyTracker {
    /// A tracker that has observed nothing.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            stages: FeedbackLatencyStages {
                report_lower_bound_seconds: None,
                label_seconds: None,
                queue_seconds: None,
                publish_seconds: None,
            },
        }
    }

    /// The stages as they stand.
    #[must_use]
    pub const fn stages(&self) -> FeedbackLatencyStages {
        self.stages
    }

    /// Folds one label's journey into the four smoothed stages.
    ///
    /// `rate` is the smoothing rate: the weight kept on the existing reading,
    /// as (´tab:config:monitoring´) declares it. Each sample is applied only
    /// to its own stage, so a label that reached some boundaries and not
    /// others contributes exactly what it measured.
    pub fn observe(&mut self, rate: f64, sample: FeedbackLatencyStages) {
        Self::fold(
            &mut self.stages.report_lower_bound_seconds,
            rate,
            sample.report_lower_bound_seconds,
        );
        Self::fold(&mut self.stages.label_seconds, rate, sample.label_seconds);
        Self::fold(&mut self.stages.queue_seconds, rate, sample.queue_seconds);
        Self::fold(&mut self.stages.publish_seconds, rate, sample.publish_seconds);
    }

    /// Seeds an unobserved stage and smooths an observed one.
    ///
    /// Seeding matters at the rates this is configured for: near one, an
    /// accumulator started at zero spends hundreds of observations climbing
    /// out of a figure nothing measured.
    fn fold(current: &mut Option<f64>, rate: f64, sample: Option<f64>) {
        let Some(sample) = sample else { return };
        *current = Some(current.map_or(sample, |held| (1.0 - rate).mul_add(sample, rate * held)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RATE: f64 = 0.9;

    fn report_only(seconds: f64) -> FeedbackLatencyStages {
        FeedbackLatencyStages {
            report_lower_bound_seconds: Some(seconds),
            ..FeedbackLatencyStages::default()
        }
    }

    /// The first sample of a stage becomes that stage's reading outright
    /// instead of being averaged against the zero an empty accumulator would
    /// otherwise hold. A smoothing rate near one takes hundreds of
    /// observations to climb out of a false zero, so a seeded start is the
    /// difference between a first reading that is the measurement and one that
    /// is an artefact of the accumulator's age.
    ///
    /// ´claim:wellness:the-first-sample-of-a-latency-stage-is-the-reading-rather-than-a-step-away-from-zero´
    /// ´test:unit:first-observation-seeds-rather-than-smooths´
    #[test]
    fn first_observation_seeds_rather_than_smooths() {
        let mut tracker = FeedbackLatencyTracker::new();
        tracker.observe(RATE, report_only(4.0));

        assert_eq!(tracker.stages().report_lower_bound_seconds, Some(4.0));
    }

    /// Once seeded, a stage moves towards new samples without jumping to them:
    /// a run of larger samples lifts the reading above the seed and leaves it
    /// below the samples themselves. Feedback latency is dominated by a human
    /// queue whose individual waits vary enormously, so a reading that tracked
    /// the last sample would be noise and one that ignored it would never
    /// reflect a real change.
    ///
    /// ´claim:wellness:a-seeded-latency-stage-moves-towards-later-samples-without-reaching-them´
    /// ´test:unit:later-observations-smooth-towards-the-samples-seen´
    #[test]
    fn later_observations_smooth_towards_the_samples_seen() {
        let mut tracker = FeedbackLatencyTracker::new();
        tracker.observe(RATE, report_only(1.0));
        for _ in 0..5 {
            tracker.observe(RATE, report_only(11.0));
        }

        let reading = tracker.stages().report_lower_bound_seconds.expect("stage observed");
        assert!(reading > 1.0, "the reading rose above the seed: {reading}");
        assert!(reading < 11.0, "the reading stayed below the samples: {reading}");
    }

    /// Each stage is absent until something is measured for it and observing
    /// one leaves the others untouched. The four stages are measured at four
    /// different boundaries and a label can reach some and not others — a
    /// replayed label has no arrival — so a tracker that seeded all four
    /// together would report figures for boundaries nothing had crossed.
    ///
    /// ´claim:wellness:each-latency-stage-is-absent-until-its-own-boundary-is-measured´
    /// ´test:unit:stages-are-independent-and-absent-until-observed´
    #[test]
    fn stages_are_independent_and_absent_until_observed() {
        let tracker = FeedbackLatencyTracker::new();
        assert_eq!(tracker.stages(), FeedbackLatencyStages::default());

        let mut tracker = tracker;
        tracker.observe(
            RATE,
            FeedbackLatencyStages {
                queue_seconds: Some(0.25),
                ..FeedbackLatencyStages::default()
            },
        );

        let stages = tracker.stages();
        assert_eq!(stages.queue_seconds, Some(0.25));
        assert_eq!(stages.report_lower_bound_seconds, None);
        assert_eq!(stages.label_seconds, None);
        assert_eq!(stages.publish_seconds, None);
    }

    /// The total adds the stages that have readings and is absent only when
    /// none has. It is honest about being partial rather than refusing to
    /// answer: an operator whose report stage is unmeasured still needs the
    /// three stages that are, and a total that vanished whenever one stage was
    /// missing would hide the other three.
    ///
    /// ´claim:wellness:the-latency-total-adds-the-stages-that-have-readings-and-is-absent-only-when-none-has´
    /// ´test:unit:the-total-is-the-sum-of-what-is-present´
    #[test]
    fn the_total_is_the_sum_of_what_is_present() {
        assert_eq!(FeedbackLatencyStages::default().total_seconds(), None);

        let partial = FeedbackLatencyStages {
            label_seconds: Some(2.0),
            queue_seconds: Some(0.5),
            ..FeedbackLatencyStages::default()
        };
        assert_eq!(partial.total_seconds(), Some(2.5));

        let whole = FeedbackLatencyStages {
            report_lower_bound_seconds: Some(1.0),
            label_seconds: Some(2.0),
            queue_seconds: Some(0.5),
            publish_seconds: Some(0.25),
        };
        assert_eq!(whole.total_seconds(), Some(3.75));
    }
}
