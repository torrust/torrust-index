// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Invariant-checked debug-mode mirrors of every benchmark workload.
//!
//! Each mirror replays the same plan × config pair from the
//! corresponding Criterion benchmark family at full bench scale
//! with full invariant checking enabled.  This ensures the workload
//! shapes that are performance-tested are also correctness-validated.
//!
//! See Phase 9 of `docs/benchmarking-implementation-plan.md`.

mod bench_mirrors {
    mod extract;
    mod lifecycle;
    mod observe;
    mod pathological;
    mod query;
    mod spray;
}

mod support;

pub use support::init_tracing;
