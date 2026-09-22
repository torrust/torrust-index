// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Low-level fixture builders for `DimensionMap` / interaction tests.
//!
//! These helpers construct the raw `IndexMap` / `Vec` shapes that
//! `DimensionMap::rebuild(_no_interactions)` expects. They are used
//! by crate tests that exercise the feature-layout machinery directly
//! — one layer below the [`World`](super::World) harness, which
//! drives the full public `Assayer` API.

use indexmap::IndexMap;

use crate::identity::CompetitiveCellId;
use crate::types::{DimensionId, OutcomeAxisId, SentinelId};

/// Creates an `IndexMap<SentinelId, ()>` from a slice of `u32` IDs.
#[must_use]
pub fn sentinels(ids: &[u32]) -> IndexMap<SentinelId, ()> {
    ids.iter().map(|&id| (SentinelId(id), ())).collect()
}

/// Creates identity dimension IDs from integer IDs.
#[must_use]
pub fn dims(ids: &[u32]) -> Vec<DimensionId> {
    ids.iter().map(|&id| DimensionId(id)).collect()
}

/// Creates competitive cells for a set of dimensions.
#[must_use]
pub fn cells(entries: &[(u32, &[CompetitiveCellId])]) -> IndexMap<DimensionId, Vec<CompetitiveCellId>> {
    entries.iter().map(|&(id, cs)| (DimensionId(id), cs.to_vec())).collect()
}

/// Creates spatial outcome axis IDs from integer IDs, in the given order.
///
/// The order is the layout's: a Sentinel slot's outcome-memory pairs are laid
/// out in the order the axes arrive, so a fixture that names them names their
/// positions too.
#[must_use]
pub fn axes(ids: &[u32]) -> Vec<OutcomeAxisId> {
    ids.iter().map(|&id| OutcomeAxisId(id)).collect()
}

/// Creates `n` spatial outcome axis IDs, numbered from one.
///
/// For fixtures that care only how many pairs the slot tail carries and not
/// which axis owns each. Where the identities are the subject, name them with
/// [`axes`] instead.
#[must_use]
pub fn n_axes(n: usize) -> Vec<OutcomeAxisId> {
    (1..=n).map(|i| OutcomeAxisId(u32::try_from(i).unwrap_or(u32::MAX))).collect()
}

/// Creates `n` distinct competitive cells for a dimension.
#[must_use]
pub fn make_cells(n: usize) -> Vec<CompetitiveCellId> {
    (0..n)
        .map(|i| CompetitiveCellId::new(u128::try_from(i * 256).unwrap(), 8))
        .collect()
}
