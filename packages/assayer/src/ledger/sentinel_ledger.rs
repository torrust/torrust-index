// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`new_ledger_is_empty`] | ledger | Construction alone gives a Sentinel no cells at all — not even the root that covers the whole coordinate space. The root is installed by an explicit step, so a ledger handed out before that step is observably uninitialised rather than quietly pretending to be a blank slate that reads would route into. |
//! | [`ensure_root_creates_root`] | ledger | Ensuring the root installs the depth-zero cell that covers the entire coordinate space, taking the ledger to exactly one entry. Every read path terminates at that cell, so its existence is what makes routing total: no coordinate can fail to find somewhere to land. |
//! | [`ensure_root_idempotent`] | ledger | Asking for the root a second time is a no-op: the count stays at one and the existing root is left alone. Because call sites ensure the root defensively rather than tracking whether someone else already did, an idempotent step is what stops a re-entrant lifecycle from wiping a Sentinel's accumulated root history. |
//! | [`insert_updates_max_depth`] | ledger | The recorded maximum depth rises to meet each newly inserted cell and never falls when a shallower one arrives afterwards. Routing walks downward from that maximum, so an under-reported figure would make deep cells unreachable; keeping it monotonic on insert costs only a wasted probe or two and can never hide a cell that exists. |

//! Per-Sentinel outcome ledger.
//!
//! This module provides the [`SentinelLedger`] type, which stores
//! hierarchical outcome data for a single Sentinel. Each ledger
//! maintains a `HashMap` of [`LedgerKey`] → [`LedgerEntry`] mappings.
//!
//! # Structure
//!
//! The ledger is a flat `HashMap` keyed by `(lo, depth)` pairs (`LedgerKey`).
//! The root entry (depth 0, lo = 0) is always present after initialisation
//! via [`ensure_root()`](SentinelLedger::ensure_root).
//!
//! # Cross-References
//!
//! - (´tab:ledger:entry-state´) — what each stored entry holds
//! - (´dec:memory:coordinate-depth-key´) — why the map is keyed by a
//!   coordinate-and-depth pair
//! - (´def:ledger:root-semantics´) — what the root entry means

use std::collections::HashMap;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::entry::LedgerEntry;
use crate::types::LedgerKey;

// ═══════════════════════════════════════════════════════════════════════════════
// Serde Helpers
// ═══════════════════════════════════════════════════════════════════════════════

/// Serializes the entries `HashMap` as a sequence of (key, value) pairs.
///
/// JSON requires string keys, so we serialize as an array of pairs instead.
#[cfg(feature = "serde")]
#[allow(clippy::missing_docs_in_private_items)]
mod entries_serde {
    use std::collections::HashMap;

    use serde::de::{SeqAccess, Visitor};
    use serde::ser::SerializeSeq;
    use serde::{Deserializer, Serializer};

    use super::LedgerEntry;
    use crate::types::LedgerKey;

    pub fn serialize<S>(entries: &HashMap<LedgerKey, LedgerEntry>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(entries.len()))?;
        for (k, v) in entries {
            seq.serialize_element(&(*k, v))?;
        }
        seq.end()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<HashMap<LedgerKey, LedgerEntry>, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct EntriesVisitor;

        impl<'de> Visitor<'de> for EntriesVisitor {
            type Value = HashMap<LedgerKey, LedgerEntry>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a sequence of (LedgerKey, LedgerEntry) pairs")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut map = HashMap::with_capacity(seq.size_hint().unwrap_or(0));
                while let Some((k, v)) = seq.next_element::<(LedgerKey, LedgerEntry)>()? {
                    map.insert(k, v);
                }
                Ok(map)
            }
        }

        deserializer.deserialize_seq(EntriesVisitor)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SentinelLedger
// ═══════════════════════════════════════════════════════════════════════════════

/// Per-Sentinel outcome ledger.
///
/// Stores hierarchical outcome entries for a single Sentinel. Entries are
/// indexed by [`LedgerKey`] (dyadic interval).
///
/// # Invariants
///
/// - After [`ensure_root()`](Self::ensure_root), the root entry (depth 0) is always present.
/// - `entry_count` equals `entries.len()`.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SentinelLedger {
    /// Ledger entries indexed by dyadic key.
    #[cfg_attr(feature = "serde", serde(with = "entries_serde"))]
    entries: HashMap<LedgerKey, LedgerEntry>,

    /// Maximum depth of any entry (for depth-walk optimisation).
    max_depth: u8,

    /// Number of entries, maintained incrementally across the events that
    /// alter the set (´sec:ledger:entry-lifecycle´).
    entry_count: usize,
}

impl SentinelLedger {
    /// Creates an empty ledger.
    ///
    /// Call [`ensure_root()`](Self::ensure_root) to add the root entry.
    #[must_use]
    pub fn new() -> Self {
        // TODO ´todo:code:make-construction-install-the-permanent-root´: Make construction install the permanent root
        // entry directly, then retire call-site `ensure_root()` calls — the
        // root is permanent, so no construction should be able to omit it
        // (´dec:memory:root-permanence´).
        Self {
            entries: HashMap::new(),
            max_depth: 0,
            entry_count: 0,
        }
    }

    /// Ensures the root entry exists.
    ///
    /// Creates a neutral root entry at depth 0 if not present.
    /// The root entry covers the entire coordinate space.
    pub fn ensure_root(&mut self) {
        let root_key = LedgerKey::new(0, 0);
        self.entries.entry(root_key).or_insert_with(LedgerEntry::new_neutral);
        self.entry_count = self.entries.len();
    }

    /// Returns the number of entries in the ledger.
    #[must_use]
    pub const fn entry_count(&self) -> usize {
        self.entry_count
    }

    /// Returns the maximum depth of any entry.
    #[must_use]
    pub const fn max_depth(&self) -> u8 {
        self.max_depth
    }

    /// Returns a reference to the entries map.
    #[must_use]
    pub const fn entries(&self) -> &HashMap<LedgerKey, LedgerEntry> {
        &self.entries
    }

    /// Returns a mutable reference to the entries map.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // Cannot be const due to &mut return
    pub fn entries_mut(&mut self) -> &mut HashMap<LedgerKey, LedgerEntry> {
        &mut self.entries
    }

    /// Returns a reference to the entry at the given key, if present.
    #[must_use]
    pub fn get(&self, key: &LedgerKey) -> Option<&LedgerEntry> {
        self.entries.get(key)
    }

    /// Returns a mutable reference to the entry at the given key, if present.
    #[must_use]
    pub fn get_mut(&mut self, key: &LedgerKey) -> Option<&mut LedgerEntry> {
        self.entries.get_mut(key)
    }

    /// Inserts an entry at the given key.
    ///
    /// Returns the previous entry if present.
    pub fn insert(&mut self, key: LedgerKey, entry: LedgerEntry) -> Option<LedgerEntry> {
        let result = self.entries.insert(key, entry);
        self.entry_count = self.entries.len();
        if key.depth > self.max_depth {
            self.max_depth = key.depth;
        }
        result
    }

    /// Removes the entry at the given key.
    ///
    /// Returns the removed entry if present.
    pub fn remove(&mut self, key: &LedgerKey) -> Option<LedgerEntry> {
        let result = self.entries.remove(key);
        self.entry_count = self.entries.len();
        // Note: max_depth is not decremented on removal (acceptable overshoot)
        result
    }

    /// Returns `true` if the root entry exists.
    #[must_use]
    pub fn has_root(&self) -> bool {
        self.entries.contains_key(&LedgerKey::new(0, 0))
    }

    /// Returns a reference to the root entry.
    ///
    /// # Panics
    ///
    /// Panics if the root entry does not exist. Call [`ensure_root()`](Self::ensure_root) first.
    #[must_use]
    pub fn root(&self) -> &LedgerEntry {
        self.entries.get(&LedgerKey::new(0, 0)).expect("root entry must exist")
    }

    /// Returns a mutable reference to the root entry.
    ///
    /// # Panics
    ///
    /// Panics if the root entry does not exist.
    #[must_use]
    pub fn root_mut(&mut self) -> &mut LedgerEntry {
        self.entries.get_mut(&LedgerKey::new(0, 0)).expect("root entry must exist")
    }

    /// Recalculates the maximum depth from the entries.
    ///
    /// Called after bulk deletions where `max_depth` may have overshot.
    pub fn recalculate_max_depth(&mut self) {
        self.max_depth = self.entries.keys().map(|k| k.depth).max().unwrap_or(0);
    }
}

impl Default for SentinelLedger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Construction alone gives a Sentinel no cells at all — not even the root
    /// that covers the whole coordinate space. The root is installed by an
    /// explicit step, so a ledger handed out before that step is observably
    /// uninitialised rather than quietly pretending to be a blank slate that
    /// reads would route into.
    ///
    /// ´claim:ledger:a-newly-constructed-ledger-has-no-entries-not-even-a-root´
    /// ´test:unit:new-ledger-is-empty´
    #[test]
    fn new_ledger_is_empty() {
        let ledger = SentinelLedger::new();
        assert_eq!(ledger.entry_count(), 0);
        assert!(!ledger.has_root());
    }

    /// Ensuring the root installs the depth-zero cell that covers the entire
    /// coordinate space, taking the ledger to exactly one entry. Every read
    /// path terminates at that cell, so its existence is what makes routing
    /// total: no coordinate can fail to find somewhere to land.
    ///
    /// ´claim:ledger:ensuring-the-root-installs-the-cell-that-covers-the-whole-space´
    /// ´test:unit:ensure-root-creates-root´
    #[test]
    fn ensure_root_creates_root() {
        let mut ledger = SentinelLedger::new();
        ledger.ensure_root();
        assert!(ledger.has_root());
        assert_eq!(ledger.entry_count(), 1);
    }

    /// Asking for the root a second time is a no-op: the count stays at one and
    /// the existing root is left alone. Because call sites ensure the root
    /// defensively rather than tracking whether someone else already did, an
    /// idempotent step is what stops a re-entrant lifecycle from wiping a
    /// Sentinel's accumulated root history.
    ///
    /// ´claim:ledger:ensuring-the-root-twice-does-not-produce-a-second-root´
    /// ´test:unit:ensure-root-idempotent´
    #[test]
    fn ensure_root_idempotent() {
        let mut ledger = SentinelLedger::new();
        ledger.ensure_root();
        ledger.ensure_root();
        assert_eq!(ledger.entry_count(), 1);
    }

    /// The recorded maximum depth rises to meet each newly inserted cell and
    /// never falls when a shallower one arrives afterwards. Routing walks
    /// downward from that maximum, so an under-reported figure would make deep
    /// cells unreachable; keeping it monotonic on insert costs only a wasted
    /// probe or two and can never hide a cell that exists.
    ///
    /// ´claim:ledger:the-recorded-maximum-depth-rises-to-the-deepest-cell-and-never-falls-on-insert´
    /// ´test:unit:insert-updates-max-depth´
    #[test]
    fn insert_updates_max_depth() {
        let mut ledger = SentinelLedger::new();
        ledger.ensure_root();
        assert_eq!(ledger.max_depth(), 0);

        ledger.insert(LedgerKey::new(0, 8), LedgerEntry::new_neutral());
        assert_eq!(ledger.max_depth(), 8);

        ledger.insert(LedgerKey::new(0, 16), LedgerEntry::new_neutral());
        assert_eq!(ledger.max_depth(), 16);

        // Inserting at lower depth doesn't decrease max
        ledger.insert(LedgerKey::new(0, 4), LedgerEntry::new_neutral());
        assert_eq!(ledger.max_depth(), 16);
    }
}
