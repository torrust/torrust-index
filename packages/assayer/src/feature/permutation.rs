// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`keys_enumerate_every_position_exactly_once`] | feature | Every physical position of the vector carries exactly one key and no key is issued twice, so the enumeration is a bijection between the map's blocks and the coordinates a model actually holds. Without that the permutation built from two enumerations could silently drop a learned weight instead of moving it. |
//! | [`identity_after_sentinel_permutes`] | feature | Registering a Sentinel and then an identity dimension — the ordering the audit reproduced the defect on — yields a permutation that is not the identity. The append leaves the Sentinel's learned coordinates sitting where the rebuild has just placed the identity blocks, and the permutation is what carries them back to the positions the map now names. |
//! | [`canonical_order_permutation_is_identity`] | feature | Registrations that arrive in canonical block order need no rearrangement: the permutation is the identity map. The repair costs nothing on the ordering that was already correct, which is why every test that pinned block positions by registering the identity dimension first went on passing. |
//! | [`permutation_carries_every_old_position`] | feature | Every position the pre-event map named appears exactly once as a source in the permutation, and the positions with no source are exactly the coordinates the extension appended. A learned weight can therefore be moved but never dropped, and a fresh coordinate never overwrites one. |
//! | [`axis_registration_permutes_within_blocks`] | feature | Registering an outcome axis widens the identity blocks and the Sentinel slots at once, so the appended coordinates belong inside blocks rather than after them. The permutation opens a gap in each widened block and slides every later coordinate along, which a block-granular rearrangement could not express. |

//! Canonical position keys and the extension-to-rebuild permutation.
//!
//! The allocation rule appends at the end (´alg:dimension:compaction´) while
//! the rebuild lays the vector out in canonical block order
//! (´def:feature:vector-structure´). The two agree only when registrations
//! happen to arrive in canonical order. Version consistency
//! (´inv:dimension:version-consistency´) is unconditional, so the rebuild
//! rearranges the parameters and the standardisation vectors into the order it
//! has just computed — the settled repair (´rem:assayer:feature-extension-order´).
//!
//! # Cross-References
//!
//! - (´def:feature:vector-structure´) — the canonical block order
//! - (´alg:dimension:compaction´) — the allocation rule and the compaction
//!   this rearrangement generalises

use std::collections::HashMap;

use crate::feature::dimension_map::{
    DimensionMap, ID_AXIS_FEATURES_PER_AXIS, ID_CROSS_DIM_FEATURE_COUNT, ID_DIM_BASE_FEATURE_COUNT, SENTINEL_OCCUPANCY_WIDTH,
    SENTINEL_Q_BASE, SENTINEL_SPATIAL_FEATURES_PER_AXIS,
};
use crate::feature::interaction::InteractionId;
use crate::identity::CompetitiveCellId;
use crate::types::{DimensionId, OutcomeAxisId, SentinelId};

// ═══════════════════════════════════════════════════════════════════════════════
// Constants
// ═══════════════════════════════════════════════════════════════════════════════

/// Width of the batch context feature that closes every Sentinel extraction.
///
/// The extraction is six groups in a fixed order and the batch context is the
/// sixth, standing after the per-axis outcome-memory pairs
/// (´def:extraction:slot´).
///
/// ´const:assayer:batch-context-width´ (´alg:const:count´)
/// ´const:assayer:batch-context-width-count-1´
pub const SENTINEL_BATCH_CONTEXT_WIDTH: usize = 1;

/// Offset of the first per-axis outcome-memory pair inside a Sentinel slot.
///
/// The slot opens with the occupancy indicator, then the extraction's four
/// fixed groups — fifty-six positions — and the outcome memory's three
/// whole-entry quantities; the per-axis pairs begin after them and the batch
/// context closes the slot (´def:dimension:layers´). One plus fifty-nine is
/// sixty, and the figure is invariant under every lifecycle event.
///
/// ´const:assayer:slot-outcome-memory-pair-offset´ (´alg:const:count´)
/// ´const:assayer:slot-outcome-memory-pair-offset-count-60´
pub const SLOT_AXIS_PAIR_OFFSET: usize = 60;

/// Offset of the first per-axis outcome-memory pair inside a Sentinel's
/// stored extraction.
///
/// The slot's own offset less the occupancy indicator that opens it: the
/// extraction is what a Sentinel reports, and it carries no occupancy flag
/// (´def:extraction:slot´).
///
/// ´const:assayer:extraction-outcome-memory-pair-offset´ (´alg:const:count´)
/// ´const:assayer:extraction-outcome-memory-pair-offset-count-59´
pub const EXTRACTION_AXIS_PAIR_OFFSET: usize = 59;

// ═══════════════════════════════════════════════════════════════════════════════
// Position keys
// ═══════════════════════════════════════════════════════════════════════════════

/// The semantic identity of one physical position of the feature vector.
///
/// Two maps that name the same coordinate differently — because a rebuild has
/// reordered the blocks between them — issue the same key for it. That is what
/// makes a permutation computable from a pair of maps rather than from the
/// event that separated them.
///
/// The granularity is the finest at which a position's meaning is stable under
/// an extension: a Sentinel's outcome-memory pair is keyed by its spatial index
/// rather than by its offset, because registering an axis widens the slot and
/// moves every later offset along.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PositionKey {
    /// The bias term.
    Bias,
    /// One of the fifteen batch aggregates, by offset in the block.
    Aggregate(usize),
    /// One of a dimension's eight base identity features, by offset.
    IdentityBase(DimensionId, usize),
    /// One of a dimension's per-axis identity triple, by axis and offset.
    IdentityAxis(DimensionId, OutcomeAxisId, usize),
    /// One of the eight cross-dimension identity aggregates, by offset.
    IdentityCross(usize),
    /// One declared signal feature, by offset in the signal block.
    Signal(usize),
    /// A Sentinel's occupancy indicator.
    SentinelOccupancy(SentinelId),
    /// A Sentinel extraction position ahead of the per-axis pairs, by offset.
    SentinelFixed(SentinelId, usize),
    /// Half of a Sentinel's outcome-memory pair, by spatial axis index.
    SentinelAxisPair(SentinelId, usize, usize),
    /// A Sentinel's batch context feature, which closes the slot.
    SentinelBatchContext(SentinelId),
    /// One expanded interaction feature.
    Interaction(InteractionId),
    /// One competitive cell indicator.
    Competitive(DimensionId, CompetitiveCellId),
}

impl DimensionMap {
    /// Enumerates the semantic key of every physical position, in index order.
    ///
    /// The returned vector has length `p` and is a bijection onto the map's
    /// blocks: every coordinate a full-dimension model holds is named exactly
    /// once (´claim:feature:keys-enumerate-every-position-exactly-once´).
    ///
    /// # Panics
    ///
    /// Debug-asserts that every position received a key.
    #[must_use]
    pub fn position_keys(&self) -> Vec<PositionKey> {
        let mut keys: Vec<Option<PositionKey>> = vec![None; self.p];
        let mut put = |idx: usize, key: PositionKey| {
            if let Some(slot) = keys.get_mut(idx) {
                debug_assert!(slot.is_none(), "position {idx} keyed twice");
                *slot = Some(key);
            }
        };

        put(self.bias_idx, PositionKey::Bias);

        for (offset, idx) in (self.agg_range.start..self.agg_range.end).enumerate() {
            put(idx, PositionKey::Aggregate(offset));
        }

        for (&dim_id, range) in &self.id_dim_ranges {
            for offset in 0..ID_DIM_BASE_FEATURE_COUNT.min(range.end - range.start) {
                put(range.start + offset, PositionKey::IdentityBase(dim_id, offset));
            }
            for (&axis_id, &axis_offset) in &self.id_axis_offsets {
                for within in 0..ID_AXIS_FEATURES_PER_AXIS {
                    let idx = range.start + axis_offset + within;
                    if idx < range.end {
                        put(idx, PositionKey::IdentityAxis(dim_id, axis_id, within));
                    }
                }
            }
        }

        if let Some(ref cross) = self.id_cross_dim_range {
            for offset in 0..ID_CROSS_DIM_FEATURE_COUNT.min(cross.end - cross.start) {
                put(cross.start + offset, PositionKey::IdentityCross(offset));
            }
        }

        for (offset, idx) in (self.sig_range.start..self.sig_range.end).enumerate() {
            put(idx, PositionKey::Signal(offset));
        }

        let spatial_axis_count = self.spatial_axis_count();
        for (&sentinel_id, range) in &self.sentinel_slots {
            put(range.start, PositionKey::SentinelOccupancy(sentinel_id));
            for offset in SENTINEL_OCCUPANCY_WIDTH..SLOT_AXIS_PAIR_OFFSET {
                put(range.start + offset, PositionKey::SentinelFixed(sentinel_id, offset));
            }
            for axis_index in 0..spatial_axis_count {
                for half in 0..SENTINEL_SPATIAL_FEATURES_PER_AXIS {
                    let offset = SLOT_AXIS_PAIR_OFFSET + axis_index * SENTINEL_SPATIAL_FEATURES_PER_AXIS + half;
                    put(
                        range.start + offset,
                        PositionKey::SentinelAxisPair(sentinel_id, axis_index, half),
                    );
                }
            }
            let batch_offset = SLOT_AXIS_PAIR_OFFSET + spatial_axis_count * SENTINEL_SPATIAL_FEATURES_PER_AXIS;
            for within in 0..SENTINEL_BATCH_CONTEXT_WIDTH {
                put(
                    range.start + batch_offset + within,
                    PositionKey::SentinelBatchContext(sentinel_id),
                );
            }
        }

        for (&int_id, &idx) in &self.interaction_indices {
            put(idx, PositionKey::Interaction(int_id));
        }

        for (&dim_id, cells) in &self.competitive_indices {
            for (&cell_id, &idx) in cells {
                put(idx, PositionKey::Competitive(dim_id, cell_id));
            }
        }

        debug_assert!(
            keys.iter().all(Option::is_some),
            "every position of a rebuilt map must carry a key",
        );
        keys.into_iter()
            .enumerate()
            .map(|(idx, key)| key.unwrap_or(PositionKey::Signal(idx)))
            .collect()
    }

    /// Returns the spatial outcome axis count the slot width implies.
    #[must_use]
    const fn spatial_axis_count(&self) -> usize {
        self.sentinel_feature_width.saturating_sub(SENTINEL_Q_BASE) / SENTINEL_SPATIAL_FEATURES_PER_AXIS
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// The extension-to-rebuild permutation
// ═══════════════════════════════════════════════════════════════════════════════

/// The rearrangement that carries an extended vector into canonical order.
///
/// `source[j]` is the index, in the extended-but-not-yet-rearranged vector,
/// whose value belongs at canonical index `j`. It is a permutation of
/// `0..extended_p`, so applying it moves every learned weight and drops none
/// (´claim:feature:permutation-carries-every-old-position´).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LayoutPermutation {
    /// The source index, in the post-event vector, for each canonical index.
    source: Vec<usize>,
}

impl LayoutPermutation {
    /// Builds the permutation carrying the post-event layout into the rebuilt
    /// map's canonical order.
    ///
    /// The handlers leave the vector in append-and-compact order: whatever
    /// survived a marginalisation keeps its relative order and closes the gaps
    /// (´alg:dimension:compaction´), and whatever an extension added stands
    /// after it. So the post-event position of a surviving coordinate is its
    /// rank among the pre-event keys the rebuild still issues, and the
    /// positions after them are the appended ones — all of them fresh
    /// isotropic prior blocks, which is why pairing them off in index order is
    /// the whole of what they need.
    ///
    /// Returns `None` when the two layouts do not describe the same vector.
    /// That is the strengthened post-condition the width comparison could not
    /// express: it compares the block layout rather than the total. A refusal
    /// is not a panic — the caller leaves the parameters where they are,
    /// exactly as they stood before this repair, and says so.
    ///
    /// Deregistering a spatial outcome axis was long carried as the case that
    /// refuses rather than rearranges, on the reasoning that retiring an axis
    /// from the middle renumbers every later axis's outcome-memory pair while
    /// the pair is keyed here by that number. Driven through the real
    /// lifecycle path it neither refuses nor rearranges wrongly: the
    /// permutation comes out the *identity*, and every surviving axis's pair
    /// carries that axis's own learned values on the far side of the event
    /// (´claim:lifespan:retiring-a-middle-spatial-axis-leaves-every-surviving-axis-pair-holding-its-own-memory´).
    ///
    /// The reason is worth stating, because two drops that are visibly not the
    /// same drop cancelling exactly reads as luck. The removal takes the
    /// retired axis's *own* pair out of every slot, addressed by that axis's
    /// rank at the moment it is removed, and the compaction closes the gap. The
    /// key filter above takes something else: the rebuilt map issues pair keys
    /// for ranks zero through `m − 2` where the pre-event map issued zero
    /// through `m − 1`, so the key it drops is always the *last*-ranked one,
    /// wherever in the tail the retirement actually happened. Those are
    /// different positions, and the construction's stated assumption — that the
    /// `r`-th surviving pre-event key names whatever compaction left at
    /// post-event position `r` — is false coordinate for coordinate.
    ///
    /// What holds instead is weaker and is enough. Both drops remove the same
    /// count from one contiguous run, the slot's outcome-memory tail, whose
    /// positions are ordered by rank on both sides of the event; the occupancy
    /// indicator and the fixed groups ahead of the run and the batch context
    /// behind it are keyed by nothing that renumbers. Within the run, the
    /// compacted vector holds the surviving axes' pairs in the order those axes
    /// stand in the authoritative model map, because the removal from it is
    /// order-preserving (´entry:assayer:wl-owner-dereg-scan´); and the rebuilt
    /// map gives tail rank `r` to the `r`-th axis of that same order
    /// (´def:extraction:slot´). The `r`-th surviving position and the `r`-th
    /// surviving key therefore name one axis. It is one ordering read twice
    /// rather than two drops that happen to sit near each other, which is why
    /// the distance between them does not enter the argument
    /// (´claim:lifespan:retiring-the-first-ranked-spatial-axis-leaves-every-surviving-axis-pair-holding-its-own-memory´).
    ///
    /// The argument's boundaries are its three premises. It needs the pair run
    /// to be the slot's only rank-keyed run, so that a second one could not
    /// borrow a position from it; it needs the model map's removal to stay
    /// order-preserving, so a swap-removal would take the cancellation with it;
    /// and it needs the retirements it is reading to have been planned against
    /// one entry layout, because "the `r`-th surviving pre-event key" is a
    /// statement about a single layout and says nothing about a vector that
    /// moved between two removals.
    ///
    /// That third premise was originally written as one retirement per
    /// submission, which is what it amounted to while the applier mutated the
    /// parameters event by event against a map the rebuild refreshed only after
    /// all of them: a second retirement addressed the layout the first had
    /// already left, and it did not reach this construction at all — the removal
    /// was refused as out-of-range before any permutation was computed
    /// (´entry:assayer:wl-owner-dereg-scan´). The applier now supplies the
    /// premise structurally instead of by restricting what a submission may
    /// carry. A submission's events are gathered into maximal chains of
    /// operations that are order-free with respect to one another, and a
    /// chain's removals all name their positions against the layout the previous
    /// publication left, so the pre-event map this construction is handed is one
    /// layout for every retirement it covers however many the chain carries
    /// (´dec:construction:compound-batch´). A retirement that is not order-free
    /// with what is open seals the chain and is read against the layout that
    /// chain published, which is the single-retirement case again.
    ///
    /// The identity `slot_axis_offsets` records — put there by the
    /// slot-placement repair (´entry:assayer:wl-feature-frozen-slot-truncation´)
    /// — is what a re-keying would use, and re-keying would make the assumption
    /// hold coordinate for coordinate rather than in aggregate. It would not
    /// change this path's output today, and it is left undone here so that what
    /// a position *means* to the permutation is not altered on an argument the
    /// measurement does not require.
    #[must_use]
    pub fn from_extension(old: &DimensionMap, new: &DimensionMap) -> Option<Self> {
        let old_keys = old.position_keys();
        let new_keys = new.position_keys();

        let new_index: HashMap<PositionKey, usize> = new_keys.iter().enumerate().map(|(idx, &key)| (key, idx)).collect();
        if new_index.len() != new_keys.len() {
            return None;
        }

        // The pre-event keys the rebuild still issues, in pre-event order:
        // their rank here is their post-compaction physical position.
        let surviving: Vec<usize> = old_keys.iter().filter_map(|key| new_index.get(key).copied()).collect();

        let mut source: Vec<Option<usize>> = vec![None; new.p];
        for (post_idx, &canonical) in surviving.iter().enumerate() {
            let slot = source.get_mut(canonical)?;
            if slot.is_some() {
                return None;
            }
            *slot = Some(post_idx);
        }

        // The canonical positions still unclaimed are exactly the ones the
        // extension appended. They take the appended coordinates in order,
        // which is the order the class extension pushes them in.
        let mut appended = surviving.len();
        for slot in &mut source {
            if slot.is_none() {
                *slot = Some(appended);
                appended += 1;
            }
        }
        if appended != new.p {
            return None;
        }

        Some(Self {
            source: source.into_iter().map(|s| s.unwrap_or(0)).collect(),
        })
    }

    /// Returns the permutation as a slice of source indices.
    #[must_use]
    pub fn as_slice(&self) -> &[usize] {
        &self.source
    }

    /// Returns whether the permutation rearranges nothing.
    ///
    /// Registrations arriving in canonical block order need no rearrangement,
    /// and the whole operation is then skippable
    /// (´claim:feature:canonical-order-permutation-is-identity´).
    #[must_use]
    pub fn is_identity(&self) -> bool {
        self.source.iter().enumerate().all(|(j, &i)| j == i)
    }

    /// Rearranges a per-position vector in place.
    ///
    /// The vector must already carry the extended length.
    pub fn apply_to<T: Copy>(&self, values: &mut Vec<T>) {
        if values.len() != self.source.len() {
            return;
        }
        let rearranged: Vec<T> = self.source.iter().map(|&i| values[i]).collect();
        *values = rearranged;
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;

    use super::*;

    fn sentinels(ids: &[u32]) -> IndexMap<SentinelId, ()> {
        ids.iter().map(|&i| (SentinelId(i), ())).collect()
    }

    fn map_of(sent: &[u32], dims: &[u32], axes: &[u32], spatial: usize) -> DimensionMap {
        let dim_ids: Vec<DimensionId> = dims.iter().map(|&d| DimensionId(d)).collect();
        let axis_ids: Vec<OutcomeAxisId> = axes.iter().map(|&a| OutcomeAxisId(a)).collect();
        // The spatially enabled axes are the leading `spatial` of the declared
        // ones. Taking them from the same list is what makes a fixture's
        // spatial axes the fixture's own axes rather than a second, invented
        // set that merely has the right cardinality.
        let spatial_ids = &axis_ids[..spatial.min(axis_ids.len())];
        DimensionMap::rebuild_with_axes_no_interactions(2, &sentinels(sent), &dim_ids, &IndexMap::new(), spatial_ids, &axis_ids)
    }

    /// Every physical position of the vector carries exactly one key and no key
    /// is issued twice, so the enumeration is a bijection between the map's
    /// blocks and the coordinates a model actually holds. Without that the
    /// permutation built from two enumerations could silently drop a learned
    /// weight instead of moving it.
    ///
    /// ´claim:feature:keys-enumerate-every-position-exactly-once´
    /// ´test:unit:keys-enumerate-every-position-exactly-once´
    #[test]
    fn keys_enumerate_every_position_exactly_once() {
        let map = map_of(&[7, 9], &[1, 2], &[3, 4], 2);
        let keys = map.position_keys();

        assert_eq!(keys.len(), map.p, "one key per position");
        let unique: std::collections::HashSet<_> = keys.iter().collect();
        assert_eq!(unique.len(), keys.len(), "keys are distinct");
    }

    /// Registering a Sentinel and then an identity dimension — the ordering the
    /// audit reproduced the defect on — yields a permutation that is not the
    /// identity. The append leaves the Sentinel's learned coordinates sitting
    /// where the rebuild has just placed the identity blocks, and the
    /// permutation is what carries them back to the positions the map now
    /// names.
    ///
    /// ´claim:feature:identity-after-sentinel-permutes´
    /// ´test:unit:identity-after-sentinel-permutes´
    #[test]
    fn identity_after_sentinel_permutes() {
        let before = map_of(&[7], &[], &[], 0);
        let after = map_of(&[7], &[1], &[], 0);

        let perm = LayoutPermutation::from_extension(&before, &after).expect("layouts agree");
        assert_eq!(perm.as_slice().len(), after.p);
        assert!(!perm.is_identity(), "the Sentinel's slot must move");

        // The Sentinel's occupancy indicator sat immediately after the
        // aggregate block and now sits after the identity blocks.
        let old_slot = before.sentinel_slots[&SentinelId(7)].start;
        let new_slot = after.sentinel_slots[&SentinelId(7)].start;
        assert_ne!(old_slot, new_slot, "the slot moved");
        assert_eq!(perm.as_slice()[new_slot], old_slot, "and the permutation carries it");
    }

    /// Registrations that arrive in canonical block order need no
    /// rearrangement: the permutation is the identity map. The repair costs
    /// nothing on the ordering that was already correct, which is why every
    /// test that pinned block positions by registering the identity dimension
    /// first went on passing.
    ///
    /// ´claim:feature:canonical-order-permutation-is-identity´
    /// ´test:unit:canonical-order-permutation-is-identity´
    #[test]
    fn canonical_order_permutation_is_identity() {
        let before = map_of(&[], &[1], &[], 0);
        let after = map_of(&[7], &[1], &[], 0);

        let perm = LayoutPermutation::from_extension(&before, &after).expect("layouts agree");
        assert!(
            perm.is_identity(),
            "a Sentinel appended after the identity blocks is already canonical"
        );
    }

    /// Every position the pre-event map named appears exactly once as a source
    /// in the permutation, and the positions with no source are exactly the
    /// coordinates the extension appended. A learned weight can therefore be
    /// moved but never dropped, and a fresh coordinate never overwrites one.
    ///
    /// ´claim:feature:permutation-carries-every-old-position´
    /// ´test:unit:permutation-carries-every-old-position´
    #[test]
    fn permutation_carries_every_old_position() {
        let before = map_of(&[7, 9], &[], &[], 0);
        let after = map_of(&[7, 9], &[1, 2], &[], 0);

        let perm = LayoutPermutation::from_extension(&before, &after).expect("layouts agree");
        let mut seen = vec![false; after.p];
        for &i in perm.as_slice() {
            assert!(!seen[i], "source {i} used twice");
            seen[i] = true;
        }
        assert!(seen.iter().all(|&s| s), "every source used");
    }

    /// Registering an outcome axis widens the identity blocks and the Sentinel
    /// slots at once, so the appended coordinates belong inside blocks rather
    /// than after them. The permutation opens a gap in each widened block and
    /// slides every later coordinate along, which a block-granular
    /// rearrangement could not express.
    ///
    /// ´claim:feature:axis-registration-permutes-within-blocks´
    /// ´test:unit:axis-registration-permutes-within-blocks´
    #[test]
    fn axis_registration_permutes_within_blocks() {
        let before = map_of(&[7], &[1], &[3], 1);
        let after = map_of(&[7], &[1], &[3, 4], 2);

        let perm = LayoutPermutation::from_extension(&before, &after).expect("layouts agree");
        assert_eq!(perm.as_slice().len(), after.p);
        assert!(!perm.is_identity(), "the widened blocks push later coordinates along");

        // The first axis's identity triple keeps its offset; the Sentinel slot
        // that follows every identity block has moved along by the widening.
        let old_slot = before.sentinel_slots[&SentinelId(7)].start;
        let new_slot = after.sentinel_slots[&SentinelId(7)].start;
        assert!(new_slot > old_slot, "the slot moved along");
        assert_eq!(perm.as_slice()[new_slot], old_slot, "and its occupancy came with it");
    }
}
