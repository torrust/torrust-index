// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Vec-backed arena with free-list reuse and bitset occupancy tracking.
//!
//! Provides O(1) allocation, deallocation, and indexed access.
//! `debug_assert!` guards catch stale-handle bugs in development;
//! compiles to bare array access in release.

use std::fmt;

/// A dense, reusable arena for cache-line-sized node types.
///
/// Slots are reused via a free list. A separate `Vec<u64>` bitset
/// tracks occupancy — cold data, never touched on the hot read/write
/// path.
pub struct Arena<T: Default> {
    slots: Vec<T>,
    /// Bitset: bit `i` is set iff slot `i` is occupied.
    occupied: Vec<u64>,
    /// Stack of freed 0-based indices available for reuse.
    free: Vec<u32>,
    /// Number of live entries.
    count: u32,
}

impl<T: Default> Arena<T> {
    /// Create an empty arena.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            slots: Vec::new(),
            occupied: Vec::new(),
            free: Vec::new(),
            count: 0,
        }
    }

    /// Number of live (occupied) entries.
    #[must_use]
    #[inline]
    pub const fn count(&self) -> u32 {
        self.count
    }

    /// Allocate a slot, returning its 0-based index.
    ///
    /// Reuses a previously freed slot if available, otherwise grows
    /// the backing `Vec`.
    pub fn alloc(&mut self, value: T) -> usize {
        let index = if let Some(idx) = self.free.pop() {
            let i = idx as usize;
            self.slots[i] = value;
            i
        } else {
            let i = self.slots.len();
            self.slots.push(value);
            // Grow bitset if needed.
            let word = i / 64;
            if word >= self.occupied.len() {
                self.occupied.resize(word + 1, 0);
            }
            i
        };
        self.set_occupied(index, true);
        self.count += 1;
        index
    }

    /// Deallocate the slot at `index`, returning the stored value
    /// and replacing it with `T::default()`.
    ///
    /// # Panics (debug only)
    ///
    /// `debug_assert`s that the slot is currently occupied.
    pub fn dealloc(&mut self, index: usize) -> T {
        debug_assert!(self.is_occupied(index), "Arena::dealloc: slot {index} is not occupied");
        self.set_occupied(index, false);
        #[allow(clippy::cast_possible_truncation)] // arena indices are always < u32::MAX
        self.free.push(index as u32);
        self.count -= 1;
        std::mem::take(&mut self.slots[index])
    }

    /// Immutable access to the slot at `index`.
    ///
    /// # Panics (debug only)
    ///
    /// `debug_assert`s that the slot is currently occupied.
    #[must_use]
    #[inline]
    pub fn get(&self, index: usize) -> &T {
        debug_assert!(self.is_occupied(index), "Arena::get: slot {index} is not occupied");
        &self.slots[index]
    }

    /// Mutable access to the slot at `index`.
    ///
    /// # Panics (debug only)
    ///
    /// `debug_assert`s that the slot is currently occupied.
    #[inline]
    pub fn get_mut(&mut self, index: usize) -> &mut T {
        debug_assert!(self.is_occupied(index), "Arena::get_mut: slot {index} is not occupied");
        &mut self.slots[index]
    }

    /// Whether the slot at `index` is currently occupied.
    #[must_use]
    #[inline]
    pub fn is_occupied(&self, index: usize) -> bool {
        let word = index / 64;
        let bit = index % 64;
        word < self.occupied.len() && (self.occupied[word] & (1u64 << bit)) != 0
    }

    /// Iterate over all occupied `(index, &T)` pairs.
    ///
    /// Useful for invariant checking and diagnostics. The iteration
    /// order is by slot index (ascending), not insertion order.
    pub fn iter_occupied(&self) -> impl Iterator<Item = (usize, &T)> + '_ {
        (0..self.slots.len())
            .filter(move |&i| self.is_occupied(i))
            .map(move |i| (i, &self.slots[i]))
    }

    // ── internal helpers ──────────────────────────────────────────

    fn set_occupied(&mut self, index: usize, value: bool) {
        let word = index / 64;
        let bit = index % 64;
        if value {
            self.occupied[word] |= 1u64 << bit;
        } else {
            self.occupied[word] &= !(1u64 << bit);
        }
    }
}

impl<T: Default> Default for Arena<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Default + fmt::Debug> fmt::Debug for Arena<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Arena")
            .field("count", &self.count)
            .field("capacity", &self.slots.len())
            .field("free_list_len", &self.free.len())
            .finish_non_exhaustive()
    }
}

impl<T: Default + Clone> Clone for Arena<T> {
    fn clone(&self) -> Self {
        Self {
            slots: self.slots.clone(),
            occupied: self.occupied.clone(),
            free: self.free.clone(),
            count: self.count,
        }
    }
}

#[cfg(test)]
mod tests {}
