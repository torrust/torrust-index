// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Crate-level tests for `pub(crate)` modules.
//!
//! These test the internal API surface of modules that are not publicly
//! exported (`linalg`, `model`). They live under `src/tests/` per the
//! project test-level conventions.
//!
//! # Module index
//!
//! | Module | Focus |
//! |--------|-------|
//! | [`aggregate`] | `compute_aggregates`, `SentinelSubScores` (´sec:feature:aggregate´) |
//! | [`assembly`] | `assemble_and_standardise_assessment`, feature placement (´chap:spec:feature-vector´) |
//! | [`dimension_map`] | `DimensionMap` rebuild, anchor projection, serde round-trip |
//! | [`health_convergence`] | Convergence tracking for warm-up stages |
//! | [`health_infrastructure`] | Health infrastructure acceptance tests, tiered by cost (´dec:health:tiered-queries´) |
//! | [`health_output_register`] | Generated whole-report health field projection and exact-byte staleness refusal |
//! | [`guidance`] | Core label guidance query and scorers (´sec:guidance:core´) |
//! | [`identity`] | `CompetitiveSetIndex`, `CellOutcomeState`, maintenance loop |
//! | [`identity_maintenance`] | Maintenance loop, G-V graph integration, lifecycle events |
//! | [`ingestion`] | Report ingestion pipeline (tests) |
//! | [`ledger`] | `LedgerEntry`, `SentinelLedger`, routing, cell-set, GC |
//! | [`linalg_bridge`] | Cholesky, inverse, subvector, solve |
//! | [`linalg_convert`] | `faer` ↔ `Vec<f64>` round-trips |
//! | [`linalg_serde_support`] | serde round-trips for `faer` types (feature-gated) |
//! | [`linalg_symmetric`] | `SymmetricMatrix` public-method tests |
//! | [`model_bayesian`] | `BayesianLinearModel` construction, mutation, snapshot |
//! | [`model_marginalise`] | Schur complement marginalisation by half-solve (´dec:posterior:half-solve´) |
//! | [`model_parameters`] | `ModelParameters` serde and construction |
//! | [`model_recompute`] | Cholesky recomputation (´dec:posterior:recomputation-trigger´): triggers, outcomes, health |
//! | [`pending`] | `PendingBuffer`, `PendingAssessment`, entry types |
//! | [`assess_pipeline`] | Assessment pipeline orchestration, pending buffer, concurrency |
//! | [`snapshot`] | `ModelSnapshot`, `WorkingCopy`, `SharedState`, ArcSwap concurrency |
//! | [`standardisation`] | `standardise_in_place`, `update_standardisation` (´sec:standardisation:online´) |
//! | [`types`] | `PersistentTimestamp` round-trip, `hours_since` clamping, construction (´dec:clock:two-domains´) |
//! | [`owner`] | `ModelOwner`, channels, `Assayer` skeleton, preemption, concurrency |
//! | [`report`] | `ReportIndex` routing, `SentinelSlot`, type traits |
//! | [`signal_schema`] | `SignalSchemaIndex::get` (pub(crate) lookup) |
//! | [`numerics`] | `decay_factor`, `decay_factor_since`, `decay_factor_elapsed` |
//! | [`label_pipeline`] | Label pipeline step ordering, CP5/CP6, replay, multi-label |
//! | [`lifecycle`] | Lifecycle registration, deregistration, compound batch |
//! | [`lifecycle_api`] | Lifecycle API surface: six public methods plus pre-seed (´dec:construction:six-methods´) |
//! | [`lifecycle_integration`] | Lifecycle integration: end-to-end multi-step, health, pre-seed, concurrency |
//! | [`metrics_export`] | Metrics export acceptance tests, passive throughout (´dec:metrics:passive-export´) |
//! | [`resonance_tags`] | Tag structure, ordering, kernel-guard (´app:spec:resonance-rendering´) |
//! | [`resonance_ambiguity`] | The ambiguity gauge and its entropy decomposition (´sig:rendering:ambiguity-gauge´) |
//! | [`resonance_crossover`] | Boundary crossover matching (´eq:landscape:crossover´) |
//! | [`resonance_robustness`] | Extreme inputs, domination, monotonicity, Q-floor (´dec:derivation:pure-transform´) |
//! | [`resonance_golden`] | Golden-value tests for the worked examples (´setup:landscape:worked-assumptions´) |
//! | [`resonance_landscape`] | The decision landscape against the worked chapter's tables (´sig:landscape:output´) |
//! | [`resonance_sufficiency`] | Derivation depends only on the sufficient statistic + explicit host inputs |
//! | [`outcome_drift`] | Outcome axis evaluation (´chap:spec:axis-prediction´) and drift accumulators, with the smoothed measures converging on the residuals actually seen (´claim:wellness:the-smoothed-residual-magnitude-and-sign-converge-on-the-residuals-actually-seen´) |
//! | [`source_audit`] | Repository hygiene: no residual `stub_…` definitions, `stubs.rs` stays deleted |
//! | [`builder`] | `AssayerBuilder` construction, host channel policy, error handling |
//! | [`benchmarks`] | Performance benchmark tests (`#[ignore]`): latency and throughput sanity checks |
//! | [`config_projection`] | The configuration chapter projected onto the package declarations it governs (´chap:spec:configuration´) |

// Test modules use convenience patterns that trigger pedantic lints
#![allow(clippy::doc_markdown)]
#![allow(clippy::float_cmp)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::similar_names)]
#![allow(clippy::significant_drop_tightening)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::print_stderr)]
#![allow(clippy::print_stdout)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::unreadable_literal)]

// Common test helpers
mod helpers;

mod aggregate;
mod api_surfaces;
mod assembly;
mod assess_pipeline;
mod benchmarks;
mod builder;
mod config_projection;
mod dimension_map;
mod guidance;
mod health_convergence;
mod health_infrastructure;
mod health_output_register;
mod hibernation;
mod identity;
mod identity_maintenance;
mod ingestion;
mod label_pipeline;
mod ledger;
mod lifecycle;
mod lifecycle_api;
mod lifecycle_integration;
mod linalg_bridge;
mod linalg_convert;
mod linalg_symmetric;
mod metrics_export;
mod model_bayesian;
mod model_marginalise;
mod model_parameters;
mod model_recompute;
mod numerics;
mod outcome_drift;
mod owner;
mod pending;
mod report;
mod resonance_ambiguity;
mod resonance_crossover;
mod resonance_golden;
mod resonance_landscape;
mod resonance_robustness;
mod resonance_sufficiency;
mod resonance_tags;
mod scoring_axes;
mod signal_schema;
mod snapshot;
mod source_audit;
mod standardisation;
mod types;

#[cfg(feature = "serde")]
mod linalg_serde_support;

#[cfg(feature = "serde")]
mod persistence;
