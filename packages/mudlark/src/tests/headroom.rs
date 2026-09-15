// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Unit tests for private budget-headroom arithmetic.
//!
//! # Test index
//!
//! | Test | Focus |
//! |------|-------|
//! | [`required_headroom_saturates_convergence_overflow`] | unrepresentable convergence headroom saturates |

#[test]
fn required_headroom_saturates_convergence_overflow() {
    assert_eq!(crate::required_headroom(0, usize::MAX / 2 + 1), usize::MAX);
}
