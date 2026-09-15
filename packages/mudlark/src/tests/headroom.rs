// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Unit tests for private budget-headroom arithmetic.
//!
//! # Test index
//!
//! | Test | Focus |
//! |------|-------|
//! | [`structural_headroom_saturates_power_overflow`] | unrepresentable structural headroom saturates |
//! | [`required_headroom_saturates_convergence_overflow`] | unrepresentable convergence headroom saturates |

#[test]
fn structural_headroom_saturates_power_overflow() {
    // A depth buffer of 41 requires 3^42 = 109418989131512359209,
    // which exceeds u64::MAX = 18446744073709551615.
    assert_eq!(crate::structural_headroom(41), usize::MAX);
}

#[test]
fn required_headroom_saturates_convergence_overflow() {
    assert_eq!(crate::required_headroom(0, usize::MAX / 2 + 1), usize::MAX);
}
