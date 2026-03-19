// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

// ── Config Presets ──────────────────────────────────────────────
//
//  Every preset is a `const fn` returning `Config<V>`.
//  Shared defaults live in `BASE_U64` / `BASE_F64` — individual
//  presets override only the fields that differ using struct update
//  syntax, making the intent of each config immediately visible.

use crate::graph::Config;

// ── Base configs ────────────────────────────────────────────────

/// Canonical baseline: θ=5, `D_create`=3, `D_evict`=6, no budget.
const BASE_U64: Config<u64> = Config {
    split_threshold: 5,
    depth_create: 3,
    depth_evict: 6,
    budget: None,
    alpha_relax: 0.75,
    bounded_eviction: true,
};

/// Canonical f64 baseline: θ=5.0, `D_create`=3, `D_evict`=8.
const BASE_F64: Config<f64> = Config {
    split_threshold: 5.0,
    depth_create: 3,
    depth_evict: 8,
    budget: None,
    alpha_relax: 0.75,
    bounded_eviction: true,
};

// ── u64 presets ─────────────────────────────────────────────────

/// Standard unbounded config: θ=5, `D_create`=3, `D_evict`=6.
#[must_use]
pub const fn default_config() -> Config<u64> {
    BASE_U64
}

/// Standard budgeted config: θ=5, `D_create`=3, `D_evict`=6.
#[must_use]
pub const fn budget_config(budget: usize) -> Config<u64> {
    Config {
        budget: Some(budget),
        ..BASE_U64
    }
}

/// Budgeted config with unbounded eviction mode.
#[must_use]
pub const fn budget_config_unbounded(budget: usize) -> Config<u64> {
    Config {
        budget: Some(budget),
        bounded_eviction: false,
        ..BASE_U64
    }
}

/// Tight budget: minimum allowed budget for buffer=3 (headroom=81,
/// budget=82, `soft_limit`=1).
#[must_use]
pub const fn tight_budget_config() -> Config<u64> {
    Config {
        budget: Some(82),
        bounded_eviction: false,
        ..BASE_U64
    }
}

/// Small buffer config: buffer=1 (`D_evict`=4), headroom=9.
#[must_use]
pub const fn small_buffer_config(budget: usize) -> Config<u64> {
    Config {
        depth_evict: 4,
        budget: Some(budget),
        bounded_eviction: false,
        ..BASE_U64
    }
}

/// Low threshold for frequent splits: θ=2, deeper depths.
#[must_use]
pub const fn low_threshold_config() -> Config<u64> {
    Config {
        split_threshold: 2,
        depth_create: 4,
        depth_evict: 8,
        ..BASE_U64
    }
}

/// Medium threshold for cascade testing: θ=3.
#[must_use]
pub const fn cascade_config() -> Config<u64> {
    Config {
        split_threshold: 3,
        depth_create: 4,
        depth_evict: 8,
        ..BASE_U64
    }
}

/// Deep config for escalation testing: θ=2, `D_create`=5, `D_evict`=10.
#[must_use]
pub const fn deep_config() -> Config<u64> {
    Config {
        split_threshold: 2,
        depth_create: 5,
        depth_evict: 10,
        ..BASE_U64
    }
}

/// Aggressive config: θ=1 forces immediate splitting, creating many
/// internal nodes and frequent violations.  Ideal for testing source
/// 10 topology (overlapping violation neighborhoods).
#[must_use]
pub const fn aggressive_config() -> Config<u64> {
    Config {
        split_threshold: 1,
        depth_create: 5,
        depth_evict: 10,
        ..BASE_U64
    }
}

/// Wide shallow config: θ=10, shallow `D_create`/`D_evict`.
/// Creates wide, shallow trees — tests lateral violation propagation
/// rather than deep escalation.
#[must_use]
pub const fn wide_shallow_config() -> Config<u64> {
    Config {
        split_threshold: 10,
        depth_create: 2,
        depth_evict: 4,
        ..BASE_U64
    }
}

/// Worked example config from §IDEA M-16: θ=5, `D_create`=3,
/// `D_evict`=6, N=3 (domain [0,8)).
#[must_use]
pub const fn worked_example_config() -> Config<u64> {
    BASE_U64
}

/// Config for `build_range_tree` equivalent: θ=1 forces immediate
/// splits.
#[must_use]
pub const fn range_tree_config() -> Config<u64> {
    Config {
        split_threshold: 1,
        ..BASE_U64
    }
}

/// Config for eviction integration tests: θ=5, `D_create`=3,
/// `D_evict`=4.
///
/// With `N=8` (domain `[0,256)`), spreading observations triggers
/// splits at low depths, producing terminals eligible for eviction
/// at the configured `D_evict`.
#[must_use]
pub const fn evictable_config() -> Config<u64> {
    Config {
        depth_evict: 4,
        ..BASE_U64
    }
}

// ── f64 presets ─────────────────────────────────────────────────

/// Standard f64 config: θ=5.0, `D_create`=3, `D_evict`=8.
#[must_use]
pub const fn f64_default_config() -> Config<f64> {
    BASE_F64
}

/// Deep f64 config for depth-saturation testing: θ=5.0,
/// `D_create`=8, `D_evict`=16.
#[must_use]
pub const fn f64_deep_config() -> Config<f64> {
    Config {
        depth_create: 8,
        depth_evict: 16,
        ..BASE_F64
    }
}
