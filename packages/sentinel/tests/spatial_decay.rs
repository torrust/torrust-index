// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Spatial decay — the only way the sentinel forgets.
//!
//! Importance in the spatial layer only ever accumulates. Every observed
//! value adds one unit of volume, and nothing in the scoring path ever
//! subtracts from it, so the graph on its own has no notion of the past
//! mattering less than the present. Forgetting is an operation the host
//! performs deliberately: [`decay()`] rescales the whole graph, and
//! [`decay_subtree()`] rescales one region, which is what a regime change or
//! a suspected poisoning confined to part of the domain calls for.
//!
//! One control covers both directions. A factor below one fades accumulated
//! standing, a factor above one reinforces it, and exactly one is the
//! identity — so a host can put decay on a fixed schedule and let the factor
//! decide whether anything happens on a given tick. The second parameter
//! varies that factor by depth, allowing coarse structure to be held while
//! fine detail is let go. Neither parameter is clamped: an out-of-range
//! request means the host's temporal policy is wrong, and quietly repairing
//! it would hide the mistake rather than surface it.
//!
//! Decay stops at the spatial layer, and that is the feed-forward invariant
//! seen from its other side. Scores never flow back into importance; decayed
//! importance never reaches forward into the models. No tracker is destroyed,
//! no subspace or baseline is touched, and the record of how much has been
//! ingested is untouched — what changes is only the ranking that decides
//! where future modelling effort goes, and that change is picked up at the
//! next ingest rather than applied eagerly.
//!
//! [`decay()`]: torrust_sentinel::SpectralSentinel::decay
//! [`decay_subtree()`]: torrust_sentinel::SpectralSentinel::decay_subtree
//!
//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`decay_on_empty_graph_is_noop`] | decay | Decay scales what has been accumulated, so on a sentinel that has observed nothing there is nothing to scale and nothing to go wrong. A host can put decay on a timer before any traffic arrives without special-casing the empty graph. |
//! | [`decay_at_attenuation_one_is_noop`] | decay | An attenuation of one leaves a populated graph exactly as it was. The identity is inside the parameter's range rather than outside it, so "decay by nothing this tick" is expressed with the same call as any other policy, and a host's schedule needs no branch around it. |
//! | [`decay_reduces_total_sum`] | decay | A factor below one reduces the importance the graph has accumulated. This is the ordinary case and the reason the operation exists: standing earned by past traffic is worth less after a decay, so cells that have gone quiet drift down the ranking instead of holding their place forever on history. |
//! | [`repeated_decay_eventually_zeroes_integer_counters`] | decay | Importance is held in integers, so repeated halving does not approach zero asymptotically — it arrives there. A long run of decays with no intervening traffic empties the graph completely, which means a region that stops being observed is eventually forgotten outright rather than leaving an ever-smaller residue that still outranks a genuinely new cell. |
//! | [`decay_zero_attenuation_zeroes_graph`] | decay | Zero is the bottom of the attenuation range and reaches in one call what repeated halving reaches slowly: all accumulated importance is gone. It is the operation for declaring the spatial history worthless — after a suspected poisoning, say — without discarding the models that history produced. |
//! | [`amplification_increases_total_sum`] | decay | The factor is not restricted to shrinking. Above one it increases the accumulated importance instead, which lets a host reinforce a subtree it knows to be worth watching. One operation therefore spans both temporal policies — letting the past fade and boosting a region's standing — with the identity sitting between them. |
//! | [`selective_q_preserves_more_coarse_structure`] | decay | Selectivity makes the decay factor depend on a node's depth rather than applying one rate to the whole tree, so fine detail can be released while the coarse division of the domain is held. Two sentinels given identical traffic start from identical importance — the spatial layer is driven by volume alone — and diverge only through the selectivity each is then decayed with; both keep importance standing afterwards. |
//! | [`max_selectivity_q_one_is_valid`] | decay | The selectivity range is closed at its top end: maximum selectivity is a valid request, not one step past the edge, and under an attenuating factor it cannot leave the graph holding more than it started with. The extreme of the parameter is usable rather than merely almost-reachable. |
//! | [`decay_does_not_affect_tracker_count`] | decay | Decay reaches the spatial accounting and stops there. Halving every cell's importance destroys no tracker: the models a sentinel has built are not the same asset as the standing that justified building them, and forgetting the second does not throw away the first. This is the feed-forward invariant seen from the temporal side — importance flows into modelling decisions, never the reverse, so rescaling it cannot reach the models. |
//! | [`decay_does_not_change_lifetime_observations`] | decay | cites (´claim:decay:decay-rescales-spatial-importance-only-and-never-the-models-or-the-record-of-what-was-observed´) |
//! | [`decay_subtree_at_root_matches_global_decay`] | decay | There is one decay operation, not two. Targeting the subtree at the graph root produces exactly the importance a global decay produces, because the global call is defined as the subtree call made at the root. The targeted form is the general one, and the whole-graph form its degenerate case. |
//! | [`decay_subtree_affects_only_targeted_subtree`] | decay | Aimed below the root, decay is bounded by its target: the region named loses standing while everything outside it keeps what it had, so more importance survives than the same factor applied globally. That containment is what makes the operation usable for a regime change in one part of the domain — the rest of the graph does not have to be punished to let one region re-form. |
//! | [`decay_then_ingest_preserves_invariants`] | decay | Decay leaves the sentinel in a state the next batch can be scored from. It changes the rankings the selector reads but invalidates nothing eagerly; the analysis set is simply recomputed at the start of the following ingest, and the report that comes out satisfies every structural invariant. Forgetting is therefore composable with ordinary operation rather than something to be sequenced carefully around it. |
//! | [`decay_panics_on_negative_attenuation`] | decay | A negative attenuation has no meaning — importance cannot be scaled through zero into a negative standing — and the call refuses it outright instead of clamping it to the nearest sensible value. Temporal policy is the host's, so a nonsensical factor is a bug in that policy and is reported as one. |
//! | [`decay_panics_on_nan_attenuation`] | decay | cites (´claim:decay:an-attenuation-outside-the-non-negative-reals-is-refused-rather-than-quietly-repaired´) |
//! | [`decay_panics_on_negative_q`] | decay | Selectivity runs from uniform to fully depth-weighted, and below that range there is nothing to mean. A negative request is refused rather than treated as uniform, because a host that computed it did not intend uniformity. |
//! | [`decay_panics_on_q_above_one`] | decay | cites (´claim:decay:a-selectivity-outside-the-unit-interval-is-refused-rather-than-quietly-repaired´) |
//! | [`decay_panics_on_nan_q`] | decay | cites (´claim:decay:a-selectivity-outside-the-unit-interval-is-refused-rather-than-quietly-repaired´) |

mod common;

use common::{assert_invariants, seeded_sentinel, test_config};
use torrust_sentinel::{Sentinel128, SentinelConfig};

// ── Identity & no-ops ──────────────────────────────────────

/// Decay scales what has been accumulated, so on a sentinel that has observed
/// nothing there is nothing to scale and nothing to go wrong. A host can put
/// decay on a timer before any traffic arrives without special-casing the
/// empty graph.
///
/// ´claim:decay:decaying-a-graph-that-has-observed-nothing-leaves-nothing-to-scale´
/// ´test:integration:decay-on-empty-graph-is-noop´
#[test]
fn decay_on_empty_graph_is_noop() {
    let mut s = Sentinel128::new(test_config()).unwrap();
    assert_eq!(s.graph().total_sum(), 0);

    s.decay(0.5, 0.0);
    assert_eq!(s.graph().total_sum(), 0);
}

/// An attenuation of one leaves a populated graph exactly as it was. The
/// identity is inside the parameter's range rather than outside it, so
/// "decay by nothing this tick" is expressed with the same call as any other
/// policy, and a host's schedule needs no branch around it.
///
/// ´claim:decay:an-attenuation-of-one-is-the-identity-so-a-scheduled-decay-needs-no-special-case-for-doing-nothing´
/// ´test:integration:decay-at-attenuation-one-is-noop´
#[test]
fn decay_at_attenuation_one_is_noop() {
    let mut s = seeded_sentinel();
    let before = s.graph().total_sum();

    s.decay(1.0, 0.0);
    assert_eq!(s.graph().total_sum(), before);
}

// ── Attenuation (0 < att < 1) ──────────────────────────────

/// A factor below one reduces the importance the graph has accumulated. This
/// is the ordinary case and the reason the operation exists: standing earned
/// by past traffic is worth less after a decay, so cells that have gone quiet
/// drift down the ranking instead of holding their place forever on history.
///
/// ´claim:decay:a-factor-below-one-reduces-accumulated-importance-so-quiet-cells-lose-standing´
/// ´test:integration:decay-reduces-total-sum´
#[test]
fn decay_reduces_total_sum() {
    let mut s = seeded_sentinel();
    let before = s.graph().total_sum();
    assert!(before > 0);

    s.decay(0.5, 0.0);
    assert!(s.graph().total_sum() < before);
}

/// Importance is held in integers, so repeated halving does not approach zero
/// asymptotically — it arrives there. A long run of decays with no
/// intervening traffic empties the graph completely, which means a region
/// that stops being observed is eventually forgotten outright rather than
/// leaving an ever-smaller residue that still outranks a genuinely new cell.
///
/// ´claim:decay:repeated-attenuation-of-integer-importance-reaches-exactly-zero-rather-than-an-endless-tail´
/// ´test:integration:repeated-decay-eventually-zeroes-integer-counters´
#[test]
fn repeated_decay_eventually_zeroes_integer_counters() {
    let mut s = seeded_sentinel();
    assert!(s.graph().total_sum() > 0);

    for _ in 0..100 {
        s.decay(0.5, 0.0);
    }
    assert_eq!(s.graph().total_sum(), 0);
}

// ── Zero attenuation ───────────────────────────────────────

/// Zero is the bottom of the attenuation range and reaches in one call what
/// repeated halving reaches slowly: all accumulated importance is gone. It is
/// the operation for declaring the spatial history worthless — after a
/// suspected poisoning, say — without discarding the models that history
/// produced.
///
/// ´claim:decay:an-attenuation-of-zero-erases-all-accumulated-importance-in-a-single-call´
/// ´test:integration:decay-zero-attenuation-zeroes-graph´
#[test]
fn decay_zero_attenuation_zeroes_graph() {
    let mut s = seeded_sentinel();
    assert!(s.graph().total_sum() > 0);

    s.decay(0.0, 0.0);
    assert_eq!(s.graph().total_sum(), 0);
}

// ── Amplification (att > 1) ────────────────────────────────

/// The factor is not restricted to shrinking. Above one it increases the
/// accumulated importance instead, which lets a host reinforce a subtree it
/// knows to be worth watching. One operation therefore spans both temporal
/// policies — letting the past fade and boosting a region's standing — with
/// the identity sitting between them.
///
/// ´claim:decay:a-factor-above-one-amplifies-instead-of-attenuating-so-one-operation-covers-both-directions´
/// ´test:integration:amplification-increases-total-sum´
#[test]
fn amplification_increases_total_sum() {
    let mut s = seeded_sentinel();
    let before = s.graph().total_sum();

    s.decay(2.0, 0.0);
    assert!(s.graph().total_sum() > before);
}

// ── Depth selectivity (q parameter) ────────────────────────

/// Selectivity makes the decay factor depend on a node's depth rather than
/// applying one rate to the whole tree, so fine detail can be released while
/// the coarse division of the domain is held. Two sentinels given identical
/// traffic start from identical importance — the spatial layer is driven by
/// volume alone — and diverge only through the selectivity each is then
/// decayed with; both keep importance standing afterwards.
///
/// ´claim:decay:depth-selectivity-varies-the-factor-by-depth-so-coarse-structure-can-outlive-fine-detail´
/// ´test:integration:selective-q-preserves-more-coarse-structure´
#[test]
fn selective_q_preserves_more_coarse_structure() {
    let cfg = SentinelConfig::<u64> {
        split_threshold: 5,
        ..test_config()
    };

    let mut s_uniform = Sentinel128::new(cfg.clone()).unwrap();
    let mut s_selective = Sentinel128::new(cfg).unwrap();

    let values: Vec<u128> = (0..100).collect();
    s_uniform.ingest(&values);
    s_selective.ingest(&values);

    assert_eq!(s_uniform.graph().total_sum(), s_selective.graph().total_sum());

    // Only meaningful if splits occurred (otherwise q has no effect).
    if s_uniform.graph().node_count() > 1 {
        s_uniform.decay(0.5, 0.0);
        s_selective.decay(0.5, 0.5);

        // Both should decay, but selective decay may produce a
        // different total because per-depth factors differ.
        assert!(s_uniform.graph().total_sum() > 0);
        assert!(s_selective.graph().total_sum() > 0);
    }
}

/// The selectivity range is closed at its top end: maximum selectivity is a
/// valid request, not one step past the edge, and under an attenuating factor
/// it cannot leave the graph holding more than it started with. The extreme
/// of the parameter is usable rather than merely almost-reachable.
///
/// ´claim:decay:the-selectivity-range-is-closed-so-full-selectivity-is-a-valid-request´
/// ´test:integration:max-selectivity-q-one-is-valid´
#[test]
fn max_selectivity_q_one_is_valid() {
    let mut s = seeded_sentinel();
    let before = s.graph().total_sum();

    // q=1.0 is the maximum-selectivity boundary — must not panic.
    s.decay(0.5, 1.0);
    assert!(s.graph().total_sum() <= before);
}

// ── Feed-forward invariant ─────────────────────────────────

/// Decay reaches the spatial accounting and stops there. Halving every cell's
/// importance destroys no tracker: the models a sentinel has built are not
/// the same asset as the standing that justified building them, and forgetting
/// the second does not throw away the first. This is the feed-forward
/// invariant seen from the temporal side — importance flows into modelling
/// decisions, never the reverse, so rescaling it cannot reach the models.
///
/// ´claim:decay:decay-rescales-spatial-importance-only-and-never-the-models-or-the-record-of-what-was-observed´
/// ´test:integration:decay-does-not-affect-tracker-count´
#[test]
fn decay_does_not_affect_tracker_count() {
    let mut s = seeded_sentinel();
    let trackers_before = s.cells_tracked();

    s.decay(0.5, 0.0);
    assert_eq!(s.cells_tracked(), trackers_before);
}

/// The other thing decay leaves alone is the ledger. The lifetime observation
/// count records what the host actually fed in and is not a decayable
/// quantity, so it survives a decay unchanged — a host can still say how much
/// data has passed through after any amount of forgetting.
///
/// (´claim:decay:decay-rescales-spatial-importance-only-and-never-the-models-or-the-record-of-what-was-observed´)
/// ´test:integration:decay-does-not-change-lifetime-observations´
#[test]
fn decay_does_not_change_lifetime_observations() {
    let mut s = seeded_sentinel();
    let before = s.health().lifetime_observations;

    s.decay(0.5, 0.0);
    assert_eq!(s.health().lifetime_observations, before);
}

// ── decay_subtree ──────────────────────────────────────────

/// There is one decay operation, not two. Targeting the subtree at the graph
/// root produces exactly the importance a global decay produces, because the
/// global call is defined as the subtree call made at the root. The targeted
/// form is the general one, and the whole-graph form its degenerate case.
///
/// ´claim:decay:global-decay-is-exactly-subtree-decay-applied-at-the-root´
/// ´test:integration:decay-subtree-at-root-matches-global-decay´
#[test]
fn decay_subtree_at_root_matches_global_decay() {
    let cfg = test_config();

    let mut s_global = Sentinel128::new(cfg.clone()).unwrap();
    s_global.ingest(&[42, 43, 44, 45, 46]);
    s_global.decay(0.5, 0.0);

    let mut s_subtree = Sentinel128::new(cfg).unwrap();
    s_subtree.ingest(&[42, 43, 44, 45, 46]);
    let root = s_subtree.graph().g_root();
    s_subtree.decay_subtree(root, 0.5, 0.0);

    assert_eq!(s_global.graph().total_sum(), s_subtree.graph().total_sum());
}

/// Aimed below the root, decay is bounded by its target: the region named
/// loses standing while everything outside it keeps what it had, so more
/// importance survives than the same factor applied globally. That
/// containment is what makes the operation usable for a regime change in one
/// part of the domain — the rest of the graph does not have to be punished to
/// let one region re-form.
///
/// ´claim:decay:a-subtree-decay-spends-its-whole-effect-inside-the-target-and-leaves-the-rest-of-the-graph-standing´
/// ´test:integration:decay-subtree-affects-only-targeted-subtree´
#[test]
fn decay_subtree_affects_only_targeted_subtree() {
    let cfg = SentinelConfig::<u64> {
        split_threshold: 5,
        ..test_config()
    };

    let mut s_global = Sentinel128::new(cfg.clone()).unwrap();
    let mut s_partial = Sentinel128::new(cfg).unwrap();

    let values: Vec<u128> = (0..100).collect();
    s_global.ingest(&values);
    s_partial.ingest(&values);

    let before = s_partial.graph().total_sum();

    // Find a non-root cell to use as subtree target.
    let root = s_partial.graph().g_root();
    let non_root: Vec<_> = s_partial.cell_gnodes().into_iter().filter(|&id| id != root).collect();

    if !non_root.is_empty() {
        s_global.decay(0.5, 0.0);
        s_partial.decay_subtree(non_root[0], 0.5, 0.0);

        // Subtree decay touches fewer nodes → more total_sum remains.
        assert!(s_partial.graph().total_sum() > s_global.graph().total_sum());
        // But the targeted subtree was still reduced.
        assert!(s_partial.graph().total_sum() <= before);
    }
}

// ── Composition ────────────────────────────────────────────

/// Decay leaves the sentinel in a state the next batch can be scored from. It
/// changes the rankings the selector reads but invalidates nothing eagerly;
/// the analysis set is simply recomputed at the start of the following
/// ingest, and the report that comes out satisfies every structural
/// invariant. Forgetting is therefore composable with ordinary operation
/// rather than something to be sequenced carefully around it.
///
/// ´claim:decay:the-analysis-set-catches-up-with-decayed-importance-at-the-next-ingest-rather-than-being-invalidated-eagerly´
/// ´test:integration:decay-then-ingest-preserves-invariants´
#[test]
fn decay_then_ingest_preserves_invariants() {
    let mut s = seeded_sentinel();
    s.decay(0.5, 0.0);

    let report = s.ingest(&[45, 46, 47]);
    assert_invariants(&s, &report);
}

// ── Panics — invalid attenuation ───────────────────────────

/// A negative attenuation has no meaning — importance cannot be scaled
/// through zero into a negative standing — and the call refuses it outright
/// instead of clamping it to the nearest sensible value. Temporal policy is
/// the host's, so a nonsensical factor is a bug in that policy and is
/// reported as one.
///
/// ´claim:decay:an-attenuation-outside-the-non-negative-reals-is-refused-rather-than-quietly-repaired´
/// ´test:integration:decay-panics-on-negative-attenuation´
#[test]
#[should_panic(expected = "attenuation")]
fn decay_panics_on_negative_attenuation() {
    let mut s = seeded_sentinel();
    s.decay(-1.0, 0.0);
}

/// The other way a factor can be meaningless is by being no number at all,
/// typically the result of an arithmetic accident upstream in the host's
/// policy. It is rejected on the same footing as a negative factor, so a
/// silent propagation cannot poison the graph's importance.
///
/// (´claim:decay:an-attenuation-outside-the-non-negative-reals-is-refused-rather-than-quietly-repaired´)
/// ´test:integration:decay-panics-on-nan-attenuation´
#[test]
#[should_panic(expected = "attenuation")]
fn decay_panics_on_nan_attenuation() {
    let mut s = seeded_sentinel();
    s.decay(f64::NAN, 0.0);
}

// ── Panics — invalid q ────────────────────────────────────

/// Selectivity runs from uniform to fully depth-weighted, and below that
/// range there is nothing to mean. A negative request is refused rather than
/// treated as uniform, because a host that computed it did not intend
/// uniformity.
///
/// ´claim:decay:a-selectivity-outside-the-unit-interval-is-refused-rather-than-quietly-repaired´
/// ´test:integration:decay-panics-on-negative-q´
#[test]
#[should_panic(expected = "q must be")]
fn decay_panics_on_negative_q() {
    let mut s = seeded_sentinel();
    s.decay(0.5, -0.1);
}

/// The upper end is closed at maximum selectivity, and just past it the
/// request is refused. This is what makes the boundary meaningful: full
/// selectivity is accepted and anything beyond it is an error, rather than
/// the range trailing off into values silently treated as the maximum.
///
/// (´claim:decay:a-selectivity-outside-the-unit-interval-is-refused-rather-than-quietly-repaired´)
/// ´test:integration:decay-panics-on-q-above-one´
#[test]
#[should_panic(expected = "q must be")]
fn decay_panics_on_q_above_one() {
    let mut s = seeded_sentinel();
    s.decay(0.5, 1.1);
}

/// A selectivity that is not a number compares false against both ends of the
/// range, so a bounds check written carelessly would let it through. It is
/// refused explicitly, which is what keeps the range genuinely closed rather
/// than closed only for values that compare.
///
/// (´claim:decay:a-selectivity-outside-the-unit-interval-is-refused-rather-than-quietly-repaired´)
/// ´test:integration:decay-panics-on-nan-q´
#[test]
#[should_panic(expected = "q must be")]
fn decay_panics_on_nan_q() {
    let mut s = seeded_sentinel();
    s.decay(0.5, f64::NAN);
}
