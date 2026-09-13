// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Model-owner thread and command types.
//!
//! This module provides the infrastructure for the single-writer
//! model-owner thread that processes lifecycle commands and labels
//! from two bounded crossbeam channels.
//!
//! # Module Structure
//!
//! - [`commands`] — `ModelOwnerCommand`, `SequencedLabel`, lifecycle
//!   types, label types, and registration payloads.
//! - [`thread`] — `ModelOwner`: the event loop that drains commands
//!   before processing labels.
//! - [`label_path`] — 17-step label pipeline orchestration (`process_label()`).
//! - [`lifecycle`] — Lifecycle event handling (`handle_lifecycle_submission()`).
//!
//! # Cross-References
//!
//! - (´dec:concurrency:single-steward´) — one steward owns the working copy
//! - (´dec:concurrency:named-thread´) — the steward is a dedicated named thread
//! - (´dec:ordering:label-function´) — the label path is one function in ordered groups
//! - (´dec:construction:compound-batch´) — lifecycle events arrive together, gather into order-free chains and publish once per chain

pub mod commands;
pub mod label_path;
pub mod lifecycle;
pub mod thread;
