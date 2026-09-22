// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Risk assessment helpers and host-owned challenge effectiveness tracking.
//!
//! This module provides Core risk helpers plus the standalone Companion Tracker
//! for challenge effectiveness. Companion state is owned and persisted by the
//! host, not the Core snapshot.
//!
//! # Cross-References
//!
//! - (´dec:ordering:label-function´) — the label path these helpers sit in
//! - (´chap:spec:challenge-effectiveness´) — the Companion Tracker

pub mod blend;
pub mod calibration;
pub mod challenge;
