// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! State persistence: checkpoint, journal, and recovery.
//!
//! This module implements crash-safe persistence for the Assayer's
//! model state as a periodic checkpoint beside a self-contained journal
//! (´dec:durability:checkpoint-journal´):
//!
//! - **Checkpoint** — Atomic full-state snapshot with magic/version/CRC32
//!   validation. Written via a rename whose directory entry is made durable
//!   before the write reports success, so the journal truncation that follows
//!   cannot outlive the checkpoint that justified it.
//! - **Journal** — Append-only label log for replay between checkpoints.
//!   Length-prefixed postcard records; truncated trailing records are
//!   silently skipped on recovery.
//! - **Recovery** — Loads checkpoint, applies bulk time-decay, returns
//!   the working copy plus un-replayed journal entries. Falls back to
//!   cold start on any failure.
//! - **Scheduler** — Periodic checkpoint trigger on a dedicated thread.
//!
//! # Cross-References
//!
//! - (´dec:durability:checkpoint-journal´) — what survives a restart, and in what two parts
//! - (´dec:durability:file-framing´) — the on-disk framing both files are written in
//! - (´dec:durability:decay-once´) — elapsed decay applied exactly once on restore

#[cfg(feature = "serde")]
pub mod checkpoint;
#[cfg(feature = "serde")]
pub mod journal;
#[cfg(feature = "serde")]
pub mod recovery;
pub mod scheduler;

#[cfg(feature = "serde")]
use std::path::Path;
#[cfg(feature = "serde")]
use std::{fs, io};

/// Renames `from` over `to` and persists the directory entry the rename creates.
///
/// [`File::sync_all`](std::fs::File::sync_all) on the temporary file persists
/// that file's own data and metadata. The name the rename then publishes is
/// metadata of the *parent directory*, and on a journalling filesystem it
/// reaches stable storage only when the directory itself is synced. Without
/// that step a power loss can leave the new file's contents durable and the
/// directory still naming the old one — which is precisely the window the
/// checkpoint-and-journal protocol cannot tolerate, because the model owner
/// truncates the journal on the strength of a checkpoint the next start would
/// then not find, discarding labels that only the new checkpoint held
/// (´dec:durability:checkpoint-journal´).
///
/// # Platform strategy
///
/// On Unix the parent directory is opened and synced, which is the mechanism
/// the platform offers for making a directory entry durable. On other targets
/// the rename is left as the operating system publishes it: no portable handle
/// to a directory exists there to sync, and the engine's deployment target is
/// Unix, so the platform that carries the extra step is the platform whose
/// failure mode it closes.
///
/// # Errors
///
/// Returns the `io::Error` of the rename, of opening the parent directory, or
/// of syncing it.
#[cfg(feature = "serde")]
pub fn rename_durably(from: &Path, to: &Path) -> Result<(), io::Error> {
    fs::rename(from, to)?;
    sync_parent_dir(to)
}

/// Syncs the directory holding `path`, so the entry naming it survives a power
/// loss. A path with no parent component is taken as naming the working
/// directory, which is the directory its entry lives in.
#[cfg(all(unix, feature = "serde"))]
fn sync_parent_dir(path: &Path) -> Result<(), io::Error> {
    let parent = match path.parent() {
        Some(p) if !p.as_os_str().is_empty() => p,
        _ => Path::new("."),
    };
    fs::File::open(parent)?.sync_all()
}

/// Without a portable directory handle there is nothing to sync; the rename
/// itself is the whole protocol on this platform.
#[cfg(all(not(unix), feature = "serde"))]
fn sync_parent_dir(_path: &Path) -> Result<(), io::Error> {
    Ok(())
}
