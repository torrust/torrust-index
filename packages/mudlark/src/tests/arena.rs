// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

use crate::arena::Arena;

#[test]
fn alloc_and_get() {
    let mut arena = Arena::<u64>::new();
    let i = arena.alloc(42);
    assert_eq!(*arena.get(i), 42);
    assert_eq!(arena.count(), 1);
}

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

#[test]
fn bitset_grows_correctly() {
    let mut arena = Arena::<u8>::new();
    // Allocate past the first bitset word boundary (64 slots).
    for i in 0..65 {
        arena.alloc(u8::try_from(i).unwrap());
    }
    assert_eq!(arena.count(), 65);
    assert!(arena.is_occupied(64));
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "not occupied")]
fn get_stale_handle_panics_in_debug() {
    let mut arena = Arena::<u64>::new();
    let i = arena.alloc(1);
    arena.dealloc(i);
    let _ = arena.get(i);
}
