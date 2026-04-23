// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Crate-level tests for **`GNodeId`**, **`GSlotPointer`**, and **`VSlotPointer`** handles.
//!
//! `GNodeId` is the public generational handle (8 bytes).  `GSlotPointer`
//! and `VSlotPointer` are internal, generation-free wrappers around
//! `NonZeroU32` (4 bytes).  The `NonZero` representation enables niche
//! optimisation so that `Option<GSlotPointer>` and `Option<VSlotPointer>`
//! are the same size as the bare handle (4 bytes).  These tests verify
//! layout guarantees, round-trip fidelity, overflow detection, trait
//! implementations (`Eq`, `Ord`, `Hash`, `Debug`, `Copy`), and
//! type-level distinctness between the handle families.
//!
//! # Test index
//!
//! ## Layout
//!
//! | Test | Focus |
//! |------|-------|
//! | [`slot_niche_optimization`] | `Option<SlotPointer>` is 4 bytes thanks to `NonZero` niche |
//! | [`gnodeid_size`] | `GNodeId` is 8 bytes; `Option<GNodeId>` is 8 bytes (niche via `NonZeroU32`) |
//!
//! ## Round-trip (`from_index` / `index`)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`round_trip_typical`] | small representative indices survive the round-trip |
//! | [`round_trip_max_valid_index`] | `u32::MAX - 1` (largest valid index) round-trips correctly |
//!
//! ## `GNodeId` round-trip (`from_parts` / `index` / `generation` / `slot`)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`gnodeid_round_trip`] | `from_parts` → `index` + `generation` |
//! | [`gnodeid_slot_extracts_index`] | `slot()` returns a `GSlotPointer` with the same index |
//! | [`gnodeid_overflow_at_u32_max`] | `u32::MAX` is reserved — must panic |
//!
//! ## Overflow (panics)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`gslot_overflow_at_u32_max`] | `u32::MAX` is reserved for the niche — must panic |
//! | [`vslot_overflow_at_u32_max`] | same check for `VSlotPointer` |
//! | [`gslot_overflow_above_u32`] | 64-bit `usize` beyond `u32` range panics |
//! | [`vslot_overflow_above_u32`] | same check for `VSlotPointer` |
//!
//! ## Equality
//!
//! | Test | Focus |
//! |------|-------|
//! | [`same_index_is_equal`] | handles with identical indices compare equal |
//! | [`different_index_is_not_equal`] | handles with distinct indices are not equal |
//!
//! ## Ordering (`GSlotPointer` only — `VSlotPointer` has no `Ord`)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`gnode_ordering`] | `<` comparison and `sort()` respect index magnitude |
//!
//! ## Hash
//!
//! | Test | Focus |
//! |------|-------|
//! | [`equal_handles_same_hash`] | equal handles produce identical `DefaultHasher` digests |
//!
//! ## Debug
//!
//! | Test | Focus |
//! |------|-------|
//! | [`debug_format`] | `Debug` output contains the type name |
//!
//! ## Copy
//!
//! | Test | Focus |
//! |------|-------|
//! | [`copy_semantics`] | both original and copy remain usable after implicit `Copy` |

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::mem::size_of;

use crate::GNodeId;
use crate::handle::{GSlotPointer, VSlotPointer};

// ── Layout ──────────────────────────────────────────────────────

#[test]
fn slot_niche_optimization() {
    assert_eq!(size_of::<GSlotPointer>(), 4);
    assert_eq!(size_of::<Option<GSlotPointer>>(), 4);
    assert_eq!(size_of::<VSlotPointer>(), 4);
    assert_eq!(size_of::<Option<VSlotPointer>>(), 4);
}

#[test]
fn gnodeid_size() {
    assert_eq!(size_of::<GNodeId>(), 8);
    // NonZeroU32 provides a niche: Option<GNodeId> fits in 8 bytes.
    assert_eq!(size_of::<Option<GNodeId>>(), 8);
}

// ── Round-trip (from_index / index) ─────────────────────────────

#[test]
fn round_trip_typical() {
    for i in [0, 1, 42, 1_000_000] {
        let g = GSlotPointer::from_index(i);
        assert_eq!(g.index(), i);

        let v = VSlotPointer::from_index(i);
        assert_eq!(v.index(), i);
    }
}

#[test]
fn round_trip_max_valid_index() {
    let max = (u32::MAX - 1) as usize;

    let g = GSlotPointer::from_index(max);
    assert_eq!(g.index(), max);

    let v = VSlotPointer::from_index(max);
    assert_eq!(v.index(), max);
}

// ── GNodeId (generational handle) ───────────────────────────────

#[test]
fn gnodeid_round_trip() {
    for (idx, generation) in [(0, 0), (1, 0), (42, 7), (1_000_000, 99)] {
        let id = GNodeId::from_parts(idx, generation);
        assert_eq!(id.index(), idx);
        assert_eq!(id.generation(), generation);
    }
}

#[test]
fn gnodeid_slot_extracts_index() {
    let id = GNodeId::from_parts(42, 5);
    let slot = id.slot();
    assert_eq!(slot.index(), 42);
}

#[test]
#[should_panic(expected = "index out of range")]
fn gnodeid_overflow_at_u32_max() {
    let _ = GNodeId::from_parts(u32::MAX as usize, 0);
}

// ── Overflow (panics) ───────────────────────────────────────────

#[test]
#[should_panic(expected = "index out of range")]
fn gslot_overflow_at_u32_max() {
    let _ = GSlotPointer::from_index(u32::MAX as usize);
}

#[test]
#[should_panic(expected = "index out of range")]
fn vslot_overflow_at_u32_max() {
    let _ = VSlotPointer::from_index(u32::MAX as usize);
}

#[test]
#[cfg(target_pointer_width = "64")]
#[should_panic(expected = "index out of range")]
fn gslot_overflow_above_u32() {
    let _ = GSlotPointer::from_index(u32::MAX as usize + 1);
}

#[test]
#[cfg(target_pointer_width = "64")]
#[should_panic(expected = "index out of range")]
fn vslot_overflow_above_u32() {
    let _ = VSlotPointer::from_index(u32::MAX as usize + 1);
}

// ── Equality ────────────────────────────────────────────────────

#[test]
fn same_index_is_equal() {
    let a = GSlotPointer::from_index(7);
    let b = GSlotPointer::from_index(7);
    assert_eq!(a, b);

    let va = VSlotPointer::from_index(7);
    let vb = VSlotPointer::from_index(7);
    assert_eq!(va, vb);
}

#[test]
fn different_index_is_not_equal() {
    assert_ne!(GSlotPointer::from_index(0), GSlotPointer::from_index(1));
    assert_ne!(VSlotPointer::from_index(0), VSlotPointer::from_index(1));
}

// ── Ordering (GSlotPointer only — VSlotPointer has no Ord) ────────────────

#[test]
fn gnode_ordering() {
    let a = GSlotPointer::from_index(0);
    let b = GSlotPointer::from_index(5);
    let c = GSlotPointer::from_index(10);
    assert!(a < b);
    assert!(b < c);

    let mut v = vec![c, a, b];
    v.sort();
    assert_eq!(v, vec![a, b, c]);
}

// ── Hash ────────────────────────────────────────────────────────

fn hash_of<T: Hash>(val: &T) -> u64 {
    let mut h = DefaultHasher::new();
    val.hash(&mut h);
    h.finish()
}

#[test]
fn equal_handles_same_hash() {
    let a = GSlotPointer::from_index(42);
    let b = GSlotPointer::from_index(42);
    assert_eq!(hash_of(&a), hash_of(&b));

    let va = VSlotPointer::from_index(42);
    let vb = VSlotPointer::from_index(42);
    assert_eq!(hash_of(&va), hash_of(&vb));
}

// ── Debug ───────────────────────────────────────────────────────

#[test]
fn debug_format() {
    let g = GSlotPointer::from_index(0);
    let dbg = format!("{g:?}");
    assert!(dbg.contains("GSlotPointer"), "expected type name in debug: {dbg}");

    let v = VSlotPointer::from_index(0);
    let dbg = format!("{v:?}");
    assert!(dbg.contains("VSlotPointer"), "expected type name in debug: {dbg}");
}

// ── Copy ────────────────────────────────────────────────────────

#[test]
fn copy_semantics() {
    let a = GSlotPointer::from_index(3);
    let b = a; // Copy
    assert_eq!(a, b); // both still usable

    let va = VSlotPointer::from_index(3);
    let vb = va;
    assert_eq!(va, vb);
}
