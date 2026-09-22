// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Guarded state-establishing fixtures.
//!
//! A fixture is state-establishing when it creates or mutates a [`World`] so its caller can rely on a named setup fact. Before returning, such a fixture measures that fact, checks it against its declared baseline, and includes the expected and measured baselines in any refusal (´dec:harness:guarded-fixtures´).
//!
//! The guard ends at the setup boundary. It establishes that the subject is ready for a test, while a result oracle separately decides whether code under test produced the promised result; combining those checks would misreport a result disagreement as broken setup (´dec:harness:separate-validation´).
//!
//! A fixture violates this convention when it returns state without reading back its named precondition, relies on an unmeasured hoped-for threshold, omits the measured baseline from its refusal, or uses one check for both setup validity and result correctness.
//!
//! The trained-state population and floors retain the public convergence witness's declarations, with their reasons registered beside this fixture's open backlog entry (´tab:assayer:harness-trained-state-fixture-figures´).

use super::label_spec::LabelSpec;
use super::signals::{score_verified_request_schema, with_score_verified};
use super::world::{World, WorldBuildError};
use crate::ChannelPolicy;
use crate::assessment::{DerivedReckoning, RequestContext};
use crate::config::types::AssayerConfig;
use crate::error::LabelError;

const TRAINING_CYCLES: usize = 500;
const HELD_OUT_SAMPLES_PER_CLASS: usize = 20;
const MIN_HELD_OUT_GAP: f64 = 0.10;
const MIN_PAIRWISE_RANK: f64 = 0.65;

/// The measured facts that make a [`World`] trained for score-verified callers.
#[derive(Clone, Copy, Debug)]
pub struct TrainedStateBaseline {
    /// Benign labels accepted during training.
    pub benign_training_class_count: usize,
    /// Adverse labels accepted during training.
    pub adverse_training_class_count: usize,
    /// Benign examples in the held-out rank population.
    pub benign_held_out_class_count: usize,
    /// Adverse examples in the held-out rank population.
    pub adverse_held_out_class_count: usize,
    /// Adverse minus benign risk on the two endpoint requests after training.
    pub held_out_gap: f64,
    /// Tie-corrected pairwise rank probability over the held-out populations.
    pub pairwise_rank: f64,
}

impl TrainedStateBaseline {
    const EXPECTED: Self = Self {
        benign_training_class_count: TRAINING_CYCLES / 2,
        adverse_training_class_count: TRAINING_CYCLES / 2,
        benign_held_out_class_count: HELD_OUT_SAMPLES_PER_CLASS,
        adverse_held_out_class_count: HELD_OUT_SAMPLES_PER_CLASS,
        held_out_gap: MIN_HELD_OUT_GAP,
        pairwise_rank: MIN_PAIRWISE_RANK,
    };

    /// Returns the exact class counts and strict separation floors declared by this fixture.
    #[must_use]
    pub const fn expected() -> Self {
        Self::EXPECTED
    }

    fn satisfies(self, expected: Self) -> bool {
        self.benign_training_class_count == expected.benign_training_class_count
            && self.adverse_training_class_count == expected.adverse_training_class_count
            && self.benign_held_out_class_count == expected.benign_held_out_class_count
            && self.adverse_held_out_class_count == expected.adverse_held_out_class_count
            && self.held_out_gap > expected.held_out_gap
            && self.pairwise_rank > expected.pairwise_rank
    }
}

/// A real-engine world and the measured baseline that admitted it as trained.
pub struct TrainedStateFixture {
    /// The world whose trained preconditions were measured.
    pub world: World,
    /// The baseline measured immediately before the fixture returned.
    pub baseline: TrainedStateBaseline,
}

impl TrainedStateFixture {
    /// Applies the trained-state precondition guard to one measured baseline.
    ///
    /// # Errors
    ///
    /// Returns [`TrainedStateFixtureError::BaselineMismatch`] with the expected and measured baselines when any class count differs or either strict separation floor is not exceeded.
    pub fn guard_baseline(measured: TrainedStateBaseline) -> Result<TrainedStateBaseline, TrainedStateFixtureError> {
        let expected = TrainedStateBaseline::EXPECTED;
        if !measured.satisfies(expected) {
            return Err(TrainedStateFixtureError::BaselineMismatch { expected, measured });
        }
        Ok(measured)
    }

    pub(super) fn establish(instance_id: &str, seed: u64) -> Result<Self, TrainedStateFixtureError> {
        let world = World::builder(AssayerConfig {
            instance_id: instance_id.to_owned(),
            infrastructure: super::test_infrastructure(),
            ..Default::default()
        })
        .signal_schema(score_verified_request_schema())
        .channel("default", ChannelPolicy::default())
        .seed(seed)
        .build()?;

        let cold_requests = [
            scored_request(&world, "cold-benign", 0.08, false),
            scored_request(&world, "cold-adverse", 0.92, true),
        ];
        world.settle_cold_ramp_with(&cold_requests);

        let mut benign_training_class_count = 0;
        let mut adverse_training_class_count = 0;
        for cycle in 0..TRAINING_CYCLES {
            let adverse = cycle % 2 == 1;
            let request = if adverse {
                scored_request(&world, "training-adverse", 0.92, true)
            } else {
                scored_request(&world, "training-benign", 0.08, false)
            };
            let reckoning = derived(&world, request);
            let label = if adverse {
                adverse_training_class_count += 1;
                LabelSpec::adverse(reckoning.assessment.id)
            } else {
                benign_training_class_count += 1;
                LabelSpec::benign(reckoning.assessment.id)
            };
            world.label(label.build())?;
            world.flush_labels()?;
        }

        let benign_endpoint = derived(&world, scored_request(&world, "held-out-benign", 0.08, false));
        let adverse_endpoint = derived(&world, scored_request(&world, "held-out-adverse", 0.92, true));
        let held_out_gap = adverse_endpoint.assessment.risk.p_bad - benign_endpoint.assessment.risk.p_bad;

        let benign_risks: Vec<_> = (0_u32..u32::try_from(HELD_OUT_SAMPLES_PER_CLASS).unwrap_or_default())
            .map(|sample| {
                let score = f64::from(sample).mul_add(0.012, 0.02);
                derived(&world, scored_request(&world, "rank-benign", score, false))
                    .assessment
                    .risk
                    .p_bad
            })
            .collect();
        let adverse_risks: Vec<_> = (0_u32..u32::try_from(HELD_OUT_SAMPLES_PER_CLASS).unwrap_or_default())
            .map(|sample| {
                let score = f64::from(sample).mul_add(0.012, 0.75);
                derived(&world, scored_request(&world, "rank-adverse", score, true))
                    .assessment
                    .risk
                    .p_bad
            })
            .collect();

        let measured = Self::guard_baseline(TrainedStateBaseline {
            benign_training_class_count,
            adverse_training_class_count,
            benign_held_out_class_count: benign_risks.len(),
            adverse_held_out_class_count: adverse_risks.len(),
            held_out_gap,
            pairwise_rank: pairwise_rank(&adverse_risks, &benign_risks),
        })?;

        Ok(Self {
            world,
            baseline: measured,
        })
    }
}

/// Failures produced while establishing or guarding a trained state.
#[derive(Debug)]
pub enum TrainedStateFixtureError {
    /// The real-engine world could not be built.
    WorldBuild(WorldBuildError),
    /// A training label or its publication barrier was rejected.
    Label(LabelError),
    /// One or more measured trained-state preconditions failed.
    BaselineMismatch {
        /// Exact class counts and strict separation floors declared by the fixture.
        expected: TrainedStateBaseline,
        /// Baseline measured after the fixture's setup work.
        measured: TrainedStateBaseline,
    },
}

impl From<WorldBuildError> for TrainedStateFixtureError {
    fn from(error: WorldBuildError) -> Self {
        Self::WorldBuild(error)
    }
}

impl From<LabelError> for TrainedStateFixtureError {
    fn from(error: LabelError) -> Self {
        Self::Label(error)
    }
}

impl std::fmt::Display for TrainedStateFixtureError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WorldBuild(error) => write!(formatter, "trained-state world construction failed: {error}"),
            Self::Label(error) => write!(formatter, "trained-state label setup failed: {error}"),
            Self::BaselineMismatch { expected, measured } => write!(
                formatter,
                "trained-state measured baseline {measured:?} does not satisfy expected counts and strict floors {expected:?}"
            ),
        }
    }
}

impl std::error::Error for TrainedStateFixtureError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::WorldBuild(error) => Some(error),
            Self::Label(error) => Some(error),
            Self::BaselineMismatch { .. } => None,
        }
    }
}

fn scored_request(world: &World, entity: &str, score: f64, verified: bool) -> RequestContext {
    with_score_verified(world.request("default", entity), score, verified)
}

fn derived(world: &World, request: RequestContext) -> DerivedReckoning {
    match world.derive_for_request(request) {
        Ok(reckoning) => reckoning,
        Err(error) => match error {},
    }
}

fn pairwise_rank(adverse_risks: &[f64], benign_risks: &[f64]) -> f64 {
    let mut wins = 0.0_f64;
    let mut comparisons = 0.0_f64;
    for adverse_risk in adverse_risks {
        for benign_risk in benign_risks {
            if adverse_risk > benign_risk {
                wins += 1.0;
            } else if adverse_risk.to_bits() == benign_risk.to_bits() {
                wins += 0.5;
            }
            comparisons += 1.0;
        }
    }
    wins / comparisons
}
