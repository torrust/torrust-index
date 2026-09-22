// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Reproducible, framework-free invariant sweeps.
//!
//! [`run_seeded_sweep`] owns the common failure envelope around the harness generator: every rejected case reports the declared seed, its zero-based index, the complete drawn input, and the witness's reason. The seed remains mandatory because the generator has no default, and no shrinking or hidden entropy can move the failure away from the stream that produced it (´dec:harness:seeded-sweeps´).

use std::fmt::{Debug, Display};

use super::rng::TestRng;

/// Draws `case_count` inputs from one declared seed and presents each to an invariant witness.
///
/// `Debug` is the replay format for the drawn input, so case types should contain every generated value the witness consumes rather than only a summary of it.
///
/// # Panics
///
/// Panics when `witness` rejects a drawn input. The panic reports the seed, case index, complete input, and rejection reason so that the exact case can be reconstructed from the output.
#[track_caller]
pub fn run_seeded_sweep<T, E>(
    seed: u64,
    case_count: usize,
    mut draw: impl FnMut(&mut TestRng) -> T,
    mut witness: impl FnMut(&T) -> Result<(), E>,
) where
    T: Debug,
    E: Display,
{
    let mut rng = TestRng::new(seed);

    for case_index in 0..case_count {
        let input = draw(&mut rng);
        if let Err(reason) = witness(&input) {
            panic!("seeded sweep failed: seed=0x{seed:016x}, case_index={case_index}, input={input:#?}, reason={reason}");
        }
    }
}
