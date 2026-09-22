// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Label-data builders for scenario tests.
//!
//! The production [`LabelData`] struct has many fields because it is
//! the public host-facing contract. Scenarios almost always only care
//! about *one* knob at a time — valence, action, or the ground-truth
//! flag. This module exposes a tiny fluent builder that fills the
//! rest with sensible defaults so a test can say
//!
//! ```ignore
//! world.label(LabelSpec::adverse(assessment_id).build()).unwrap();
//! ```
//!
//! without restating ten fields of boilerplate.

use std::collections::HashMap;

use crate::owner::commands::LabelData;
use crate::types::{Action, AssessmentId, ChallengeResult, OutcomeAxisId};

/// Fluent builder for [`LabelData`].
#[derive(Clone, Debug)]
pub struct LabelSpec {
    /// Underlying data, mutated by the builder methods.
    data: LabelData,
    /// Optional host-owned challenge outcome routed to the companion tracker.
    challenge_result: Option<ChallengeResult>,
}

impl LabelSpec {
    /// Start a new spec linked to `assessment_id`.
    ///
    /// Defaults: `Allow` action, valence 0.0 (benign), no outcomes,
    /// no ground-truth override.
    #[must_use]
    pub fn new(assessment_id: AssessmentId) -> Self {
        Self {
            data: LabelData {
                assessment_id,
                action_taken: Action::Allow,
                valence: 0.0,
                outcomes: HashMap::new(),
                ground_truth: false,
            },
            challenge_result: None,
        }
    }

    /// Shorthand for a clearly-adverse label: valence = 1.0.
    #[must_use]
    pub fn adverse(assessment_id: AssessmentId) -> Self {
        Self::new(assessment_id).valence(1.0)
    }

    /// Shorthand for a clearly-benign label: valence = -1.0.
    #[must_use]
    pub fn benign(assessment_id: AssessmentId) -> Self {
        Self::new(assessment_id).valence(-1.0)
    }

    /// Set the host action taken.
    #[must_use]
    pub const fn action(mut self, action: Action) -> Self {
        self.data.action_taken = action;
        self
    }

    /// Set the valence (positive = adverse).
    #[must_use]
    pub const fn valence(mut self, v: f64) -> Self {
        self.data.valence = v;
        self
    }

    /// Mark this label as ground-truth (bypasses action confounding).
    #[must_use]
    pub const fn ground_truth(mut self) -> Self {
        self.data.ground_truth = true;
        self
    }

    /// Attach a host-observed challenge result for companion tracking.
    ///
    /// This value is intentionally not part of [`LabelData`]. Test harnesses
    /// can read it via [`Self::challenge_outcome`] and route it to
    /// `ChallengeEffectivenessTracker` before submitting the core label.
    #[must_use]
    pub const fn challenge_result(mut self, result: ChallengeResult) -> Self {
        self.challenge_result = Some(result);
        self
    }

    /// Returns the companion challenge result attached to this spec, if any.
    #[must_use]
    pub const fn challenge_outcome(&self) -> Option<ChallengeResult> {
        self.challenge_result
    }

    /// Returns the action that will be submitted to the core label path.
    #[must_use]
    pub const fn action_taken(&self) -> Action {
        self.data.action_taken
    }

    /// Record an outcome-axis value.
    #[must_use]
    pub fn outcome(mut self, axis: OutcomeAxisId, value: f64) -> Self {
        self.data.outcomes.insert(axis, value);
        self
    }

    /// Consume the builder and produce the underlying [`LabelData`].
    #[must_use]
    pub fn build(self) -> LabelData {
        self.data
    }
}

impl From<LabelSpec> for LabelData {
    fn from(spec: LabelSpec) -> Self {
        spec.build()
    }
}
