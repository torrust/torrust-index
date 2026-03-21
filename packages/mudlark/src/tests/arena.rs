// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Crate-level tests for [`Arena`](crate::arena::Arena).
//!
//! The arena is a slot-based allocator with generational indices and
//! free-list reuse.  These tests exercise the full lifecycle —
//! allocation, mutation, deallocation, slot recycling — plus edge
//! cases such as bitset growth past word boundaries and stale-handle
//! detection in debug builds.
//!
//! # Test index
//!
//! ## Allocation & access
//!
//! | Test | Focus |
//! |------|-------|
//! | [`alloc_and_get`] | basic alloc + read-back |
//! | [`get_mut_updates_value`] | in-place mutation via `get_mut` |
//!
//! ## Deallocation & free-list reuse
//!
//! | Test | Focus |
//! |------|-------|
//! | [`dealloc_and_reuse`] | free-list recycles a single slot |
//! | [`multiple_alloc_dealloc_cycles`] | 100 slots, free half, realloc into freed indices |
//!
//! ## Bitset internals
//!
//! | Test | Focus |
//! |------|-------|
//! | [`bitset_grows_past_word_boundary`] | occupancy bitmap extends beyond 64 slots |
//! | [`is_occupied_out_of_range_returns_false`] | out-of-range index returns `false`, not panic |
//!
//! ## Iteration
//!
//! | Test | Focus |
//! |------|-------|
//! | [`iter_occupied_returns_live_entries`] | skips deallocated slots, yields live ones |
//! | [`iter_occupied_on_empty_arena`] | empty arena yields nothing |
//!
//! ## Trait impls (`Default`, `Clone`, `Debug`)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`default_creates_empty_arena`] | `Default` produces count-0 arena |
//! | [`clone_is_independent`] | clone is a deep copy, mutations are independent |
//! | [`debug_format_is_sensible`] | `Debug` output contains `"Arena"` and count |
//!
//! ## Stale-handle panics (debug only)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`get_stale_handle_panics_in_debug`] | `get` on freed slot panics |
//! | [`get_mut_stale_handle_panics_in_debug`] | `get_mut` on freed slot panics |
//! | [`double_dealloc_panics_in_debug`] | double-free panics |

use crate::arena::Arena;

// ── Allocation & access ─────────────────────────────────────────

#[test]
fn alloc_and_get() {
    let mut arena = Arena::<u64>::new();
    let i = arena.alloc(42);
    assert_eq!(*arena.get(i), 42);
    assert_eq!(arena.count(), 1);
}

#[test]
fn get_mut_updates_value() {
    let mut arena = Arena::<u64>::new();
    let i = arena.alloc(10);
    *arena.get_mut(i) = 99;
    assert_eq!(*arena.get(i), 99);
}

// ── Deallocation & free-list reuse ──────────────────────────────

#[test]
fn dealloc_and_reuse() {
    let mut arena = Arena::<u64>::new();
    let a = arena.alloc(10);
    let b = arena.alloc(20);
    assert_eq!(arena.count(), 2);

    let val = arena.dealloc(a);
    assert_eq!(val, 10);
    assert_eq!(arena.count(), 1);
    assert!(!arena.is_occupied(a));
    assert!(arena.is_occupied(b));

    // Realloc reuses the freed slot.
    let c = arena.alloc(30);
    assert_eq!(c, a, "freed slot should be reused");
    assert_eq!(*arena.get(c), 30);
    assert_eq!(arena.count(), 2);
}

#[test]
fn multiple_alloc_dealloc_cycles() {
    let mut arena = Arena::<u32>::new();
    let mut indices = Vec::new();
    for i in 0..100 {
        indices.push(arena.alloc(i));
    }
    assert_eq!(arena.count(), 100);

    // Free every other slot.
    for &i in indices.iter().step_by(2) {
        arena.dealloc(i);
    }
    assert_eq!(arena.count(), 50);

    // Reallocate — should reuse freed slots.
    for v in 200..250 {
        let idx = arena.alloc(v);
        assert!(idx < 100, "should reuse existing slots");
    }
    assert_eq!(arena.count(), 100);
}

// ── Bitset internals ────────────────────────────────────────────

#[test]
fn bitset_grows_past_word_boundary() {
    let mut arena = Arena::<u8>::new();
    // Allocate past the first bitset word boundary (64 slots).
    for i in 0..65 {
        arena.alloc(u8::try_from(i).unwrap());
    }
    assert_eq!(arena.count(), 65);
    assert!(arena.is_occupied(64));
}

#[test]
fn is_occupied_out_of_range_returns_false() {
    let arena = Arena::<u64>::new();
    assert!(!arena.is_occupied(0));
    assert!(!arena.is_occupied(999));
}

// ── Iteration ───────────────────────────────────────────────────

#[test]
fn iter_occupied_returns_live_entries() {
    let mut arena = Arena::<u64>::new();
    let a = arena.alloc(10);
    let b = arena.alloc(20);
    let c = arena.alloc(30);
    arena.dealloc(b);

    let entries: Vec<(usize, &u64)> = arena.iter_occupied().collect();
    assert_eq!(entries.len(), 2);
    assert!(entries.contains(&(a, &10)));
    assert!(entries.contains(&(c, &30)));
}

#[test]
fn iter_occupied_on_empty_arena() {
    let arena = Arena::<u64>::new();
    assert_eq!(arena.iter_occupied().count(), 0);
}

// ── Trait impls (`Default`, `Clone`, `Debug`) ───────────────────

#[test]
fn default_creates_empty_arena() {
    let arena = Arena::<u64>::default();
    assert_eq!(arena.count(), 0);
    assert_eq!(arena.iter_occupied().count(), 0);
}

#[test]
fn clone_is_independent() {
    let mut arena = Arena::<u64>::new();
    arena.alloc(1);
    arena.alloc(2);

    let mut cloned = arena.clone();
    let idx = cloned.alloc(3);

    assert_eq!(arena.count(), 2);
    assert_eq!(cloned.count(), 3);
    assert_eq!(*cloned.get(idx), 3);
}

#[test]
fn debug_format_is_sensible() {
    let mut arena = Arena::<u64>::new();
    arena.alloc(42);
    let dbg = format!("{arena:?}");
    assert!(dbg.contains("Arena"));
    assert!(dbg.contains("count: 1"));
}

// ── Stale-handle panics (debug only) ────────────────────────────

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "not occupied")]
fn get_stale_handle_panics_in_debug() {
    let mut arena = Arena::<u64>::new();
    let i = arena.alloc(1);
    arena.dealloc(i);
    let _ = arena.get(i);
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "not occupied")]
fn get_mut_stale_handle_panics_in_debug() {
    let mut arena = Arena::<u64>::new();
    let i = arena.alloc(1);
    arena.dealloc(i);
    let _ = arena.get_mut(i);
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "not occupied")]
fn double_dealloc_panics_in_debug() {
    let mut arena = Arena::<u64>::new();
    let i = arena.alloc(1);
    arena.dealloc(i);
    arena.dealloc(i);
}
