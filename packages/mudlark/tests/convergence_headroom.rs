// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Integration test for convergence headroom arithmetic through the public configuration path.
//!
//! # Test index
//!
//! | Test | Focus |
//! |------|-------|
//! | [`budget_convergence_witness_preserves_64_bit_requirement`] | the executed witness retains its representable 64-bit requirement |

#[cfg(target_pointer_width = "64")]
use torrust_mudlark::{Config, GvGraph};

#[cfg(target_pointer_width = "64")]
#[test]
#[should_panic(expected = "= 4294967296")]
fn budget_convergence_witness_preserves_64_bit_requirement() {
    drop(GvGraph::<u64, u64, 8>::new(Config {
        split_threshold: 1,
        depth_create: 2_147_483_649,
        depth_evict: 2_147_483_650,
        budget: Some(100),
        alpha_relax: 0.5,
        bounded_eviction: false,
    }));
}
