// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Append-only label journal for crash-safe replay.
//!
//! The file opens with an 8-byte header, then length-prefixed postcard
//! records:
//!
//! | Offset | Size | Content |
//! |--------|------|---------|
//! | 0 | 4 | Magic bytes `ASAJ` |
//! | 4 | 4 | Format version (little-endian `u32`) |
//! | 8+ | 4 | Payload length (little-endian `u32`) |
//! | 12+ | variable | Postcard-serialised `JournalEntry` |
//!
//! On recovery, truncated trailing records (from a crash mid-write)
//! are silently skipped — only complete records are returned.
//!
//! ## Format Version Policy
//!
//! As for the checkpoint: a version mismatch discards the journal by
//! design — the writer starts a fresh file and recovery replays
//! nothing. The entry layout is internal and unstable, and postcard
//! records of another layout must be refused on the version rather
//! than misread (´dec:durability:checkpoint-journal´).
//!
//! # Cross-References
//!
//! - (´dec:durability:checkpoint-journal´) — the self-contained journal replay reads
//! - (´dec:durability:file-framing´) — the length-prefixed framing this file appends in

use std::io::{self, Write};
use std::path::Path;
use std::{fmt, fs};

use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::owner::commands::{LabelData, PendingContext};
use crate::types::PersistentTimestamp;

/// Magic bytes identifying an Assayer journal file.
///
/// The four opening bytes of the header: the same package prefix the checkpoint
/// carries, with the role byte that tells the two apart, so a checkpoint offered
/// as a journal is refused on that byte (´dec:durability:file-framing´).
///
/// ´const:assayer:journal-file-signature´ (´alg:const:form´)
/// ´const:assayer:journal-file-signature-form-x4a6778b0´
const JOURNAL_MAGIC: [u8; 4] = *b"ASAJ";

/// Current journal format version. Incremented on any entry-layout
/// change. The repair campaign introduced the header at version 1 when
/// the journal context became self-contained, and bumped it to 2 when
/// the context gained the four reconstruction fields — the two Sentinel
/// lists and the two frozen identity feature blocks
/// (´def:runtime:pending-entry´). A later change bumped it to 3, for the
/// spatial-axis order the context now carries beside its extractions
/// (´entry:assayer:wl-feature-frozen-slot-truncation´). This change bumps
/// it to 4 for the codec change: the record payload is postcard-encoded,
/// and because the two encodings agree byte for byte on integers below 128
/// a record of the previous generation would not reliably fail to decode.
/// Headerless files are the pre-version layout; both they and prior
/// versions are refused on the version.
///
/// Refusing on the version is what the structural check asks of a
/// restore (´dec:durability:structural-compatibility´), and the bump is
/// the design rather than an accident.
///
/// The bump is the whole compatibility answer for this field, and refusal
/// is the right answer rather than a tolerated default. A context written
/// before the order was stored has extractions whose pairs are laid out in
/// an order nothing records; accepting it and reading the missing field as
/// an empty list would silently place no pairs at all, and inferring an
/// order from the current registry would assert the very thing the field
/// exists to stop being assumed. Refusing costs the pending entries in the
/// journal, which are assessments still awaiting a label — the same loss a
/// restore already accepts for a checkpoint of another generation.
///
/// ´const:assayer:journal-layout-generation´ (´alg:const:version´)
/// ´const:assayer:journal-layout-generation-version-4´
const JOURNAL_FORMAT_VERSION: u32 = 4;

/// Size of the journal header (magic + version).
///
/// Two four-byte fields, signature and generation, and no third: the journal
/// carries no header checksum because its per-record length prefix is what
/// bounds crash damage to a truncated tail, so its width is four bytes short of
/// the checkpoint's (´cor:durability:header-width´). It is read here as the
/// offset at which the first record begins.
///
/// ´const:assayer:journal-header-width´ (´alg:const:count´)
/// ´const:assayer:journal-header-width-count-8´
const JOURNAL_HEADER_SIZE: usize = 8;

// ═══════════════════════════════════════════════════════════════════════════════
// Journal Entry
// ═══════════════════════════════════════════════════════════════════════════════

/// A single journal record.
///
/// Each label processed by the model-owner thread is journaled before
/// the update is applied, enabling replay after crash recovery.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JournalEntry {
    /// Label sequence number (monotonically increasing).
    pub seq: u64,
    /// The label data from the caller.
    pub label: LabelData,
    /// Stripped assessment context for replay.
    pub context: PendingContext,
    /// Timestamp when this entry was journaled.
    pub timestamp: PersistentTimestamp,
}

impl JournalEntry {
    /// Creates a journal entry with a placeholder sequence number.
    ///
    /// The `seq` field is set to `0` and will be overwritten by
    /// [`JournalWriter::append()`] when the entry is persisted.
    #[must_use]
    pub fn unsequenced(label: LabelData, context: PendingContext) -> Self {
        Self {
            seq: 0,
            label,
            context,
            timestamp: PersistentTimestamp::now(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Error Types
// ═══════════════════════════════════════════════════════════════════════════════

/// Error reading or writing the journal.
#[derive(Debug)]
pub enum JournalError {
    /// I/O error.
    Io(io::Error),
    /// A record was corrupt (but not merely truncated at the tail).
    Corrupt {
        /// Byte offset where the corruption was detected.
        offset: u64,
        /// Description of the issue.
        reason: String,
    },
    /// The file does not carry this build's journal format version.
    VersionMismatch {
        /// Version found in the header; `None` for a headerless
        /// (pre-version) file.
        found: Option<u32>,
        /// Version expected by this binary.
        expected: u32,
    },
}

impl fmt::Display for JournalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "journal I/O error: {e}"),
            Self::Corrupt { offset, reason } => {
                write!(f, "journal corrupt at offset {offset}: {reason}")
            }
            Self::VersionMismatch { found, expected } => match found {
                Some(found) => write!(f, "journal version mismatch: found {found}, expected {expected}"),
                None => write!(
                    f,
                    "journal version mismatch: headerless pre-version file, expected {expected}"
                ),
            },
        }
    }
}

impl std::error::Error for JournalError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Corrupt { .. } | Self::VersionMismatch { .. } => None,
        }
    }
}

impl From<io::Error> for JournalError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Writer
// ═══════════════════════════════════════════════════════════════════════════════

/// Appends a journal entry to an open file, durably.
///
/// Each record is length-prefixed: `[u32 LE length][postcard payload]`.
///
/// The record is on stable storage when this returns. That is the whole
/// point of the call: the label path journals an entry and then acknowledges
/// the submission, and the acknowledgement consumes the pending assessment
/// the label was matched against, so a record still only in the page cache
/// when the power fails is a label the host was told was durable and that no
/// longer exists in either artefact (´dec:durability:checkpoint-journal´).
///
/// The sync is `sync_data` rather than `sync_all`. The record is appended,
/// and the only metadata a reader needs in order to find it is the file's
/// length, which `sync_data` is defined to persist because the data cannot be
/// retrieved without it; the timestamps `sync_all` would additionally flush
/// are read by nothing here, and paying for them once per label buys no
/// guarantee this function makes.
///
/// # Errors
///
/// Returns `io::Error` on serialisation failure, on write failure, or when
/// the record cannot be synced — in which case it is not durable and the
/// caller must not acknowledge it.
pub fn append_journal_entry(file: &mut fs::File, entry: &JournalEntry) -> Result<(), io::Error> {
    let payload = crate::serde_codec::serialize(entry).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let len = u32::try_from(payload.len())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "journal entry too large (> 4 GiB)"))?;

    file.write_all(&len.to_le_bytes())?;
    file.write_all(&payload)?;
    // `File::flush` is a no-op: `std::fs::File` holds no userspace buffer, so
    // the call that used to stand here returned success over bytes that had
    // reached the page cache and nothing further.
    file.sync_data()?;

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════════
// Reader
// ═══════════════════════════════════════════════════════════════════════════════

/// Reads all complete journal entries from `path`.
///
/// Truncated trailing records (from a crash during write) are silently
/// skipped — only complete, deserializable records are returned.
///
/// # Errors
///
/// Returns `JournalError` on I/O failure. Deserialization failures in
/// the middle of the file (not at the tail) are treated as corruption.
pub fn read_journal(path: &Path) -> Result<Vec<JournalEntry>, JournalError> {
    let data = match fs::read(path) {
        Ok(d) => d,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(JournalError::Io(e)),
    };

    // An empty file is a journal with nothing to replay; anything else
    // must open with this build's header, or it is another layout and
    // is refused on the version rather than misread.
    if data.is_empty() {
        return Ok(Vec::new());
    }
    if data.len() < JOURNAL_HEADER_SIZE || data[0..4] != JOURNAL_MAGIC {
        return Err(JournalError::VersionMismatch {
            found: None,
            expected: JOURNAL_FORMAT_VERSION,
        });
    }
    let version = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
    if version != JOURNAL_FORMAT_VERSION {
        return Err(JournalError::VersionMismatch {
            found: Some(version),
            expected: JOURNAL_FORMAT_VERSION,
        });
    }

    let mut entries = Vec::new();
    let mut cursor: usize = JOURNAL_HEADER_SIZE;
    let total = data.len();

    while cursor + 4 <= total {
        // Read length prefix.
        let len_bytes: [u8; 4] = match data[cursor..cursor + 4].try_into() {
            Ok(b) => b,
            Err(_) => break, // Truncated length prefix — stop.
        };
        let payload_len = u32::from_le_bytes(len_bytes) as usize;

        // Check if the full payload is available.
        let payload_start = cursor + 4;
        let payload_end = payload_start + payload_len;
        if payload_end > total {
            // Truncated record at the tail — silently skip.
            break;
        }

        let payload = &data[payload_start..payload_end];

        match crate::serde_codec::deserialize::<JournalEntry>(payload) {
            Ok(entry) => {
                entries.push(entry);
                cursor = payload_end;
            }
            Err(e) => {
                // If this is the last record, treat it as truncation.
                if payload_end == total {
                    break;
                }
                // Otherwise, it's a genuine corruption in the middle.
                return Err(JournalError::Corrupt {
                    offset: cursor as u64,
                    reason: format!("postcard deserialisation failed: {e}"),
                });
            }
        }
    }

    Ok(entries)
}

/// Opens or creates a journal file for appending, stamping the format
/// header onto a fresh (empty) file.
///
/// # Errors
///
/// Returns `io::Error` on filesystem failure.
#[allow(dead_code)] // used by model owner in Phase 1+ (´dec:durability:checkpoint-journal´)
pub fn open_journal_for_append(path: &Path) -> Result<fs::File, io::Error> {
    let mut file = fs::OpenOptions::new().create(true).append(true).open(path)?;
    if file.metadata()?.len() == 0 {
        file.write_all(&JOURNAL_MAGIC)?;
        file.write_all(&JOURNAL_FORMAT_VERSION.to_le_bytes())?;
        file.flush()?;
    }
    Ok(file)
}

/// Replaces the file at `path` with a fresh, header-only journal.
///
/// The discard path for a version mismatch: as for the checkpoint, a
/// journal of another format cold-starts by design.
fn recreate_journal(path: &Path) -> Result<fs::File, io::Error> {
    let mut file = fs::OpenOptions::new().create(true).write(true).truncate(true).open(path)?;
    file.write_all(&JOURNAL_MAGIC)?;
    file.write_all(&JOURNAL_FORMAT_VERSION.to_le_bytes())?;
    file.sync_all()?;
    drop(file);
    open_journal_for_append(path)
}

// ═══════════════════════════════════════════════════════════════════════════════
// JournalWriter
// ═══════════════════════════════════════════════════════════════════════════════

/// Struct-based journal writer for use on the `Assayer` struct.
///
/// Wraps an open file handle for append-mode journal writes.
/// Intended to be stored as `Option<Arc<Mutex<JournalWriter>>>` on the
/// `Assayer`, where the Mutex serialises writes from `label()`.
///
/// Maintains a monotonically increasing sequence counter. The counter
/// is initialised from the max seq of existing journal entries at
/// open time: the sequence number is assigned at journal write time
/// (´dec:durability:checkpoint-journal´).
///
/// # Cross-References
///
/// - (´dec:durability:checkpoint-journal´) — the journal this writer appends to
/// - (´dec:contracts:construction-surface´) — the builder that creates it
#[allow(dead_code)] // L4: used by `label()`
pub struct JournalWriter {
    /// Open file handle in append mode.
    file: fs::File,
    /// Next sequence number to assign.
    next_seq: u64,
    /// The journal's path, for the sequence-aware truncation's rewrite.
    path: std::path::PathBuf,
}

#[allow(dead_code)] // L4: methods used by `label()`
impl JournalWriter {
    /// Opens or creates a journal file at `path` for appending.
    ///
    /// Reads existing entries to determine the starting sequence
    /// number. If the file contains entries, `next_seq` is set to
    /// `max(existing seq) + 1`. Otherwise it starts at 1.
    ///
    /// # Errors
    ///
    /// Returns `io::Error` on filesystem failure.
    pub fn open(path: &Path) -> Result<Self, io::Error> {
        // Read existing entries to find the max seq.
        let max_seq = match read_journal(path) {
            Ok(entries) => entries.iter().map(|e| e.seq).max().unwrap_or(0),
            Err(JournalError::Io(e)) if e.kind() == io::ErrorKind::NotFound => 0,
            Err(JournalError::Io(e)) => return Err(e),
            Err(JournalError::Corrupt { offset, reason }) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("journal corrupt at offset {offset}: {reason}"),
                ));
            }
            Err(e @ JournalError::VersionMismatch { .. }) => {
                // As for the checkpoint: another format's journal is
                // discarded by design, never misread.
                warn!(%e, "journal of another format version discarded; starting fresh");
                let file = recreate_journal(path)?;
                return Ok(Self {
                    file,
                    next_seq: 1,
                    path: path.to_path_buf(),
                });
            }
        };

        let file = open_journal_for_append(path)?;
        Ok(Self {
            file,
            next_seq: max_seq + 1,
            path: path.to_path_buf(),
        })
    }

    /// Opens or creates a journal file at `path` for appending,
    /// starting sequence numbers after `last_processed_seq`.
    ///
    /// Use this when restoring from a checkpoint: the checkpoint's
    /// `last_processed_label_seq` provides a lower bound. The
    /// actual starting seq is `max(last_processed_seq, max_journal_seq) + 1`.
    ///
    /// # Errors
    ///
    /// Returns `io::Error` on filesystem failure.
    pub fn open_after(path: &Path, last_processed_seq: u64) -> Result<Self, io::Error> {
        let max_journal_seq = match read_journal(path) {
            Ok(entries) => entries.iter().map(|e| e.seq).max().unwrap_or(0),
            Err(JournalError::Io(e)) if e.kind() == io::ErrorKind::NotFound => 0,
            Err(JournalError::Io(e)) => return Err(e),
            Err(JournalError::Corrupt { offset, reason }) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("journal corrupt at offset {offset}: {reason}"),
                ));
            }
            Err(e @ JournalError::VersionMismatch { .. }) => {
                // As for the checkpoint: another format's journal is
                // discarded by design, never misread. The checkpoint's
                // mark still bounds the numbering.
                warn!(%e, "journal of another format version discarded; starting fresh");
                let file = recreate_journal(path)?;
                return Ok(Self {
                    file,
                    next_seq: last_processed_seq + 1,
                    path: path.to_path_buf(),
                });
            }
        };

        let file = open_journal_for_append(path)?;
        Ok(Self {
            file,
            next_seq: last_processed_seq.max(max_journal_seq) + 1,
            path: path.to_path_buf(),
        })
    }

    /// Truncates the journal to the entries a checkpoint has not absorbed:
    /// entries with `seq > last_processed_seq` are retained, everything at
    /// or below the mark is discarded. Called by the model owner after a
    /// successful checkpoint, through the same mutex the append path
    /// holds, so a label journalled after the checkpoint's capture is
    /// kept rather than truncated away with the absorbed prefix
    /// (´dec:durability:checkpoint-journal´).
    ///
    /// The rewrite is atomic: retained entries go to `{path}.new`, which
    /// is synced and renamed over the journal, and the directory the
    /// rename published the name in is synced too; a crash at any point
    /// leaves either the old journal or the new one, and never a
    /// truncation that is durable only until the power fails. That last
    /// step carries the same weight here as it does for the checkpoint:
    /// the two renames are the pair whose ordering the recovery argument
    /// rests on, so a truncation that outlived its checkpoint would lose
    /// exactly the labels the checkpoint had absorbed. The append handle
    /// is reopened over the renamed file and the sequence counter is
    /// untouched — it is already above everything retained.
    ///
    /// # Errors
    ///
    /// Returns `io::Error` on read, write or rename failure.
    pub fn truncate_to(&mut self, last_processed_seq: u64) -> Result<(), io::Error> {
        let entries = match read_journal(&self.path) {
            Ok(entries) => entries,
            Err(JournalError::Io(e)) if e.kind() == io::ErrorKind::NotFound => Vec::new(),
            Err(JournalError::Io(e)) => return Err(e),
            Err(JournalError::Corrupt { offset, reason }) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("journal corrupt at offset {offset}: {reason}"),
                ));
            }
            // Unreachable through the writer, which discards a
            // mismatched file at open; rewriting to a fresh header is
            // the same discard.
            Err(JournalError::VersionMismatch { .. }) => Vec::new(),
        };

        let tmp_path = self.path.with_extension("new");
        {
            let mut tmp = fs::File::create(&tmp_path)?;
            tmp.write_all(&JOURNAL_MAGIC)?;
            tmp.write_all(&JOURNAL_FORMAT_VERSION.to_le_bytes())?;
            for entry in entries.iter().filter(|entry| entry.seq > last_processed_seq) {
                append_journal_entry(&mut tmp, entry)?;
            }
            tmp.sync_all()?;
        }
        super::rename_durably(&tmp_path, &self.path)?;

        self.file = open_journal_for_append(&self.path)?;
        Ok(())
    }

    /// Appends a journal entry, assigning a monotonic sequence number.
    ///
    /// Returns the assigned sequence number.
    ///
    /// The entry is durable when this returns, which is what lets the label
    /// path acknowledge the submission it belongs to
    /// (´dec:durability:checkpoint-journal´).
    ///
    /// # Errors
    ///
    /// Returns `io::Error` on serialisation, write or sync failure. The
    /// sequence counter is not advanced on failure, so the number is free for
    /// the next attempt.
    pub fn append(&mut self, entry: &JournalEntry) -> Result<u64, io::Error> {
        let seq = self.next_seq;
        let sequenced = JournalEntry { seq, ..entry.clone() };
        append_journal_entry(&mut self.file, &sequenced)?;
        self.next_seq += 1;
        Ok(seq)
    }

    /// Returns the next sequence number that will be assigned.
    #[must_use]
    pub const fn next_seq(&self) -> u64 {
        self.next_seq
    }
}

impl fmt::Debug for JournalWriter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JournalWriter").finish_non_exhaustive()
    }
}
