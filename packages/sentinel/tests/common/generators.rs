// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Value generators for test data.

/// Generate `count` values in a narrow range with leading nibble
/// `nibble` and sequential low bits.
pub fn cell_values(nibble: u128, count: usize) -> Vec<u128> {
    (0..count).map(|i| (nibble << 124) | (i as u128 + 1)).collect()
}

/// Generate values with a dense bit pattern to create
/// structurally novel data relative to [`cell_values()`].
pub fn anomalous_values(nibble: u128, count: usize) -> Vec<u128> {
    // Set a dense block of high bits in the middle — structurally
    // very different from the sparse sequential values above.
    (0..count)
        .map(|i| (nibble << 124) | 0x0FFF_FFFF_FFFF_FFFF_FFFF_FFFF_0000_0000 | (i as u128))
        .collect()
}
