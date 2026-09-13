// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`batch_report_json_round_trip`] | serde | A whole batch report survives being written out and read back with its structure intact: the cell, ancestor and coordination lists return at the same lengths, and the summary counts are unchanged. The nesting is deep and generic over the coordinate type, so this is the claim that the readout can be handed to a host in another process at all. |
//! | [`empty_batch_report_json_round_trip`] | serde | cites (´claim:serde:a-whole-batch-report-survives-a-round-trip-with-its-structure-intact´) |
//! | [`batch_report_without_age_field_deserializes`] | serde | A payload written before the age existed still reads back, its age absent rather than filled in: a report object simply missing that field deserialises with no age and everything else intact. Absence is the honest reading of an older payload — a number would claim a measurement the sender never made and could not have sent. |
//! | [`analysis_set_summary_json_round_trip`] | serde | A report component can be carried on its own, not only inside the batch that produced it: the analysis-set summary alone round-trips with its sizes, its depth span and its skipped-cell count intact. A host forwarding structural state to one consumer and scores to another need not ship the whole readout to either. |
//! | [`cell_report_json_round_trip`] | serde | Optional detail crosses the boundary as present or absent rather than collapsing: each cell report returns with its depth, counts and scores, and with per-sample detail still attached and still the same length when the host enabled it. An option that silently became absent in transit would look to the receiver exactly like a host that had never asked for it. |
//! | [`contour_snapshot_json_round_trip`] | serde | cites (´claim:serde:every-report-component-can-be-carried-on-its-own-not-only-inside-the-batch-that-produced-it´) |
//! | [`coordination_report_json_round_trip`] | serde | cites (´claim:serde:optional-detail-crosses-the-boundary-as-present-or-absent-rather-than-collapsing´) |
//! | [`health_report_json_round_trip`] | serde | cites (´claim:serde:every-report-component-can-be-carried-on-its-own-not-only-inside-the-batch-that-produced-it´) |
//! | [`cell_inspection_json_round_trip`] | serde | A cell inspection, taken by handle between batches rather than emitted by one, transports like any other readout: its depth, width and rank return exact and its per-axis baselines come back intact. Both ways of getting state out of the sentinel are equally publishable, so a host is not forced through the batch path merely to be able to forward what it learned. |
//! | [`member_score_json_round_trip`] | serde | Integers cross exactly and floating-point values within one unit in the last place — the residue of passing through a decimal text form, checked here on a member score whose interval runs to the extreme of the coordinate domain. The cell identity is integral and so is preserved outright, while the scores are close enough that no threshold a host applies can turn on the difference. |
//! | [`sample_score_json_round_trip`] | serde | cites (´claim:serde:a-float-returns-within-one-unit-in-the-last-place-of-the-value-that-was-sent´) |

//! Transport tests for the readouts the sentinel hands back: what survives
//! being written out and read back in, under the optional serialisation
//! feature.
//!
//! The sentinel measures and the host decides, and the host is often
//! somewhere else — another process, a log, a queue, a store consulted long
//! after the batch that produced the numbers. Serialisation is therefore not
//! a convenience bolted onto the report types but part of how the
//! measurement reaches whoever acts on it, and a field that does not survive
//! the trip is a measurement that was never really published.
//!
//! Two properties are worth stating plainly. Every report component
//! serialises on its own as well as inside the batch that produced it, so a
//! host can forward just the health snapshot or just one cell without
//! carrying the whole readout. And the values that come back are the values
//! that went out — floating-point figures within one unit in the last place,
//! the residue of passing through a decimal text form, which is close enough
//! that no comparison a host makes can turn on the difference.
//!
//! The reports are generated from a sentinel driven only far enough to
//! produce a structurally complete readout — cells, ancestors, coordination
//! contexts and per-sample detail all populated. What is under test here is
//! the shape of what crosses the boundary, not whether the statistics behind
//! it have converged.

#![cfg(feature = "serde")]

mod common;

use common::{ScenarioBuilder, cell_values, integration_config};
use torrust_sentinel::{
    AnalysisSetSummary, BatchReport, CellInspection, CellReport, ContourSnapshot, CoordinationReport, HealthReport, MemberScore,
    SampleScore, Sentinel128, SentinelConfig,
};

/// Assert two f64s are within 1 ULP (unit in the last place) of each other.
///
/// JSON round-trips through decimal representation can lose 1 ULP for
/// certain edge-case values at the boundary of shortest-representation
/// algorithms (e.g. when the 15-digit and 16-digit decimal forms straddle
/// a float boundary).
fn assert_f64_ulp(label: &str, a: f64, b: f64) {
    let a_bits = a.to_bits();
    let b_bits = b.to_bits();
    let ulp_dist = a_bits.abs_diff(b_bits);
    assert!(
        ulp_dist <= 1,
        "{label}: {a} vs {b}, ULP distance = {ulp_dist} (bits: {a_bits} vs {b_bits})"
    );
}

/// Produce a report with enough structure for round-trip testing.
///
/// Only 5 warm-up batches — serde tests need a structurally complete
/// report (cells, ancestors, coordination), not a statistically
/// converged one.  See ADR-S-012.
fn rich_report() -> BatchReport<u128> {
    let cfg = SentinelConfig::<u64> {
        split_threshold: 10,
        per_sample_scores: true,
        ..integration_config()
    };
    let (mut s, _) = ScenarioBuilder::new()
        .config(cfg)
        .seed_range(0xA, 16)
        .seed_range(0xB, 16)
        .warm_batches(4)
        .build_with_reports();

    s.ingest(&[cell_values(0xA, 8), cell_values(0xB, 8)].concat())
}

// ─── Composite reports ──────────────────────────────────────

/// A whole batch report survives being written out and read back with its
/// structure intact: the cell, ancestor and coordination lists return at the
/// same lengths, and the summary counts are unchanged. The nesting is deep
/// and generic over the coordinate type, so this is the claim that the
/// readout can be handed to a host in another process at all.
///
/// ´claim:serde:a-whole-batch-report-survives-a-round-trip-with-its-structure-intact´
/// ´test:integration:batch-report-json-round-trip´
#[test]
fn batch_report_json_round_trip() {
    let report = rich_report();

    let json = serde_json::to_string(&report).unwrap();
    let deser: BatchReport<u128> = serde_json::from_str(&json).unwrap();

    assert_eq!(report.cell_reports.len(), deser.cell_reports.len());
    assert_eq!(report.ancestor_reports.len(), deser.ancestor_reports.len());
    assert_eq!(report.coordination_reports.len(), deser.coordination_reports.len());
    assert_eq!(
        report.analysis_set_summary.competitive_size,
        deser.analysis_set_summary.competitive_size,
    );
    assert_eq!(report.analysis_set_summary.full_size, deser.analysis_set_summary.full_size);
    assert_eq!(report.oldest_observation_age_micros, deser.oldest_observation_age_micros);
}

/// The empty end of the same trip: a report from an ingest with no
/// observations comes back with its lists present and empty rather than
/// missing. Emptiness is carried across the boundary as a fact, so a
/// receiving host distinguishes "nothing reported" from a truncated or
/// malformed readout.
///
/// (´claim:serde:a-whole-batch-report-survives-a-round-trip-with-its-structure-intact´)
/// ´test:integration:empty-batch-report-json-round-trip´
#[test]
fn empty_batch_report_json_round_trip() {
    let mut s = Sentinel128::new(integration_config()).unwrap();
    let report = s.ingest(&[]);

    let json = serde_json::to_string(&report).unwrap();
    let deser: BatchReport<u128> = serde_json::from_str(&json).unwrap();

    assert!(deser.cell_reports.is_empty());
    assert!(deser.ancestor_reports.is_empty());
    assert!(deser.coordination_reports.is_empty());
    assert!(deser.oldest_observation_age_micros.is_none());
}

/// A payload written before the age existed still reads back, its age absent
/// rather than filled in: a report object simply missing that field
/// deserialises with no age and everything else intact. Absence is the honest
/// reading of an older payload — a number would claim a measurement the sender
/// never made and could not have sent.
///
/// ´claim:serde:a-payload-lacking-the-observation-age-reads-back-with-no-age-rather-than-a-fabricated-one´
/// ´test:integration:batch-report-without-age-field-deserializes´
#[test]
fn batch_report_without_age_field_deserializes() {
    let report = rich_report();
    assert!(
        report.oldest_observation_age_micros.is_some(),
        "a live batch states its age, so there is something to take away",
    );

    // Cut the field back out of the text. The surgery is textual because a
    // 128-bit coordinate does not fit `serde_json::Value`'s number, so the
    // payload cannot be taken apart as a tree and put back together.
    let json = serde_json::to_string(&report).unwrap();
    let key = ",\"oldest_observation_age_micros\":";
    let at = json.find(key).expect("a live batch's payload carries the age field");
    let closing = json.rfind('}').expect("a batch report serialises as a JSON object");
    let cut = &json[at + key.len()..closing];
    assert!(
        cut.parse::<u64>().is_ok(),
        "the age should be the object's last member, but {cut} stands between it and the end",
    );
    let older_payload = format!("{}{}", &json[..at], &json[closing..]);

    let deser: BatchReport<u128> = serde_json::from_str(&older_payload).unwrap();

    assert!(deser.oldest_observation_age_micros.is_none());
    assert_eq!(report.cell_reports.len(), deser.cell_reports.len());
    assert_eq!(report.ancestor_reports.len(), deser.ancestor_reports.len());
    assert_eq!(report.analysis_set_summary.full_size, deser.analysis_set_summary.full_size,);
}

// ─── Component reports (alphabetical) ───────────────────────

/// A report component can be carried on its own, not only inside the batch
/// that produced it: the analysis-set summary alone round-trips with its
/// sizes, its depth span and its skipped-cell count intact. A host
/// forwarding structural state to one consumer and scores to another need
/// not ship the whole readout to either.
///
/// ´claim:serde:every-report-component-can-be-carried-on-its-own-not-only-inside-the-batch-that-produced-it´
/// ´test:integration:analysis-set-summary-json-round-trip´
#[test]
fn analysis_set_summary_json_round_trip() {
    let report = rich_report();

    let json = serde_json::to_string(&report.analysis_set_summary).unwrap();
    let deser: AnalysisSetSummary = serde_json::from_str(&json).unwrap();

    assert_eq!(report.analysis_set_summary.competitive_size, deser.competitive_size);
    assert_eq!(report.analysis_set_summary.full_size, deser.full_size);
    assert_eq!(report.analysis_set_summary.investment_set_size, deser.investment_set_size);
    assert_eq!(report.analysis_set_summary.depth_range, deser.depth_range);
    assert_eq!(
        report.analysis_set_summary.degenerate_cells_skipped,
        deser.degenerate_cells_skipped,
    );
}

/// Optional detail crosses the boundary as present or absent rather than
/// collapsing: each cell report returns with its depth, counts and scores,
/// and with per-sample detail still attached and still the same length when
/// the host enabled it. An option that silently became absent in transit
/// would look to the receiver exactly like a host that had never asked for
/// it.
///
/// ´claim:serde:optional-detail-crosses-the-boundary-as-present-or-absent-rather-than-collapsing´
/// ´test:integration:cell-report-json-round-trip´
#[test]
fn cell_report_json_round_trip() {
    let report = rich_report();

    for cr in &report.cell_reports {
        let json = serde_json::to_string(cr).unwrap();
        let deser: CellReport<u128> = serde_json::from_str(&json).unwrap();

        assert_eq!(cr.depth, deser.depth);
        assert_eq!(cr.sample_count, deser.sample_count);
        assert_eq!(cr.rank, deser.rank);
        assert_eq!(cr.is_competitive, deser.is_competitive);
        assert_f64_ulp("novelty.mean", cr.scores.novelty.mean, deser.scores.novelty.mean);
        assert_f64_ulp(
            "novelty.clip_pressure",
            cr.scores.novelty.clip_pressure,
            deser.scores.novelty.clip_pressure,
        );
        // per_sample populated when per_sample_scores is enabled.
        assert_eq!(cr.per_sample.is_some(), deser.per_sample.is_some());
        if let (Some(orig), Some(rt)) = (&cr.per_sample, &deser.per_sample) {
            assert_eq!(orig.len(), rt.len());
        }
    }
}

/// The contour travels alone as well, its plateau and cell counts exact and
/// its accumulated importance intact. This is the section a host watching
/// spatial structure over time would forward on its own, so it has to be
/// meaningful detached from the scores it shipped beside.
///
/// (´claim:serde:every-report-component-can-be-carried-on-its-own-not-only-inside-the-batch-that-produced-it´)
/// ´test:integration:contour-snapshot-json-round-trip´
#[test]
fn contour_snapshot_json_round_trip() {
    let report = rich_report();

    let json = serde_json::to_string(&report.contour).unwrap();
    let deser: ContourSnapshot = serde_json::from_str(&json).unwrap();

    assert_eq!(report.contour.plateau_count, deser.plateau_count);
    assert_eq!(report.contour.cell_count, deser.cell_count);
    assert_f64_ulp("total_importance", report.contour.total_importance, deser.total_importance);
}

/// The optional-detail rule holds a tier up, where the attachment is a list
/// of per-member scores rather than per-sample ones: each coordination
/// report returns with its group size and rank, and with its membership
/// still present and the same length. The identity of who was in the group
/// is what makes a group finding actionable, so it must not be the part that
/// transport drops.
///
/// (´claim:serde:optional-detail-crosses-the-boundary-as-present-or-absent-rather-than-collapsing´)
/// ´test:integration:coordination-report-json-round-trip´
#[test]
fn coordination_report_json_round_trip() {
    let report = rich_report();

    for coord in &report.coordination_reports {
        let json = serde_json::to_string(coord).unwrap();
        let deser: CoordinationReport<u128> = serde_json::from_str(&json).unwrap();

        assert_eq!(coord.depth, deser.depth);
        assert_eq!(coord.cells_reporting, deser.cells_reporting);
        assert_eq!(coord.rank, deser.rank);
        assert_f64_ulp("novelty.mean", coord.scores.novelty.mean, deser.scores.novelty.mean);
        // per_member populated when per_sample_scores is enabled.
        assert_eq!(coord.per_member.is_some(), deser.per_member.is_some());
        if let (Some(orig), Some(rt)) = (&coord.per_member, &deser.per_member) {
            assert_eq!(orig.len(), rt.len());
        }
    }
}

/// Health travels alone too, and it is the component with the most nesting:
/// the tracker counts return exactly and the clip-pressure distribution
/// inside it survives with its ends and its mean. Operational state is
/// typically the piece a host routes to a different consumer from the
/// scores, which is precisely why it has to stand on its own.
///
/// (´claim:serde:every-report-component-can-be-carried-on-its-own-not-only-inside-the-batch-that-produced-it´)
/// ´test:integration:health-report-json-round-trip´
#[test]
fn health_report_json_round_trip() {
    let report = rich_report();

    let json = serde_json::to_string(&report.health).unwrap();
    let deser: HealthReport = serde_json::from_str(&json).unwrap();

    assert_eq!(report.health.lifetime_observations, deser.lifetime_observations);
    assert_eq!(report.health.active_trackers, deser.active_trackers);
    assert_eq!(report.health.active_competitive_trackers, deser.active_competitive_trackers);
    assert_eq!(report.health.active_ancestor_trackers, deser.active_ancestor_trackers);
    assert_eq!(report.health.investment_set_size, deser.investment_set_size);
    assert_f64_ulp(
        "clip_pressure_distribution.min",
        report.health.clip_pressure_distribution.min,
        deser.clip_pressure_distribution.min,
    );
    assert_f64_ulp(
        "clip_pressure_distribution.max",
        report.health.clip_pressure_distribution.max,
        deser.clip_pressure_distribution.max,
    );
    assert_f64_ulp(
        "clip_pressure_distribution.mean",
        report.health.clip_pressure_distribution.mean,
        deser.clip_pressure_distribution.mean,
    );
}

// ─── Inspection types ───────────────────────────────────────

/// A cell inspection, taken by handle between batches rather than emitted by
/// one, transports like any other readout: its depth, width and rank return
/// exact and its per-axis baselines come back intact. Both ways of getting
/// state out of the sentinel are equally publishable, so a host is not
/// forced through the batch path merely to be able to forward what it
/// learned.
///
/// ´claim:serde:an-inspection-taken-outside-the-batch-path-transports-like-any-other-readout´
/// ´test:integration:cell-inspection-json-round-trip´
#[test]
fn cell_inspection_json_round_trip() {
    let cfg = SentinelConfig::<u64> {
        split_threshold: 10,
        ..integration_config()
    };
    let (s, _) = ScenarioBuilder::new()
        .config(cfg)
        .seed_range(0xA, 20)
        .warm_batches(4)
        .build_with_reports();

    let root = s.graph().g_root();
    let inspection = s.inspect_cell(root).unwrap();

    let json = serde_json::to_string(&inspection).unwrap();
    let deser: CellInspection<u128> = serde_json::from_str(&json).unwrap();

    assert_eq!(inspection.depth, deser.depth);
    assert_eq!(inspection.analysis_width, deser.analysis_width);
    assert_eq!(inspection.rank, deser.rank);
    assert_f64_ulp(
        "baselines.novelty.mean",
        inspection.baselines.novelty.mean,
        deser.baselines.novelty.mean,
    );
}

// ─── Leaf types ─────────────────────────────────────────────

/// Integers cross exactly and floating-point values within one unit in the
/// last place — the residue of passing through a decimal text form, checked
/// here on a member score whose interval runs to the extreme of the
/// coordinate domain. The cell identity is integral and so is preserved
/// outright, while the scores are close enough that no threshold a host
/// applies can turn on the difference.
///
/// ´claim:serde:a-float-returns-within-one-unit-in-the-last-place-of-the-value-that-was-sent´
/// ´test:integration:member-score-json-round-trip´
#[test]
fn member_score_json_round_trip() {
    let ms = MemberScore::<u128> {
        cell_start: 0,
        cell_end: u128::MAX / 2,
        cell_depth: 1,
        novelty: 0.5,
        displacement: 0.3,
        surprise: 0.7,
        coherence: 0.9,
        novelty_z: 1.0,
        displacement_z: -0.5,
        surprise_z: 2.1,
        coherence_z: 0.0,
    };

    let json = serde_json::to_string(&ms).unwrap();
    let deser: MemberScore<u128> = serde_json::from_str(&json).unwrap();

    assert_eq!(ms.cell_start, deser.cell_start);
    assert_eq!(ms.cell_end, deser.cell_end);
    assert_eq!(ms.cell_depth, deser.cell_depth);
    assert_f64_ulp("novelty", ms.novelty, deser.novelty);
    assert_f64_ulp("displacement", ms.displacement, deser.displacement);
    assert_f64_ulp("surprise", ms.surprise, deser.surprise);
    assert_f64_ulp("coherence", ms.coherence, deser.coherence);
    assert_f64_ulp("novelty_z", ms.novelty_z, deser.novelty_z);
    assert_f64_ulp("displacement_z", ms.displacement_z, deser.displacement_z);
    assert_f64_ulp("surprise_z", ms.surprise_z, deser.surprise_z);
    assert_f64_ulp("coherence_z", ms.coherence_z, deser.coherence_z);
}

/// The same tolerance holds for values the engine actually computed rather
/// than ones chosen by hand: every per-observation score the sentinel
/// produced, raw and standardised on all four axes, returns within a unit in
/// the last place. Real scores are arbitrary bit patterns rather than tidy
/// decimals, which is the case a text encoding is most likely to round.
///
/// (´claim:serde:a-float-returns-within-one-unit-in-the-last-place-of-the-value-that-was-sent´)
/// ´test:integration:sample-score-json-round-trip´
#[test]
fn sample_score_json_round_trip() {
    let report = rich_report();

    // Extract SampleScores from the first cell with per-sample data.
    let samples = report
        .cell_reports
        .iter()
        .find_map(|cr| cr.per_sample.as_ref())
        .expect("per_sample_scores is enabled; at least one cell should have samples");

    for ss in samples {
        let json = serde_json::to_string(ss).unwrap();
        let deser: SampleScore = serde_json::from_str(&json).unwrap();

        assert_f64_ulp("novelty", ss.novelty, deser.novelty);
        assert_f64_ulp("displacement", ss.displacement, deser.displacement);
        assert_f64_ulp("surprise", ss.surprise, deser.surprise);
        assert_f64_ulp("coherence", ss.coherence, deser.coherence);
        assert_f64_ulp("novelty_z", ss.novelty_z, deser.novelty_z);
        assert_f64_ulp("displacement_z", ss.displacement_z, deser.displacement_z);
        assert_f64_ulp("surprise_z", ss.surprise_z, deser.surprise_z);
        assert_f64_ulp("coherence_z", ss.coherence_z, deser.coherence_z);
    }
}
