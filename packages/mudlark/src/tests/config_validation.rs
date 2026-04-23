// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Boundary tests for **`Config::validate()`** (§ADR M-039, D7).
//!
//! Every `Config` field has at least one domain constraint checked at
//! construction time.  These tests exercise the exact boundary values
//! so that formula mutations (`<` → `<=`, `>` → `>=`, `>` → `==`,
//! etc.) are caught immediately.  Each pair of tests straddles the
//! boundary: one value that is just-valid and one that is
//! just-invalid.
//!
//! # Test index
//!
//! ## `depth_create` / `depth_evict` boundary (D-I3)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`depth_create_one_less_than_evict_is_valid`] | tightest valid gap (`d_create == d_evict - 1`) |
//! | [`depth_create_equal_to_evict_is_invalid`] | equal depths rejected |
//! | [`depth_create_greater_than_evict_is_invalid`] | inverted depths rejected |
//!
//! ## `depth_create` >= 1
//!
//! | Test | Focus |
//! |------|-------|
//! | [`depth_create_exactly_one_is_valid`] | minimum valid `depth_create` |
//! | [`depth_create_zero_is_invalid`] | zero rejected |
//!
//! ## `alpha_relax` boundaries
//!
//! | Test | Focus |
//! |------|-------|
//! | [`alpha_relax_just_above_zero_is_valid`] | smallest positive f64 accepted |
//! | [`alpha_relax_just_below_one_is_valid`] | largest sub-one f64 accepted |
//! | [`alpha_relax_zero_is_invalid`] | exact zero rejected |
//! | [`alpha_relax_one_is_invalid`] | exact one rejected |
//! | [`alpha_relax_negative_is_invalid`] | negative value rejected |
//! | [`alpha_relax_above_one_is_invalid`] | value > 1.0 rejected |
//! | [`alpha_relax_nan_is_invalid`] | NaN rejected |
//! | [`alpha_relax_positive_infinity_is_invalid`] | +∞ rejected |
//! | [`alpha_relax_negative_infinity_is_invalid`] | −∞ rejected |
//!
//! ## Budget headroom
//!
//! | Test | Focus |
//! |------|-------|
//! | [`budget_at_minimum_valid_value`] | `budget == required + 1` accepted |
//! | [`budget_exactly_at_required_is_invalid`] | `budget == required` rejected |
//! | [`budget_below_required_is_invalid`] | `budget << required` rejected |
//!
//! ## Budget with minimal buffer (buffer = 1)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`budget_minimal_buffer_at_boundary`] | just-valid with smallest buffer |
//! | [`budget_minimal_buffer_exactly_at_required`] | boundary with smallest buffer rejected |
//!
//! ## Budget where convergence dominates
//!
//! | Test | Focus |
//! |------|-------|
//! | [`budget_convergence_dominates_at_boundary`] | convergence term larger than headroom term |
//! | [`budget_convergence_dominates_exactly_at_required`] | exact convergence boundary rejected |
//!
//! ## No budget
//!
//! | Test | Focus |
//! |------|-------|
//! | [`no_budget_skips_headroom_check`] | `budget: None` bypasses headroom check |

use crate::graph::{Config, GvGraph};
use crate::testing::default_config;

// ── Helpers ─────────────────────────────────────────────────────

/// Instantiate a graph to trigger `Config::validate()`.
fn validate(cfg: Config<u64>) {
    let _g: GvGraph<u64, u64, 8> = GvGraph::new(cfg);
}

// ── depth_create / depth_evict boundary (D-I3) ─────────────────

#[test]
fn depth_create_one_less_than_evict_is_valid() {
    // depth_create == depth_evict - 1 is the tightest valid gap.
    validate(Config {
        depth_create: 5,
        depth_evict: 6,
        ..default_config()
    });
}

#[test]
#[should_panic(expected = "D_create")]
fn depth_create_equal_to_evict_is_invalid() {
    validate(Config {
        depth_create: 6,
        depth_evict: 6,
        ..default_config()
    });
}

#[test]
#[should_panic(expected = "D_create")]
fn depth_create_greater_than_evict_is_invalid() {
    validate(Config {
        depth_create: 7,
        depth_evict: 6,
        ..default_config()
    });
}

// ── depth_create >= 1 ───────────────────────────────────────────

#[test]
fn depth_create_exactly_one_is_valid() {
    validate(Config {
        depth_create: 1,
        depth_evict: 2,
        ..default_config()
    });
}

#[test]
#[should_panic(expected = "D_create")]
fn depth_create_zero_is_invalid() {
    validate(Config {
        depth_create: 0,
        depth_evict: 6,
        ..default_config()
    });
}

// ── alpha_relax boundaries ──────────────────────────────────────

#[test]
fn alpha_relax_just_above_zero_is_valid() {
    validate(Config {
        alpha_relax: f64::MIN_POSITIVE,
        ..default_config()
    });
}

#[test]
fn alpha_relax_just_below_one_is_valid() {
    validate(Config {
        alpha_relax: 1.0 - f64::EPSILON,
        ..default_config()
    });
}

#[test]
#[should_panic(expected = "alpha_relax")]
fn alpha_relax_zero_is_invalid() {
    validate(Config {
        alpha_relax: 0.0,
        ..default_config()
    });
}

#[test]
#[should_panic(expected = "alpha_relax")]
fn alpha_relax_one_is_invalid() {
    validate(Config {
        alpha_relax: 1.0,
        ..default_config()
    });
}

#[test]
#[should_panic(expected = "alpha_relax")]
fn alpha_relax_negative_is_invalid() {
    validate(Config {
        alpha_relax: -0.1,
        ..default_config()
    });
}

#[test]
#[should_panic(expected = "alpha_relax")]
fn alpha_relax_above_one_is_invalid() {
    validate(Config {
        alpha_relax: 1.5,
        ..default_config()
    });
}

#[test]
#[should_panic(expected = "alpha_relax")]
fn alpha_relax_nan_is_invalid() {
    validate(Config {
        alpha_relax: f64::NAN,
        ..default_config()
    });
}

#[test]
#[should_panic(expected = "alpha_relax")]
fn alpha_relax_positive_infinity_is_invalid() {
    validate(Config {
        alpha_relax: f64::INFINITY,
        ..default_config()
    });
}

#[test]
#[should_panic(expected = "alpha_relax")]
fn alpha_relax_negative_infinity_is_invalid() {
    validate(Config {
        alpha_relax: f64::NEG_INFINITY,
        ..default_config()
    });
}

// ── Budget headroom boundary ────────────────────────────────────
//
// The formula is: budget > max(3^(buffer+1), 2*(D_c - 1))
// where buffer = D_evict - D_create.
//
// With D_create=3, D_evict=6, buffer=3:
//   headroom  = 3^4 = 81
//   convergence = 2 * 2 = 4
//   required = max(81, 4) = 81
//   ⇒ budget must be > 81, so 82 is the minimum valid budget.

#[test]
fn budget_at_minimum_valid_value() {
    validate(Config {
        budget: Some(82),
        ..default_config()
    });
}

#[test]
#[should_panic(expected = "budget")]
fn budget_exactly_at_required_is_invalid() {
    // budget == required (81) should fail since we need budget > required.
    validate(Config {
        budget: Some(81),
        ..default_config()
    });
}

#[test]
#[should_panic(expected = "budget")]
fn budget_below_required_is_invalid() {
    validate(Config {
        budget: Some(10),
        ..default_config()
    });
}

// ── Budget with minimal buffer (buffer=1) ───────────────────────
//
// D_create=2, D_evict=3, buffer=1:
//   headroom  = 3^2 = 9
//   convergence = 2 * 1 = 2
//   required = 9
//   ⇒ minimum valid budget = 10.

#[test]
fn budget_minimal_buffer_at_boundary() {
    validate(Config {
        depth_create: 2,
        depth_evict: 3,
        budget: Some(10),
        ..default_config()
    });
}

#[test]
#[should_panic(expected = "budget")]
fn budget_minimal_buffer_exactly_at_required() {
    validate(Config {
        depth_create: 2,
        depth_evict: 3,
        budget: Some(9),
        ..default_config()
    });
}

// ── Budget where convergence dominates ──────────────────────────
//
// D_create=50, D_evict=51, buffer=1:
//   headroom  = 3^2 = 9
//   convergence = 2 * 49 = 98
//   required = 98
//   ⇒ minimum valid budget = 99.

#[test]
fn budget_convergence_dominates_at_boundary() {
    validate(Config {
        depth_create: 50,
        depth_evict: 51,
        budget: Some(99),
        ..default_config()
    });
}

#[test]
#[should_panic(expected = "budget")]
fn budget_convergence_dominates_exactly_at_required() {
    validate(Config {
        depth_create: 50,
        depth_evict: 51,
        budget: Some(98),
        ..default_config()
    });
}

// ── No budget is always valid ───────────────────────────────────

#[test]
fn no_budget_skips_headroom_check() {
    validate(Config {
        budget: None,
        ..default_config()
    });
}
