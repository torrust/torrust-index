// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Sentinel configurations for tests.

use torrust_sentinel::{NoiseSchedule, SentinelConfig, SvdStrategy};

/// Test config with faster EWMA parameters (ADR-S-012).
///
/// Uses λ=0.90 (`forgetting_factor`) and `λ_s`=0.99 (`cusum_slow_decay`)
/// giving speed-separation ratio R = `W_s`/`W_f` = 100/10 = 10×,
/// matching the production ratio (λ=0.99, `λ_s`=0.999 → R=10×).
///
/// Convergence properties at λ=0.90:
///   η < 0.50 after  7 batches  (was 14 at λ=0.95)
///   η < 0.05 after 29 batches  (was 59 at λ=0.95)
///   half-life `h_f` = 6.6 rounds  (was 13.5)
///
/// The slow EWMA at `λ_s`=0.99 has `h_s`=69 rounds, settling
/// to 87.5% after ~207 rounds (3× `h_s`).
pub fn test_config() -> SentinelConfig<u64> {
    SentinelConfig::<u64> {
        max_rank: 4,
        forgetting_factor: 0.90,
        rank_update_interval: 10,
        analysis_k: 16,
        analysis_depth_cutoff: 6,
        energy_threshold: 0.90,
        eps: 1e-6,
        per_sample_scores: true,
        cusum_allowance_sigmas: 0.5,
        cusum_slow_decay: 0.99,
        cusum_coord_slow_decay: 0.99,
        clip_sigmas: 3.0,
        clip_pressure_decay: 0.95,
        split_threshold: 100,
        d_create: 3,
        d_evict: 6,
        budget: 100_000,
        noise_schedule: NoiseSchedule::Explicit(vec![5]),
        noise_batch_size: 4,
        noise_seed: Some(42),
        background_warming: false,
        svd_strategy: SvdStrategy::Brand,
    }
}

/// Config with noise disabled — trackers start cold.
pub fn cold_config() -> SentinelConfig<u64> {
    SentinelConfig::<u64> {
        noise_schedule: NoiseSchedule::Explicit(vec![]),
        ..test_config()
    }
}

/// Config tuned for integration tests: tight rank so novelty
/// signals are clearer, deterministic noise, fast rank adaptation.
///
/// Overrides from [`test_config()`]:
/// - `max_rank`: 4 → 2
/// - `rank_update_interval`: 10 → 5
/// - `per_sample_scores`: true → false
pub fn integration_config() -> SentinelConfig<u64> {
    SentinelConfig::<u64> {
        max_rank: 2,
        rank_update_interval: 5,
        per_sample_scores: false,
        ..test_config()
    }
}
