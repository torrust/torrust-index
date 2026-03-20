// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

use std::mem::size_of;

use crate::GNodeId;
use crate::handle::VNodeId;

#[test]
fn handle_round_trip() {
    for i in [0, 1, 42, 1_000_000] {
        let g = GNodeId::from_index(i);
        assert_eq!(g.index(), i);

        let v = VNodeId::from_index(i);
        assert_eq!(v.index(), i);
    }
}

#[test]
fn niche_optimization() {
    assert_eq!(size_of::<GNodeId>(), 4);
    assert_eq!(size_of::<Option<GNodeId>>(), 4);
    assert_eq!(size_of::<VNodeId>(), 4);
    assert_eq!(size_of::<Option<VNodeId>>(), 4);
}

#[test]
#[should_panic(expected = "index out of range")]
fn gnode_id_overflow() {
    let _ = GNodeId::from_index(u32::MAX as usize);
}

#[test]
#[should_panic(expected = "index out of range")]
fn vnode_id_overflow() {
    let _ = VNodeId::from_index(u32::MAX as usize);
}
