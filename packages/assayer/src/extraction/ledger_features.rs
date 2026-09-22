// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`no_spatial_axes`] | extract | The ledger block is three whole-entry quantities — adverse rate, compressed valence and raw valence — followed by two slots for every spatial axis the caller names. With no axes named the block is exactly those three, in that order, so a deployment that tracks no spatial outcomes still produces a well-formed, fixed-width ledger block. |
//! | [`single_spatial_axis`] | extract | cites (´claim:extract:ledger-features-are-three-base-quantities-plus-two-per-spatial-axis´) |
//! | [`multiple_spatial_axes`] | extract | Per-axis valences are paired by axis and not grouped by kind: each axis contributes its compressed valence immediately followed by its own raw valence, and only then does the next axis begin. The pair is the unit the rest of the package addresses — the dimension map hands out one offset per axis and expects that axis's two features to sit at it, and the standardisation class vector pushes a compressed class then a raw class once per axis — so a layout grouped by kind would hand the second axis's compressed average to the raw class's prior, whose variance differs from the compressed one by a factor of four. |
//! | [`missing_axis_defaults_to_zero`] | extract | An axis the caller names but the view has never recorded contributes zero in its slot while its neighbours keep their true values. The slot belongs to the axis identifier rather than to whatever the view happens to contain, so an axis that has seen no outcomes yet cannot shift every later feature one place along and silently re-label the whole block. |
//! | [`neutral_view_all_zeros`] | extract | The ledger's neutral state extracts to zero in every slot, base and per-axis alike. An entry that has recorded no outcomes therefore enters the model as no evidence at all rather than as a mild opinion, and the ledger block stays silent until outcomes actually arrive. |

//! Ledger outcome feature extraction.
//!
//! Extracts features from the decayed ledger view. The count depends on
//! the number of spatial outcome axes.
//!
//! # Feature Layout
//!
//! | Index | Feature | Description |
//! |-------|---------|-------------|
//! | 0 | Bad rate | Decayed EWMA of adverse outcome rate |
//! | 1 | Compressed valence | Decayed EWMA of compressed valence |
//! | 2 | Raw valence | Decayed EWMA of raw valence |
//! | 3 + 2k | Per-axis compressed | Compressed valence of the k-th spatial axis |
//! | 4 + 2k | Per-axis raw | Raw valence of the k-th spatial axis |
//!
//! # Cross-References
//!
//! - (´tab:extraction:ledger-features´) — the outcome-memory features this
//!   module reads out of the decayed view

use crate::ledger::DecayedView;
use crate::types::OutcomeAxisId;

// ═══════════════════════════════════════════════════════════════════════════════
// Ledger Feature Extraction
// ═══════════════════════════════════════════════════════════════════════════════

/// Extracts ledger features from a decayed view.
///
/// Returns `3 + 2 × spatial_axis_ids.len()` features.
///
/// # Arguments
///
/// * `view` — Decayed ledger view from `LedgerEntry::read_decayed()`.
/// * `spatial_axis_ids` — Ordered list of spatial outcome axis IDs.
///
/// # Returns
///
/// A vector of f64 values: `[bad_rate, compressed_valence, raw_valence,
/// compressed_axis_0, raw_axis_0, ..., compressed_axis_n, raw_axis_n]` — the
/// per-axis features paired by axis rather than grouped by kind.
///
/// # Cross-References
///
/// - (´tab:extraction:ledger-features´) — the outcome-memory features
#[must_use]
pub fn extract_ledger_features(view: &DecayedView, spatial_axis_ids: &[OutcomeAxisId]) -> Vec<f64> {
    let m_s = spatial_axis_ids.len();
    let mut features = Vec::with_capacity(3 + 2 * m_s);

    // Base features (3)
    features.push(view.bad_rate);
    features.push(view.compressed_valence);
    features.push(view.raw_valence);

    // Per-axis pairs, interleaved: each axis's compressed valence immediately
    // followed by its own raw valence, one pair per eligible axis
    // (´tab:extraction:reference-index´). The map's per-axis outcome-memory
    // offsets address that pair as a unit (´def:dimension:layers´).
    for axis_id in spatial_axis_ids {
        let recorded = view.per_axis.iter().find(|(id, _, _)| id == axis_id);
        features.push(recorded.map_or(0.0, |(_, c, _)| *c));
        features.push(recorded.map_or(0.0, |(_, _, r)| *r));
    }

    features
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    #![allow(clippy::similar_names)]

    use super::*;

    fn make_view(bad_rate: f64, comp: f64, raw: f64) -> DecayedView {
        DecayedView {
            bad_rate,
            compressed_valence: comp,
            raw_valence: raw,
            per_axis: Vec::new(),
        }
    }

    fn make_view_with_axes(bad_rate: f64, comp: f64, raw: f64, axes: Vec<(OutcomeAxisId, f64, f64)>) -> DecayedView {
        DecayedView {
            bad_rate,
            compressed_valence: comp,
            raw_valence: raw,
            per_axis: axes,
        }
    }

    /// The ledger block is three whole-entry quantities — adverse rate,
    /// compressed valence and raw valence — followed by two slots for every
    /// spatial axis the caller names. With no axes named the block is exactly
    /// those three, in that order, so a deployment that tracks no spatial
    /// outcomes still produces a well-formed, fixed-width ledger block.
    ///
    /// ´claim:extract:ledger-features-are-three-base-quantities-plus-two-per-spatial-axis´
    /// ´test:unit:no-spatial-axes´
    #[test]
    fn no_spatial_axes() {
        let view = make_view(0.1, 0.5, 2.0);
        let axes: &[OutcomeAxisId] = &[];
        let features = extract_ledger_features(&view, axes);

        assert_eq!(features.len(), 3);
        assert!((features[0] - 0.1).abs() < 1e-14, "bad_rate");
        assert!((features[1] - 0.5).abs() < 1e-14, "compressed_valence");
        assert!((features[2] - 2.0).abs() < 1e-14, "raw_valence");
    }

    /// Naming one spatial axis widens the block from three slots to five, the
    /// two extra carrying that axis's compressed and raw valence while the
    /// three base quantities keep their leading positions. The axis list is
    /// what sets the width, so the caller's own ordering — not whatever the
    /// view happens to hold — determines the shape of the block.
    ///
    /// (´claim:extract:ledger-features-are-three-base-quantities-plus-two-per-spatial-axis´)
    /// ´test:unit:single-spatial-axis´
    #[test]
    fn single_spatial_axis() {
        let axis = OutcomeAxisId(1);
        let axes = vec![(axis, 0.3, 1.5)];

        let view = make_view_with_axes(0.1, 0.5, 2.0, axes);
        let spatial_axes = &[axis];
        let features = extract_ledger_features(&view, spatial_axes);

        assert_eq!(features.len(), 5); // 3 + 2×1
        assert!((features[0] - 0.1).abs() < 1e-14, "bad_rate");
        assert!((features[1] - 0.5).abs() < 1e-14, "compressed_valence");
        assert!((features[2] - 2.0).abs() < 1e-14, "raw_valence");
        assert!((features[3] - 0.3).abs() < 1e-14, "axis_0 compressed");
        assert!((features[4] - 1.5).abs() < 1e-14, "axis_0 raw");
    }

    /// Per-axis valences are paired by axis and not grouped by kind: each
    /// axis contributes its compressed valence immediately followed by its own
    /// raw valence, and only then does the next axis begin. The pair is the
    /// unit the rest of the package addresses — the dimension map hands out one
    /// offset per axis and expects that axis's two features to sit at it, and
    /// the standardisation class vector pushes a compressed class then a raw
    /// class once per axis — so a layout grouped by kind would hand the second
    /// axis's compressed average to the raw class's prior, whose variance
    /// differs from the compressed one by a factor of four.
    ///
    /// ´claim:extract:per-axis-valences-are-paired-by-axis-compressed-then-raw´
    /// ´test:unit:multiple-spatial-axes´
    #[test]
    fn multiple_spatial_axes() {
        let axis1 = OutcomeAxisId(1);
        let axis2 = OutcomeAxisId(2);
        let axes = vec![(axis1, 0.3, 1.5), (axis2, 0.6, 3.0)];

        let view = make_view_with_axes(0.1, 0.5, 2.0, axes);
        let spatial_axes = &[axis1, axis2];
        let features = extract_ledger_features(&view, spatial_axes);

        assert_eq!(features.len(), 7); // 3 + 2×2
        assert!((features[0] - 0.1).abs() < 1e-14, "bad_rate");
        assert!((features[1] - 0.5).abs() < 1e-14, "compressed_valence");
        assert!((features[2] - 2.0).abs() < 1e-14, "raw_valence");
        assert!((features[3] - 0.3).abs() < 1e-14, "axis1 compressed");
        assert!((features[4] - 1.5).abs() < 1e-14, "axis1 raw");
        assert!((features[5] - 0.6).abs() < 1e-14, "axis2 compressed");
        assert!((features[6] - 3.0).abs() < 1e-14, "axis2 raw");
    }

    /// An axis the caller names but the view has never recorded contributes
    /// zero in its slot while its neighbours keep their true values. The slot
    /// belongs to the axis identifier rather than to whatever the view happens
    /// to contain, so an axis that has seen no outcomes yet cannot shift every
    /// later feature one place along and silently re-label the whole block.
    ///
    /// ´claim:extract:an-axis-absent-from-the-view-contributes-zero-without-shifting-later-slots´
    /// ´test:unit:missing-axis-defaults-to-zero´
    #[test]
    fn missing_axis_defaults_to_zero() {
        let axis1 = OutcomeAxisId(1);
        let axis2 = OutcomeAxisId(2); // Not in vec
        let axes = vec![(axis1, 0.3, 1.5)];

        let view = make_view_with_axes(0.1, 0.5, 2.0, axes);
        let spatial_axes = &[axis1, axis2];
        let features = extract_ledger_features(&view, spatial_axes);

        assert!((features[3] - 0.3).abs() < 1e-14, "axis1 compressed present");
        assert!((features[4] - 1.5).abs() < 1e-14, "axis1 raw present");
        assert!((features[5]).abs() < 1e-14, "axis2 compressed missing → 0");
        assert!((features[6]).abs() < 1e-14, "axis2 raw missing → 0");
    }

    /// The ledger's neutral state extracts to zero in every slot, base and
    /// per-axis alike. An entry that has recorded no outcomes therefore enters
    /// the model as no evidence at all rather than as a mild opinion, and the
    /// ledger block stays silent until outcomes actually arrive.
    ///
    /// ´claim:extract:a-neutral-ledger-view-extracts-to-zero-in-every-slot´
    /// ´test:unit:neutral-view-all-zeros´
    #[test]
    fn neutral_view_all_zeros() {
        let view = DecayedView::neutral();
        let axes = &[OutcomeAxisId(1)];
        let features = extract_ledger_features(&view, axes);

        assert!(features.iter().all(|&v| v == 0.0), "All zeros for neutral view");
    }
}
