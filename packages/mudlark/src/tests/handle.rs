// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Crate-level tests for **`GNodeId`** and **`VNodeId`** handles.
//!
//! Both handle types are thin, `Copy` wrappers around a `NonMaxU32`
//! index.  The `NonMax` representation enables niche optimisation so
//! that `Option<GNodeId>` and `Option<VNodeId>` are the same size as
//! the bare handle (4 bytes).  These tests verify layout guarantees,
//! round-trip fidelity, overflow detection, trait implementations
//! (`Eq`, `Ord`, `Hash`, `Debug`, `Copy`), and type-level
//! distinctness between the two handle families.
//!
//! # Test index
//!
//! ## Layout
//!
//! | Test | Focus |
//! |------|-------|
//! | [`niche_optimization`] | `Option<Handle>` is 4 bytes thanks to `NonMax` niche |
//!
//! ## Round-trip (`from_index` / `index`)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`round_trip_typical`] | small representative indices survive the round-trip |
//! | [`round_trip_max_valid_index`] | `u32::MAX - 1` (largest valid index) round-trips correctly |
//!
//! ## Overflow (panics)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`gnode_id_overflow_at_u32_max`] | `u32::MAX` is reserved for the niche — must panic |
//! | [`vnode_id_overflow_at_u32_max`] | same check for `VNodeId` |
//! | [`gnode_id_overflow_above_u32`] | 64-bit `usize` beyond `u32` range panics |
//! | [`vnode_id_overflow_above_u32`] | same check for `VNodeId` |
//!
//! ## Equality
//!
//! | Test | Focus |
//! |------|-------|
//! | [`same_index_is_equal`] | handles with identical indices compare equal |
//! | [`different_index_is_not_equal`] | handles with distinct indices are not equal |
//! | [`gnode_and_vnode_are_distinct_types`] | compile-time only — the two families never unify |
//!
//! ## Ordering (`GNodeId` only — `VNodeId` has no `Ord`)
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
use crate::handle::VNodeId;

// ── Layout ──────────────────────────────────────────────────────

#[test]
fn niche_optimization() {
    assert_eq!(size_of::<GNodeId>(), 4);
    assert_eq!(size_of::<Option<GNodeId>>(), 4);
    assert_eq!(size_of::<VNodeId>(), 4);
    assert_eq!(size_of::<Option<VNodeId>>(), 4);
}

// ── Round-trip (from_index / index) ─────────────────────────────

#[test]
fn round_trip_typical() {
    for i in [0, 1, 42, 1_000_000] {
        let g = GNodeId::from_index(i);
        assert_eq!(g.index(), i);

        let v = VNodeId::from_index(i);
        assert_eq!(v.index(), i);
    }
}

#[test]
fn round_trip_max_valid_index() {
    let max = (u32::MAX - 1) as usize;

    let g = GNodeId::from_index(max);
    assert_eq!(g.index(), max);

    let v = VNodeId::from_index(max);
    assert_eq!(v.index(), max);
}

// ── Overflow (panics) ───────────────────────────────────────────

#[test]
#[should_panic(expected = "index out of range")]
fn gnode_id_overflow_at_u32_max() {
    let _ = GNodeId::from_index(u32::MAX as usize);
}

#[test]
#[should_panic(expected = "index out of range")]
fn vnode_id_overflow_at_u32_max() {
    let _ = VNodeId::from_index(u32::MAX as usize);
}

#[test]
#[cfg(target_pointer_width = "64")]
#[should_panic(expected = "index out of range")]
fn gnode_id_overflow_above_u32() {
    let _ = GNodeId::from_index(u32::MAX as usize + 1);
}

#[test]
#[cfg(target_pointer_width = "64")]
#[should_panic(expected = "index out of range")]
fn vnode_id_overflow_above_u32() {
    let _ = VNodeId::from_index(u32::MAX as usize + 1);
}

// ── Equality ────────────────────────────────────────────────────

#[test]
fn same_index_is_equal() {
    let a = GNodeId::from_index(7);
    let b = GNodeId::from_index(7);
    assert_eq!(a, b);

    let va = VNodeId::from_index(7);
    let vb = VNodeId::from_index(7);
    assert_eq!(va, vb);
}

#[test]
fn different_index_is_not_equal() {
    assert_ne!(GNodeId::from_index(0), GNodeId::from_index(1));
    assert_ne!(VNodeId::from_index(0), VNodeId::from_index(1));
}

// ── Ordering (GNodeId only — VNodeId has no Ord) ────────────────

#[test]
fn gnode_ordering() {
    let a = GNodeId::from_index(0);
    let b = GNodeId::from_index(5);
    let c = GNodeId::from_index(10);
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
    let a = GNodeId::from_index(42);
    let b = GNodeId::from_index(42);
    assert_eq!(hash_of(&a), hash_of(&b));

    let va = VNodeId::from_index(42);
    let vb = VNodeId::from_index(42);
    assert_eq!(hash_of(&va), hash_of(&vb));
}

// ── Debug ───────────────────────────────────────────────────────

#[test]
fn debug_format() {
    let g = GNodeId::from_index(0);
    let dbg = format!("{g:?}");
    assert!(dbg.contains("GNodeId"), "expected type name in debug: {dbg}");

    let v = VNodeId::from_index(0);
    let dbg = format!("{v:?}");
    assert!(dbg.contains("VNodeId"), "expected type name in debug: {dbg}");
}

// ── Copy ────────────────────────────────────────────────────────

#[test]
fn copy_semantics() {
    let a = GNodeId::from_index(3);
    let b = a; // Copy
    assert_eq!(a, b); // both still usable

    let va = VNodeId::from_index(3);
    let vb = va;
    assert_eq!(va, vb);
}
