// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Common test helpers for Assayer tests.
//!
//! This module provides convenient helper functions for constructing
//! test configurations, avoiding boilerplate in test code.
//!
//! For richer integration-test scaffolding — a named [`World`](crate::testing::World),
//! deterministic clock, seeded RNG, and golden reports — see the
//! [`crate::testing`] harness. The helpers here complement that harness
//! by providing terse config constructors for crate-level tests.

use std::path::Path;

use torrust_sentinel::BatchReport;

use crate::config::types::{AssayerConfig, InfrastructureConfig, ModelConfig, PersistenceConfig};

/// Creates a minimal test `AssayerConfig` with sensible defaults.
///
/// Uses small dimensions and default infrastructure settings.
pub fn test_assayer_config() -> AssayerConfig {
    AssayerConfig {
        instance_id: "test".to_owned(),
        // The capacity is host-set with no default; the fixture declares
        // the harness value.
        infrastructure: crate::testing::test_infrastructure(),
        ..Default::default()
    }
}

/// Creates a test `AssayerConfig` with a custom label channel capacity.
pub fn test_config_with_capacity(label_channel_capacity: usize) -> AssayerConfig {
    AssayerConfig {
        instance_id: "test".to_owned(),
        infrastructure: InfrastructureConfig {
            label_channel_capacity,
            ..crate::testing::test_infrastructure()
        },
        ..Default::default()
    }
}

/// Creates a test `AssayerConfig` with a custom instance ID.
pub fn test_config_with_id(instance_id: &str) -> AssayerConfig {
    AssayerConfig {
        instance_id: instance_id.to_owned(),
        infrastructure: crate::testing::test_infrastructure(),
        ..Default::default()
    }
}

/// Creates a test `ModelConfig` with sensible defaults for testing.
#[allow(dead_code)]
pub fn test_model_config() -> ModelConfig {
    ModelConfig::default()
}

/// Creates a test `ModelConfig` — anchor dimension is always P_A (15).
///
/// This function is retained for call-site compatibility but the
/// `anchor_dim` parameter is ignored: the anchor operates on a fixed
/// fifteen-dimensional vector and is never extended
/// (´def:risk:anchor-model´).
#[allow(dead_code)]
pub fn test_model_config_with_anchor(_anchor_dim: usize) -> ModelConfig {
    ModelConfig::default()
}

/// Creates a test `PersistenceConfig` with the given paths.
#[allow(dead_code)]
pub fn test_persistence_config(checkpoint_path: &Path, journal_path: &Path) -> PersistenceConfig {
    PersistenceConfig {
        checkpoint_dir: checkpoint_path.parent().unwrap_or(checkpoint_path).to_path_buf(),
        journal_dir: journal_path.parent().unwrap_or(journal_path).to_path_buf(),
        checkpoint_interval: std::time::Duration::from_secs(3600),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Test Infrastructure Helpers
// ═══════════════════════════════════════════════════════════════════════════════

use std::convert::Infallible;

use crate::types::EntityKey;

/// Creates a deterministic `EntityKey` from a seed.
///
/// Useful for tests that need reproducible entity identifiers.
/// Thin wrapper around [`EntityKey::new`] — callers working with
/// the [`crate::testing::World`] harness should prefer
/// [`World::entity`](crate::testing::World::entity), which derives
/// the key from a stable string name.
///
/// # Examples
///
/// ```ignore
/// let entity1 = test_entity(1);
/// let entity2 = test_entity(2);
/// assert_ne!(entity1, entity2);
/// ```
pub fn test_entity(seed: u64) -> EntityKey {
    EntityKey::new(seed.to_le_bytes().to_vec())
}

/// Crate-test compatibility helper for tests that still need the old
/// assessment-plus-derivation shape while the production API exposes
/// `Assayer::assess()` only.
pub trait AssayerAssessDeriveCompat {
    fn derive_from_assess(&self, requests: &[crate::RequestContext]) -> Vec<Result<crate::DerivedReckoning, Infallible>>;
}

impl AssayerAssessDeriveCompat for crate::Assayer {
    fn derive_from_assess(&self, requests: &[crate::RequestContext]) -> Vec<Result<crate::DerivedReckoning, Infallible>> {
        let assessments = self.assess(requests);
        let policy = crate::ChannelPolicy::default();
        assessments
            .into_iter()
            .zip(requests)
            .map(|(assessment, request)| {
                let landscape = crate::derive_landscape(&assessment, &policy, crate::ChallengeEstimate::default());
                let profile = crate::render_resonances(&landscape, &assessment.risk, &crate::ResonanceConfig::default());
                Ok(crate::DerivedReckoning {
                    channel: request.channel_hint().unwrap_or(crate::ChannelId(0)),
                    assessment,
                    landscape,
                    profile,
                })
            })
            .collect()
    }
}

/// Creates a valid minimal `BatchReport<u128>` with configurable cells.
///
/// Each tuple in `cells` is `(lo, depth)` representing a cell's start
/// coordinate and depth. The root cell (depth 0) is added automatically.
/// Cells at depth < 8 go to `ancestor_reports`; depth >= 8 go to
/// `cell_reports` as competitive.
///
/// The root cell is shaped identically to the root in
/// [`crate::testing::golden_report`]; added cells use milder anomaly
/// values so individual tests can assemble predictable reports
/// without driving scores to their golden extremes.
///
/// # Examples
///
/// ```ignore
/// // Root-only report
/// let report = test_report(&[]);
///
/// // Root + one competitive cell at depth 8
/// let report = test_report(&[(0x1200_0000_0000_0000_0000_0000_0000_0000, 8)]);
/// ```
pub fn test_report(cells: &[(u128, u8)]) -> BatchReport<u128> {
    use crate::testing::scores::{ROOT_CUSUM, ROOT_MAX_Z};
    use crate::testing::{default_analysis_set_summary, default_contour, default_health, make_cell_report};

    // Milder anomaly profile for non-root cells added by this helper.
    // Deliberately lower than `testing::scores::D8_*` so tests can
    // inject cells without triggering the golden-report thresholds.
    const NONROOT_MAX_Z: [f64; crate::types::SCORING_AXIS_COUNT] = [1.0, 0.5, 0.8, 0.3];
    const NONROOT_CUSUM: [f64; crate::types::SCORING_AXIS_COUNT] = [0.1, 0.05, 0.1, 0.05];

    // Root at depth 0 — matches `testing::golden_report()`'s root.
    let root = make_cell_report(0, 0, 1000, 4, 8, 0.85, 0.05, ROOT_MAX_Z, ROOT_CUSUM, false);

    let mut cell_reports = vec![root];
    let mut ancestor_reports = vec![];

    for &(lo, depth) in cells {
        if depth == 0 {
            continue; // Skip root duplicates
        }
        let cell = make_cell_report(
            lo,
            u32::from(depth),
            100,
            4,
            8,
            0.8,
            0.1,
            NONROOT_MAX_Z,
            NONROOT_CUSUM,
            depth >= 8,
        );
        if depth >= 8 {
            cell_reports.push(cell);
        } else {
            ancestor_reports.push(cell);
        }
    }

    BatchReport {
        cell_reports,
        ancestor_reports,
        coordination_reports: vec![],
        contour: default_contour(),
        health: default_health(),
        analysis_set_summary: default_analysis_set_summary(),
        oldest_observation_age_micros: None,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Derivation-Layer Fixtures
// ═══════════════════════════════════════════════════════════════════════════════

/// A risk basis carrying exactly the sufficient statistic the derivation
/// reads (´schema:risk:basis´); every other field is inert there by
/// outcome-prediction neutrality (´def:landscape:outcome-neutrality´).
#[must_use]
pub fn risk_basis(p_bad: f64, sigma_eff: f64, kappa_eff: f64) -> crate::RiskBasis {
    crate::RiskBasis {
        p_bad,
        uncertainty: p_bad * (1.0 - p_bad) * sigma_eff / kappa_eff,
        // A basis built to a stated uncertainty has not been tempered, so it
        // reports nothing borrowed and the two figures stay consistent.
        borrowed_share: 0.0,
        anchor_weight: 0.0,
        rho_eff: crate::numerics::stable_logit(p_bad.clamp(1e-9, 1.0 - 1e-9)) * kappa_eff,
        sigma_eff,
        kappa_eff,
        p_bad_sister: p_bad,
        p_bad_operational: p_bad,
        intervention_effectiveness: 0.0,
        anchor_converged: false,
        n_sentinels_reporting: 0,
        sister_regime_calibration_records: 0.0,
        anchor_regime_calibration_records: 0.0,
    }
}

/// The four kernel scalars the retired input bundle carried, kept as a
/// terse fixture shape for the rendering tests.
#[derive(Clone, Debug)]
pub struct KernelScalars {
    /// Calibrated probability of adverse outcome.
    pub p_hat: f64,
    /// Logit-space effective standard deviation.
    pub sigma_eff: f64,
    /// Effective calibration condition number.
    pub kappa_eff: f64,
    /// Variance of the challenge-effectiveness estimate.
    pub sigma2_q_hat_c: f64,
}

/// Runs the layered pipeline — derive the landscape, render it — and
/// returns the rendered tags, the shape the retired single-call kernel
/// returned (´dec:landscape:rendering-optional´).
#[must_use]
pub fn derive_tags(
    scalars: &KernelScalars,
    q_hat_c: f64,
    policy: &crate::ChannelPolicy,
    config: &crate::ResonanceConfig,
) -> Vec<crate::TagResonance> {
    let basis = risk_basis(scalars.p_hat, scalars.sigma_eff, scalars.kappa_eff);
    let estimate = crate::ChallengeEstimate::new(q_hat_c, scalars.sigma2_q_hat_c).expect("test estimate pair in domain");
    let landscape = crate::resonance::landscape::derive_landscape_from_basis(&basis, policy, estimate);
    crate::render_resonances(&landscape, &basis, config).tags
}

/// Waits until the published snapshot version reaches `target`, returning the
/// version seen — which is below `target` only when the deadline expired.
///
/// The one copy for the crate suite: the owner tests and the persistence tests
/// both wait on the same published counter for the same reason, and two
/// helpers that had drifted apart would be two different answers to one
/// liveness question.
///
/// It sleeps between reads rather than yielding. A spin competes for the core
/// with the very thread whose progress it is waiting for, and on a small
/// instrumented runner that competition is what makes the wait fail: the
/// waiter is the reason the answer is late. A millisecond is far below any
/// deadline here and hands the core back.
pub fn wait_for_version(shared: &crate::snapshot::shared::SharedState, target: u64, timeout: std::time::Duration) -> u64 {
    let start = std::time::Instant::now();
    loop {
        let guard = shared.published.load();
        if guard.version >= target {
            return guard.version;
        }
        if start.elapsed() > timeout {
            return guard.version;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}
