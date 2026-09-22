// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Derivation Function support: the decision landscape and its rendering.
//!
//! The derivation transforms a Core `RiskAssessment`, a host
//! `ChannelPolicy` and an explicit challenge posterior into a
//! `DecisionLandscape` (´sig:landscape:output´); the resonance
//! rendering is an optional layer over that object
//! (´dec:landscape:rendering-optional´) taking the display
//! configuration the landscape deliberately excludes.
//!
//! # Module Structure
//!
//! - [`landscape`] — `derive_landscape()`, `DecisionLandscape`, `ActionCrossover`, `ActionRegime`
//! - [`derivation`] — `render_resonances()`, `ResonanceProfile`, `ResonanceConfig`, profile utilities
//! - [`tags`] — `TagResonance`, `cauchy_kernel()`
//! - [`ambiguity`] — `compute_profile_ambiguity()`, `ProfileAmbiguity`
//! - [`channel`] — `ChannelPolicy`, `RewardParameters`, `ChannelDerivedConstants`
//!
//! # Cross-References
//!
//! - (´dec:derivation:pure-transform´) — what the resonance derivation this
//!   module implements is: a transform that reads no Core state and cannot fail
//! - (´chap:spec:derivation-interface´) — the Derivation Function interface
//! - (´chap:spec:crossover-landscape´) — the resonance model these per-tag
//!   scores are computed against
//! - (´chap:spec:worked-landscapes´) — the derivation worked end to end
//! - (´chap:spec:exploration-and-guidance´) — the exploration metrics and the
//!   label guidance they serve

pub mod ambiguity;
pub mod channel;
pub mod derivation;
pub mod landscape;
pub mod tags;
