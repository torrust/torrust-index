// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Value generators for test data.

/// Generate `count` values in a narrow range with leading nibble
/// `nibble` and sequential low bits.
///
/// The nibble is four bits wide, which is what the shift leaves room for: a
/// value of sixteen or above pushes its high bits off the top of the
/// coordinate and lands on the range sixteen below it, so a caller sweeping
/// past fifteen would revisit ranges it believed were new. The assertion
/// refuses that rather than letting the aliasing pass as traffic.
/// [`cell_values_prefix`] is the generator for a wider sweep.
pub fn cell_values(nibble: u128, count: usize) -> Vec<u128> {
    debug_assert!(
        nibble < 16,
        "cell_values takes a four-bit nibble; cell_values_prefix addresses a wider sweep"
    );
    (0..count).map(|i| (nibble << 124) | (i as u128 + 1)).collect()
}

/// Generate `count` values in a narrow range with leading six-bit `prefix`
/// and sequential low bits.
///
/// Six bits address sixty-four leading ranges, each a $2^{122}$-wide
/// interval, so a sweep over `0..64` reaches sixty-four ranges that differ
/// inside their first six bits. The four-bit generator cannot express such a
/// sweep: past fifteen its ranges repeat, and a spray that believed it was
/// touching sixty-four ranges would be touching sixteen of them four times
/// each — traffic concentrated enough to build the very structure the spray
/// was meant to spread thin.
pub fn cell_values_prefix(prefix: u128, count: usize) -> Vec<u128> {
    debug_assert!(prefix < 64, "cell_values_prefix takes a six-bit prefix");
    (0..count).map(|i| (prefix << 122) | (i as u128 + 1)).collect()
}

/// Generate values with a dense bit pattern to create
/// structurally novel data relative to [`cell_values()`].
pub fn anomalous_values(nibble: u128, count: usize) -> Vec<u128> {
    debug_assert!(nibble < 16, "anomalous_values takes a four-bit nibble");
    // Set a dense block of high bits in the middle — structurally
    // very different from the sparse sequential values above.
    (0..count)
        .map(|i| (nibble << 124) | 0x0FFF_FFFF_FFFF_FFFF_FFFF_FFFF_0000_0000 | (i as u128))
        .collect()
}
