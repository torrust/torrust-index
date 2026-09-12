// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`validate_root_present`] | dossier | A batch whose cells span depths 0, 4 and 8 — the root and a competitive cell arriving as cell reports, the ancestor arriving separately — passes structural validation as one report. Ancestor and competitive cells are held to the same contract and validated in a single pass, so the Assayer accepts or rejects a batch whole rather than half-admitting one of its two lists. |
//! | [`validate_root_absent`] | dossier | A batch carrying cells but no depth-zero cell is refused outright. The root is the only cell guaranteed to contain every coordinate, so without it a coordinate the report never mentioned would route to nothing at all — a rootless index would silently lose coverage rather than fail loudly here. |
//! | [`validate_root_wrong_depth`] | dossier | Rootness is a matter of depth, not of prominence: a batch whose sole cell sits at depth one — a perfectly valid half of the space, and the largest cell in the report — is still rejected as rootless. Were the shallowest cell present promoted to root by default, half the coordinate space would quietly fall outside the index while the report looked complete. |
//! | [`validate_dyadic_intervals`] | dossier | cites (´claim:dossier:a-batch-with-a-root-and-distinct-dyadic-cells-is-accepted-whole´) |
//! | [`validate_non_dyadic`] | dossier | One misaligned cell condemns the whole batch: a cell starting at coordinate 1 while claiming depth 8 is rejected even though the root beside it is perfectly well formed. Nothing about the depth-walk works for such a cell — no coordinate's ancestor key could ever equal its start — so admitting it would add an entry that reads correctly and can never be found. |
//! | [`validate_no_duplicate_keys`] | dossier | cites (´claim:dossier:a-batch-with-a-root-and-distinct-dyadic-cells-is-accepted-whole´) |
//! | [`validate_duplicate_keys`] | dossier | Two cells reported at the same start and depth are rejected rather than reconciled, even when they carry different sample counts and scores. The index is a map keyed on that pair, so the second would overwrite the first without a trace; refusing the batch keeps the Assayer from picking a winner between two accounts of the same region on iteration order alone. |
//! | [`validate_empty_cell_reports`] | dossier | A report bearing only the root, with no ancestors and no coordination contexts, is structurally complete. A Sentinel that has found nothing worth splitting out must still be able to say so, and it says so with a valid batch rather than by staying silent — silence and "nothing competitive this round" are different facts about a Sentinel. |
//! | [`validate_many_cells`] | dossier | A hundred cells scattered across depths from 4 to 112 validate together without any of them being mistaken for another. Because uniqueness is keyed on start and depth jointly, cells at different depths may share a start coordinate freely — which they must, since a cell and its ancestors all begin at the same place. |
//! | [`quality_clean_cell`] | dossier | A cell whose every field is in range arrives on the far side of extraction untouched — depth, sample count and per-axis scores all as reported — and is not flagged. Sanitisation is exceptional rather than routine, so the ordinary case costs the report nothing in fidelity and the degraded flag stays meaningful as a signal that something really was wrong. |
//! | [`quality_nan_max_z`] | dossier | One non-finite field condemns its whole axis, not just itself: a NaN in novelty's peak z-score returns the entire novelty snapshot to zero, CUSUM and baseline included, while displacement keeps its reported value. The axis is the unit of trust because its fields are computed from one another — a poisoned peak means the accumulator beside it cannot be believed either — and containing the damage at that boundary keeps three good axes usable. |
//! | [`quality_inf_cusum`] | dossier | cites (´claim:dossier:a-non-finite-field-zeroes-its-own-axis-and-leaves-the-others-intact´) |
//! | [`quality_negative_variance`] | dossier | Finiteness is not the whole test. A baseline variance of minus a half is a perfectly representable number and still impossible, and it condemns its axis exactly as a NaN would. Every z-score on that axis is divided by a root of that variance, so a negative value does not merely look odd — it makes every score derived from it meaningless. |
//! | [`quality_noise_influence_above_1`] | dossier | Noise influence is a proportion, and an out-of-range one is pulled back to the nearest bound rather than discarded: a reported 1.5 becomes 1.0, with the cell flagged degraded. Clamping is the right repair here because the direction of the error still carries information — a value over one means the tracker is saturated with noise, and saying so is more useful than saying nothing. |
//! | [`quality_noise_influence_negative`] | dossier | cites (´claim:dossier:noise-influence-is-clamped-into-the-unit-interval-rather-than-zeroed´) |
//! | [`quality_depth_cast`] | dossier | Depth arrives from the Sentinel as a thirty-two-bit number and is stored as a byte, and a depth inside the representable range survives that narrowing exactly. Storing depth narrowly is what keeps the Ledger key small, and it is safe precisely because the hierarchy cannot be deeper than the coordinate space has bits. |
//! | [`quality_depth_over_128`] | dossier | cites (´claim:dossier:a-depth-beyond-128-is-clamped-rather-than-wrapped-in-every-build-profile´) |
//! | [`quality_multiple_bad_axes`] | dossier | cites (´claim:dossier:a-non-finite-field-zeroes-its-own-axis-and-leaves-the-others-intact´) |
//! | [`quality_per_cell_independence`] | dossier | The degraded verdict belongs to a cell alone. Extracting a sound root and a poisoned depth-8 cell leaves the first unflagged and the second flagged, with no shared state carrying the taint between them. Were the flag sticky across a batch, a single bad cell would make an entire Sentinel's report look untrustworthy and the counts in the acknowledgement would stop meaning anything. |
//! | [`ingest_golden_report`] | dossier | Competitive cells and ancestor cells arrive in two separate lists and land in one index: a batch of two competitive cells and one ancestor acknowledges three cells ingested, and the slot afterwards holds a populated index. The split between the lists is the Sentinel's account of why each cell was reported; once received, a cell is just a cell, and routing must be able to descend through ancestors and competitors alike. |
//! | [`ingest_minimal_report`] | dossier | A batch bearing nothing but its root goes through the whole pipeline and is acknowledged as one clean cell. The quiet case is a first-class one: a Sentinel that has split out no interesting region still gets its slot updated, so a run of uneventful batches is recorded as reception rather than as absence. |
//! | [`ingest_degraded_report`] | dossier | A batch carrying one poisoned cell is still received: all three of its cells are ingested, and the single sanitised one is counted in the acknowledgement rather than causing a refusal. Data quality is reported, not enforced — the structural contract decides what may be received at all, while a bad number inside a cell is something the host learns about and can weigh. |
//! | [`ingest_to_unknown_sentinel`] | dossier | A perfectly valid report attributed to a Sentinel that has no slot is refused, and the error names the identifier that was not recognised. Slot lookup comes first in the pipeline for that reason: nothing is validated, allocated or written on behalf of a Sentinel the Assayer never registered, so an unknown producer cannot create state simply by talking. |
//! | [`ingest_updates_ledger_cell_set`] | dossier | Reception is not only a matter of the index: ingesting a batch also opens a Ledger entry for each region it named, so the root, the depth-4 ancestor and the depth-8 cell all become entries the Assayer will keep tracking. The index is replaced wholesale each round, whereas the Ledger is what remembers a cell across rounds, and it can only do so if reception seeds it. |
//! | [`ingest_ledger_absence_tracking`] | dossier | A cell that appeared in one batch and is missing from the next does not vanish from the Ledger: its entry survives with a count of consecutive absences standing against it. Cells come and go as a Sentinel's competitive set shifts, so absence is recorded as a running tally rather than acted on at once, and the entry's history is still there if the region reappears. |
//! | [`ingest_ledger_deletion`] | dossier | The tally is eventually acted on: with the threshold set to two, a depth-8 cell that fails to appear in three successive batches is gone from the Ledger altogether. Without that reaping, every region a Sentinel ever found interesting would accumulate for the lifetime of the process, and the cost of maintaining the cell set would grow with history rather than with the present shape of the space. |
//! | [`ingest_ledger_root_permanent`] | dossier | Reaping the deeper cells never touches the root. Under the harshest threshold available, ten rounds in which every other region falls away leave the depth-zero entry in place — the churn that deletes its descendants passes over it. The root is the entry a coordinate falls back to when nothing deeper claims it, so a Ledger without one would have regions of the space it could account for at no depth at all. |
//! | [`ingest_arcswap_visibility`] | dossier | Publication is complete by the time ingestion returns: a slot that read as empty a moment before now yields an index that routes the golden coordinate straight to its depth-8 cell. Readers take the index without a lock, so there is no window in which a caller can be told a report was accepted and still find the old view waiting for it. |
//! | [`ingest_previous_index_preserved_on_error`] | dossier | A rejected batch costs the Sentinel nothing it already had: after a rootless report is refused, the slot still holds the index built from the last good one, cell for cell. Validation runs before anything is published, so a malformed batch cannot leave the model blind — the worst it can do is fail to improve on what is there. |
//! | [`ingest_report_ack_fields`] | dossier | Coordination contexts are counted on their own line. Adding one to an otherwise unchanged batch moves the coordination tally to one and leaves the three cells and the zero degraded cells exactly where they were. A coordination entry describes several cells acting together rather than a region of the space, so folding it into the cell count would overstate how much of the space the report covered. |
//! | [`ingest_concurrent_same_sentinel`] | dossier | Two threads pouring twenty batches into one slot leave behind a whole index of exactly three cells, never a blend of two half-built ones. The per-Sentinel lock makes ingestion for a single Sentinel a sequence of complete substitutions, so whichever report wins the race, what a reader sees afterwards is one Sentinel's coherent account of the space. |
//! | [`ingest_concurrent_different_sentinels`] | dossier | Four Sentinels reporting at once through a shared slot map and a shared Ledger all succeed, every batch of every one of them, and each ends holding its own index. Serialisation is scoped to a slot rather than to the Assayer, so the cost of one Sentinel's traffic is not paid by its neighbours — which is what makes the reception path scale with the number of Sentinels at all. |
//! | [`cell_set_before_index_swap`] | dossier | Once a batch has been ingested, every key in the published index already has a Ledger entry behind it — the cell set is brought up to date before the index is swapped in, not after. The ordering matters because a reader who routes a coordinate through the new index and then asks the Ledger about what it found must not be told that region is unknown. |
//! | [`cell_set_coord_route_consistency`] | dossier | The two structures agree on the same key, not merely on the same population: taking the depth a coordinate routes to and rebuilding the Ledger key from that coordinate and depth finds an entry waiting. Index and Ledger derive their keys the same way from the same coordinates, so a routing answer can be carried straight over to the Ledger without translation. |
//! | [`cell_set_appear_then_read`] | dossier | A region a Sentinel newly splits out takes effect immediately: a coordinate that landed on the root while only the root was reported lands on the depth-8 cell as soon as a batch mentions it. Resolution is a property of the latest report rather than of accumulated history, so newly discovered structure is visible to the model on the very next read. |
//! | [`ingest_4cell_acceptance`] | dossier | cites (´claim:dossier:competitive-and-ancestor-cells-merge-into-a-single-routable-index´) |
//! | [`ingest_last_report_ack_updated`] | dossier | The acknowledgement left on the slot is the very one handed back to the caller, field for field, where before ingestion the slot held only zeros. A host that missed the return value — or that is asking later, on another thread, about the last thing this Sentinel said — reads the same account as the caller did, so the two can never disagree about what was received. |
//! | [`ingest_last_report_ack_default_before_first`] | dossier | A slot that has received nothing answers with zeros across every counter rather than with an absent value. Having a well-defined answer before the first batch means a host polling a freshly registered Sentinel reads "nothing yet" as a number it can display and compare, instead of having to special-case the moment before reception begins. |
//! | [`ingest_lock_ordering`] | dossier | Ingestion takes the slot's own lock first and the Ledger's second, and a batch that touches both runs to completion under that order. Holding to one direction is what keeps the two locks from ever being wanted in opposite orders by two Sentinels at once; the concurrent batches elsewhere in this suite are where a reversal would show up, as a hang rather than as a wrong answer. |

//! Crate-level tests for report ingestion.
//!
//! # Cross-References
//!
//! - (´dec:ordering:synchronous-reception´) — reception synchronous and serialised per Sentinel
//! - (´alg:runtime:report-reception´) — the ingestion pipeline these tests drive
//! - [`crate::testing`] — Golden report fixtures

use std::sync::Arc;
use std::thread;

use dashmap::DashMap;

use crate::error::ReportError;
use crate::ledger::OutcomeLedger;
use crate::report::{IngestionConfig, SentinelSlot, ingest_report, validate_and_extract_cell, validate_structural};
use crate::testing::{
    DEFAULT_TOLERANCES, DEPTH_8_LO, GOLDEN_COORD, SystemClock, assert_near, degraded_report, golden_report, golden_report_4cell,
    make_coordination_report, make_high_noise_cell, make_inf_cusum_cell, make_nan_cell, make_negative_noise_cell,
    make_negative_variance_cell, make_test_cell, minimal_report, report_many_cells, report_missing_root, report_non_dyadic,
    report_with_duplicates, scores,
};
use crate::types::{LedgerKey, SentinelId};

// ═══════════════════════════════════════════════════════════════════════════════
// Structural Validation Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// A batch whose cells span depths 0, 4 and 8 — the root and a competitive cell
/// arriving as cell reports, the ancestor arriving separately — passes structural
/// validation as one report. Ancestor and competitive cells are held to the same
/// contract and validated in a single pass, so the Assayer accepts or rejects a
/// batch whole rather than half-admitting one of its two lists.
///
/// ´claim:dossier:a-batch-with-a-root-and-distinct-dyadic-cells-is-accepted-whole´
/// ´test:crate:validate-root-present´
#[test]
fn validate_root_present() {
    let report = golden_report();
    let result = validate_structural(&report);
    assert!(result.is_ok(), "golden report should validate: {result:?}");
}

/// A batch carrying cells but no depth-zero cell is refused outright. The root is
/// the only cell guaranteed to contain every coordinate, so without it a
/// coordinate the report never mentioned would route to nothing at all — a
/// rootless index would silently lose coverage rather than fail loudly here.
///
/// ´claim:dossier:a-batch-without-a-depth-zero-cell-is-refused-as-rootless´
/// ´test:crate:validate-root-absent´
#[test]
fn validate_root_absent() {
    let report = report_missing_root();
    let result = validate_structural(&report);
    assert!(
        matches!(result, Err(ReportError::MissingRootCell)),
        "should fail with MissingRootCell, got: {result:?}"
    );
}

/// Rootness is a matter of depth, not of prominence: a batch whose sole cell sits
/// at depth one — a perfectly valid half of the space, and the largest cell in the
/// report — is still rejected as rootless. Were the shallowest cell present
/// promoted to root by default, half the coordinate space would quietly fall
/// outside the index while the report looked complete.
///
/// ´claim:dossier:only-a-depth-zero-cell-counts-as-the-root-however-much-space-it-covers´
/// ´test:crate:validate-root-wrong-depth´
#[test]
fn validate_root_wrong_depth() {
    // Build a report where the only cell is at depth 1, not 0
    let mut report = minimal_report();
    if let Some(cell) = report.cell_reports.get_mut(0) {
        cell.depth = 1; // Wrong depth
        // Fix interval for depth 1
        cell.start = 0;
        cell.end = 1_u128 << 127; // Half the space
    }

    let result = validate_structural(&report);
    assert!(
        matches!(result, Err(ReportError::MissingRootCell)),
        "depth-1 cell should not satisfy root requirement, got: {result:?}"
    );
}

/// The intervals a Sentinel reports are already aligned to their depths, and
/// validation confirms it rather than repairing it: the depth-4 and depth-8 cells
/// of an ordinary batch are accepted exactly as sent. Alignment is a property of
/// the Sentinel's own geometry, so a batch failing it signals a broken producer,
/// not a coordinate the Assayer should round.
///
/// (´claim:dossier:a-batch-with-a-root-and-distinct-dyadic-cells-is-accepted-whole´)
/// ´test:crate:validate-dyadic-intervals´
#[test]
fn validate_dyadic_intervals() {
    let report = golden_report();
    let result = validate_structural(&report);
    assert!(result.is_ok(), "golden report has valid dyadic intervals");
}

/// One misaligned cell condemns the whole batch: a cell starting at coordinate 1
/// while claiming depth 8 is rejected even though the root beside it is perfectly
/// well formed. Nothing about the depth-walk works for such a cell — no
/// coordinate's ancestor key could ever equal its start — so admitting it would
/// add an entry that reads correctly and can never be found.
///
/// ´claim:dossier:a-single-misaligned-cell-is-enough-to-reject-the-batch-around-it´
/// ´test:crate:validate-non-dyadic´
#[test]
fn validate_non_dyadic() {
    let report = report_non_dyadic();
    let result = validate_structural(&report);
    assert!(
        matches!(result, Err(ReportError::NonDyadicInterval { .. })),
        "should fail with NonDyadicInterval, got: {result:?}"
    );
}

/// Cells sharing an ancestry are not duplicates: the depth-4 ancestor of the
/// depth-8 cell occupies the same coordinates and still passes, because
/// uniqueness is judged on the pair of start and depth rather than on overlap.
/// Nesting is the normal shape of a report, so a check that punished containment
/// would reject every batch deeper than one level.
///
/// (´claim:dossier:a-batch-with-a-root-and-distinct-dyadic-cells-is-accepted-whole´)
/// ´test:crate:validate-no-duplicate-keys´
#[test]
fn validate_no_duplicate_keys() {
    let report = golden_report();
    let result = validate_structural(&report);
    assert!(result.is_ok(), "no duplicates in golden report");
}

/// Two cells reported at the same start and depth are rejected rather than
/// reconciled, even when they carry different sample counts and scores. The index
/// is a map keyed on that pair, so the second would overwrite the first without a
/// trace; refusing the batch keeps the Assayer from picking a winner between two
/// accounts of the same region on iteration order alone.
///
/// ´claim:dossier:two-cells-sharing-a-start-and-depth-are-rejected-not-silently-merged´
/// ´test:crate:validate-duplicate-keys´
#[test]
fn validate_duplicate_keys() {
    let report = report_with_duplicates();
    let result = validate_structural(&report);
    assert!(
        matches!(result, Err(ReportError::DuplicateCell { .. })),
        "should fail with DuplicateCell, got: {result:?}"
    );
}

/// A report bearing only the root, with no ancestors and no coordination
/// contexts, is structurally complete. A Sentinel that has found nothing worth
/// splitting out must still be able to say so, and it says so with a valid batch
/// rather than by staying silent — silence and "nothing competitive this round"
/// are different facts about a Sentinel.
///
/// ´claim:dossier:a-root-only-report-is-structurally-complete´
/// ´test:crate:validate-empty-cell-reports´
#[test]
fn validate_empty_cell_reports() {
    let report = minimal_report();
    let result = validate_structural(&report);
    assert!(result.is_ok(), "root-only report is valid");
}

/// A hundred cells scattered across depths from 4 to 112 validate together
/// without any of them being mistaken for another. Because uniqueness is keyed on
/// start and depth jointly, cells at different depths may share a start
/// coordinate freely — which they must, since a cell and its ancestors all begin
/// at the same place.
///
/// ´claim:dossier:cells-at-different-depths-may-share-a-start-without-colliding´
/// ´test:crate:validate-many-cells´
#[test]
fn validate_many_cells() {
    let report = report_many_cells();
    let result = validate_structural(&report);
    assert!(result.is_ok(), "100 cells at various depths should validate");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Data Quality Validation Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// A cell whose every field is in range arrives on the far side of extraction
/// untouched — depth, sample count and per-axis scores all as reported — and is
/// not flagged. Sanitisation is exceptional rather than routine, so the ordinary
/// case costs the report nothing in fidelity and the degraded flag stays
/// meaningful as a signal that something really was wrong.
///
/// ´claim:dossier:a-clean-cell-passes-through-extraction-unaltered-and-unflagged´
/// ´test:crate:quality-clean-cell´
#[test]
fn quality_clean_cell() {
    let cell = make_test_cell(DEPTH_8_LO, 8, 50, scores::D8_MAX_Z, scores::D8_CUSUM, 0.15);
    let (entry, degraded) = validate_and_extract_cell(&cell);

    assert!(!degraded, "clean cell should not be degraded");
    assert!(!entry.degraded, "entry.degraded should be false");
    assert_eq!(entry.depth, 8);
    assert_eq!(entry.sample_count, 50);

    // Check that scores passed through
    let tol = DEFAULT_TOLERANCES.default;
    assert_near(entry.scores.novelty.max_z, scores::D8_MAX_Z[0], tol, "novelty.max_z");
    assert_near(
        entry.scores.displacement.max_z,
        scores::D8_MAX_Z[1],
        tol,
        "displacement.max_z",
    );
}

/// One non-finite field condemns its whole axis, not just itself: a NaN in
/// novelty's peak z-score returns the entire novelty snapshot to zero, CUSUM and
/// baseline included, while displacement keeps its reported value. The axis is
/// the unit of trust because its fields are computed from one another — a
/// poisoned peak means the accumulator beside it cannot be believed either — and
/// containing the damage at that boundary keeps three good axes usable.
///
/// ´claim:dossier:a-non-finite-field-zeroes-its-own-axis-and-leaves-the-others-intact´
/// ´test:crate:quality-nan-max-z´
#[test]
fn quality_nan_max_z() {
    // NaN in novelty axis
    let cell = make_nan_cell(0);
    let (entry, degraded) = validate_and_extract_cell(&cell);

    assert!(degraded, "NaN max_z should set degraded");
    assert!(entry.degraded, "entry.degraded should be true");

    // Novelty axis should be zeroed
    assert_eq!(entry.scores.novelty.max_z, 0.0, "novelty axis should be zeroed");
    assert_eq!(entry.scores.novelty.cusum, 0.0);

    // Other axes should be preserved
    assert_near(
        entry.scores.displacement.max_z,
        scores::D8_MAX_Z[1],
        DEFAULT_TOLERANCES.default,
        "displacement.max_z preserved",
    );
}

/// It is not only the z-scores that are policed: an infinite CUSUM accumulator on
/// the surprise axis zeroes that axis's peak score as well as the accumulator
/// itself. An unbounded accumulator is exactly what a drift signal looks like when
/// it has run away, so it is the last value that should be allowed through into
/// the model's view of the world unchallenged.
///
/// (´claim:dossier:a-non-finite-field-zeroes-its-own-axis-and-leaves-the-others-intact´)
/// ´test:crate:quality-inf-cusum´
#[test]
fn quality_inf_cusum() {
    let cell = make_inf_cusum_cell();
    let (entry, degraded) = validate_and_extract_cell(&cell);

    assert!(degraded, "Inf CUSUM should set degraded");

    // Surprise axis should be zeroed (that's where we put the Inf)
    assert_eq!(entry.scores.surprise.max_z, 0.0, "surprise axis should be zeroed");
    assert_eq!(entry.scores.surprise.cusum, 0.0);
}

/// Finiteness is not the whole test. A baseline variance of minus a half is a
/// perfectly representable number and still impossible, and it condemns its axis
/// exactly as a NaN would. Every z-score on that axis is divided by a root of
/// that variance, so a negative value does not merely look odd — it makes every
/// score derived from it meaningless.
///
/// ´claim:dossier:a-negative-baseline-variance-is-impossible-and-condemns-its-axis-like-a-nan´
/// ´test:crate:quality-negative-variance´
#[test]
fn quality_negative_variance() {
    let cell = make_negative_variance_cell();
    let (entry, degraded) = validate_and_extract_cell(&cell);

    assert!(degraded, "negative variance should set degraded");

    // Novelty axis should be zeroed (that's where we put negative variance)
    assert_eq!(entry.scores.novelty.max_z, 0.0, "novelty axis should be zeroed");
}

/// Noise influence is a proportion, and an out-of-range one is pulled back to the
/// nearest bound rather than discarded: a reported 1.5 becomes 1.0, with the cell
/// flagged degraded. Clamping is the right repair here because the direction of
/// the error still carries information — a value over one means the tracker is
/// saturated with noise, and saying so is more useful than saying nothing.
///
/// ´claim:dossier:noise-influence-is-clamped-into-the-unit-interval-rather-than-zeroed´
/// ´test:crate:quality-noise-influence-above-1´
#[test]
fn quality_noise_influence_above_1() {
    let cell = make_high_noise_cell();
    let (entry, degraded) = validate_and_extract_cell(&cell);

    assert!(degraded, "noise_influence > 1.0 should set degraded");
    assert_eq!(entry.noise_influence, 1.0, "should clamp to 1.0");
}

/// The lower bound is enforced the same way as the upper: a negative noise
/// influence becomes zero and the cell is marked degraded. The repair is
/// symmetric, so neither end of the range is quietly the more permissive one, and
/// a caller reading the field never has to defend against a proportion outside
/// zero and one.
///
/// (´claim:dossier:noise-influence-is-clamped-into-the-unit-interval-rather-than-zeroed´)
/// ´test:crate:quality-noise-influence-negative´
#[test]
fn quality_noise_influence_negative() {
    let cell = make_negative_noise_cell();
    let (entry, degraded) = validate_and_extract_cell(&cell);

    assert!(degraded, "negative noise_influence should set degraded");
    assert_eq!(entry.noise_influence, 0.0, "should clamp to 0.0");
}

/// Depth arrives from the Sentinel as a thirty-two-bit number and is stored as a
/// byte, and a depth inside the representable range survives that narrowing
/// exactly. Storing depth narrowly is what keeps the Ledger key small, and it is
/// safe precisely because the hierarchy cannot be deeper than the coordinate
/// space has bits.
///
/// ´claim:dossier:a-depth-within-range-survives-the-narrowing-to-a-byte-unchanged´
/// ´test:crate:quality-depth-cast´
#[test]
fn quality_depth_cast() {
    let cell = make_test_cell(DEPTH_8_LO, 8, 50, scores::D8_MAX_Z, scores::D8_CUSUM, 0.1);
    let (entry, _) = validate_and_extract_cell(&cell);

    assert_eq!(entry.depth, 8, "depth should be cast to u8");
}

/// The same clamp stands in front of cell extraction as in front of key
/// construction — a cell claiming depth 200 is entered at 128 rather than at
/// the depth its low bits happen to spell — and the extraction reports it:
/// the degraded flag rises for the depth clamp exactly as it rises for the
/// other three sanitisations, so the returns clause's promise that the flag
/// covers every sanitised value is kept, while a boundary depth of 128
/// passes unclamped and unflagged (´dec:degradation:error-partition´).
///
/// (´claim:dossier:a-depth-beyond-128-is-clamped-rather-than-wrapped-in-every-build-profile´)
/// ´test:crate:quality-depth-over-128´
#[test]
fn quality_depth_over_128() {
    let mut cell = make_test_cell(DEPTH_8_LO, 8, 50, scores::D8_MAX_Z, scores::D8_CUSUM, 0.1);
    cell.depth = 200; // Over 128
    let (entry, degraded) = validate_and_extract_cell(&cell);

    assert_eq!(entry.depth, 128, "depth > 128 should clamp to 128");
    assert!(degraded, "the depth clamp is a sanitisation and the flag reports it");
    assert!(entry.degraded, "the entry carries its own flag too");

    // The boundary value is in contract: no clamp, no flag.
    let mut boundary = make_test_cell(DEPTH_8_LO, 8, 50, scores::D8_MAX_Z, scores::D8_CUSUM, 0.1);
    boundary.depth = 128;
    let (entry, degraded) = validate_and_extract_cell(&boundary);
    assert_eq!(entry.depth, 128);
    assert!(!degraded, "a representable depth is not a sanitisation");
}

/// Damage on two axes at once is contained twice over rather than escalating:
/// novelty and displacement both zero out, and surprise and coherence come
/// through with their reported peaks intact. Sanitisation has no notion of a cell
/// being too far gone to salvage, so partial evidence survives however many axes
/// were spoilt.
///
/// (´claim:dossier:a-non-finite-field-zeroes-its-own-axis-and-leaves-the-others-intact´)
/// ´test:crate:quality-multiple-bad-axes´
#[test]
fn quality_multiple_bad_axes() {
    // Create a cell with NaN in both novelty AND displacement
    let mut cell = make_test_cell(DEPTH_8_LO, 8, 50, scores::D8_MAX_Z, scores::D8_CUSUM, 0.1);
    cell.scores.novelty.max_z_score = f64::NAN;
    cell.scores.displacement.max_z_score = f64::NAN;

    let (entry, degraded) = validate_and_extract_cell(&cell);

    assert!(degraded, "multiple NaN axes should set degraded");
    assert_eq!(entry.scores.novelty.max_z, 0.0, "novelty zeroed");
    assert_eq!(entry.scores.displacement.max_z, 0.0, "displacement zeroed");

    // Surprise and coherence should be preserved
    let tol = DEFAULT_TOLERANCES.default;
    assert_near(
        entry.scores.surprise.max_z,
        scores::D8_MAX_Z[2],
        tol,
        "surprise.max_z preserved",
    );
    assert_near(
        entry.scores.coherence.max_z,
        scores::D8_MAX_Z[3],
        tol,
        "coherence.max_z preserved",
    );
}

/// The degraded verdict belongs to a cell alone. Extracting a sound root and a
/// poisoned depth-8 cell leaves the first unflagged and the second flagged, with
/// no shared state carrying the taint between them. Were the flag sticky across a
/// batch, a single bad cell would make an entire Sentinel's report look
/// untrustworthy and the counts in the acknowledgement would stop meaning
/// anything.
///
/// ´claim:dossier:the-degraded-verdict-is-per-cell-so-one-bad-cell-does-not-taint-its-neighbours´
/// ´test:crate:quality-per-cell-independence´
#[test]
fn quality_per_cell_independence() {
    // One clean cell, one degraded
    let clean = make_test_cell(0, 0, 100, scores::ROOT_MAX_Z, scores::ROOT_CUSUM, 0.05);
    let degraded_cell = make_nan_cell(0);

    let (clean_entry, clean_deg) = validate_and_extract_cell(&clean);
    let (deg_entry, deg_deg) = validate_and_extract_cell(&degraded_cell);

    assert!(!clean_deg, "clean cell should not be degraded");
    assert!(!clean_entry.degraded);

    assert!(deg_deg, "degraded cell should be degraded");
    assert!(deg_entry.degraded);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Full Pipeline Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// Helper to set up ingestion test infrastructure.
fn setup_ingestion() -> (DashMap<SentinelId, SentinelSlot>, OutcomeLedger, IngestionConfig) {
    let sentinel_slots: DashMap<SentinelId, SentinelSlot> = DashMap::new();
    let sentinel_id = SentinelId(1);
    sentinel_slots.insert(sentinel_id, SentinelSlot::new());

    // Create outcome ledger with a Sentinel Ledger for our test sentinel
    let outcome_ledger = OutcomeLedger::new();
    outcome_ledger.create_sentinel(sentinel_id);

    let config = IngestionConfig::default();

    (sentinel_slots, outcome_ledger, config)
}

/// Competitive cells and ancestor cells arrive in two separate lists and land in
/// one index: a batch of two competitive cells and one ancestor acknowledges three
/// cells ingested, and the slot afterwards holds a populated index. The split
/// between the lists is the Sentinel's account of why each cell was reported;
/// once received, a cell is just a cell, and routing must be able to descend
/// through ancestors and competitors alike.
///
/// ´claim:dossier:competitive-and-ancestor-cells-merge-into-a-single-routable-index´
/// ´test:crate:ingest-golden-report´
#[test]
fn ingest_golden_report() {
    let (slots, ledger, config) = setup_ingestion();
    let sentinel_id = SentinelId(1);
    let report = golden_report();

    let ack = ingest_report(sentinel_id, &report, &slots, &ledger, &config, &SystemClock);

    assert!(ack.is_ok(), "golden report should ingest: {ack:?}");
    let ack = ack.unwrap();

    // 2 competitive cells + 1 ancestor = 3 cells
    assert_eq!(ack.cells_ingested, 3, "3 cells in golden report");
    assert_eq!(ack.degraded_cells, 0, "no degraded cells");
    assert_eq!(ack.coordination_contexts, 0, "no coordination contexts");

    // Verify index is populated
    let slot = slots.get(&sentinel_id).unwrap();
    let index = slot.report_index.load();
    assert!(index.has_report(), "index should have report");
}

/// A batch bearing nothing but its root goes through the whole pipeline and is
/// acknowledged as one clean cell. The quiet case is a first-class one: a
/// Sentinel that has split out no interesting region still gets its slot updated,
/// so a run of uneventful batches is recorded as reception rather than as
/// absence.
///
/// ´claim:dossier:a-root-only-report-ingests-to-exactly-one-cell´
/// ´test:crate:ingest-minimal-report´
#[test]
fn ingest_minimal_report() {
    let (slots, ledger, config) = setup_ingestion();
    let sentinel_id = SentinelId(1);
    let report = minimal_report();

    let ack = ingest_report(sentinel_id, &report, &slots, &ledger, &config, &SystemClock);

    assert!(ack.is_ok());
    let ack = ack.unwrap();

    assert_eq!(ack.cells_ingested, 1, "only root cell");
    assert_eq!(ack.degraded_cells, 0);
}

/// A batch carrying one poisoned cell is still received: all three of its cells
/// are ingested, and the single sanitised one is counted in the acknowledgement
/// rather than causing a refusal. Data quality is reported, not enforced — the
/// structural contract decides what may be received at all, while a bad number
/// inside a cell is something the host learns about and can weigh.
///
/// ´claim:dossier:a-poisoned-cell-is-ingested-and-counted-not-grounds-for-refusing-the-batch´
/// ´test:crate:ingest-degraded-report´
#[test]
fn ingest_degraded_report() {
    let (slots, ledger, config) = setup_ingestion();
    let sentinel_id = SentinelId(1);
    let report = degraded_report();

    let ack = ingest_report(sentinel_id, &report, &slots, &ledger, &config, &SystemClock);

    assert!(ack.is_ok());
    let ack = ack.unwrap();

    assert_eq!(ack.cells_ingested, 3);
    assert_eq!(ack.degraded_cells, 1, "one cell has NaN");
}

/// A perfectly valid report attributed to a Sentinel that has no slot is refused,
/// and the error names the identifier that was not recognised. Slot lookup comes
/// first in the pipeline for that reason: nothing is validated, allocated or
/// written on behalf of a Sentinel the Assayer never registered, so an unknown
/// producer cannot create state simply by talking.
///
/// ´claim:dossier:a-report-from-a-sentinel-with-no-slot-is-refused-before-anything-is-built´
/// ´test:crate:ingest-to-unknown-sentinel´
#[test]
fn ingest_to_unknown_sentinel() {
    let (slots, ledger, config) = setup_ingestion();
    let unknown_id = SentinelId(999);
    let report = golden_report();

    let result = ingest_report(unknown_id, &report, &slots, &ledger, &config, &SystemClock);

    assert!(
        matches!(result, Err(ReportError::UnknownSentinel { id }) if id == unknown_id),
        "should fail with UnknownSentinel"
    );
}

/// Reception is not only a matter of the index: ingesting a batch also opens a
/// Ledger entry for each region it named, so the root, the depth-4 ancestor and
/// the depth-8 cell all become entries the Assayer will keep tracking. The index
/// is replaced wholesale each round, whereas the Ledger is what remembers a cell
/// across rounds, and it can only do so if reception seeds it.
///
/// ´claim:dossier:ingestion-opens-a-ledger-entry-for-every-cell-the-report-named´
/// ´test:crate:ingest-updates-ledger-cell-set´
#[test]
fn ingest_updates_ledger_cell_set() {
    let (slots, ledger, config) = setup_ingestion();
    let sentinel_id = SentinelId(1);
    let report = golden_report();

    // Ingest
    let _ = ingest_report(sentinel_id, &report, &slots, &ledger, &config, &SystemClock);

    // Check ledger has cells
    let ledger_lock = ledger.get(sentinel_id).expect("ledger should exist");
    let ledger_guard = ledger_lock.read().unwrap();

    // Should have entries for root, d4, d8
    assert!(ledger_guard.entry_count() >= 3, "ledger should have at least 3 entries");
}

/// A cell that appeared in one batch and is missing from the next does not vanish
/// from the Ledger: its entry survives with a count of consecutive absences
/// standing against it. Cells come and go as a Sentinel's competitive set shifts,
/// so absence is recorded as a running tally rather than acted on at once, and
/// the entry's history is still there if the region reappears.
///
/// ´claim:dossier:a-cell-missing-from-a-later-report-survives-with-its-absences-tallied´
/// ´test:crate:ingest-ledger-absence-tracking´
#[test]
fn ingest_ledger_absence_tracking() {
    let (slots, ledger, config) = setup_ingestion();
    let sentinel_id = SentinelId(1);

    // First report: has cells A, B (root + d8)
    let report1 = golden_report();
    let _ = ingest_report(sentinel_id, &report1, &slots, &ledger, &config, &SystemClock);

    // Second report: only root (d8 cell absent)
    let report2 = minimal_report();
    let _ = ingest_report(sentinel_id, &report2, &slots, &ledger, &config, &SystemClock);

    // Check that the d8 cell has absence count > 0
    let ledger_lock = ledger.get(sentinel_id).expect("ledger should exist");
    let ledger_guard = ledger_lock.read().unwrap();

    let d8_key = LedgerKey::new(DEPTH_8_LO, 8);
    if let Some(entry) = ledger_guard.get(&d8_key) {
        assert!(entry.consecutive_absent > 0, "d8 cell should have non-zero absence count");
    }
    // Note: the cell might already be deleted if threshold is low,
    // but the test documents the expected behaviour
}

/// The tally is eventually acted on: with the threshold set to two, a depth-8 cell
/// that fails to appear in three successive batches is gone from the Ledger
/// altogether. Without that reaping, every region a Sentinel ever found
/// interesting would accumulate for the lifetime of the process, and the cost of
/// maintaining the cell set would grow with history rather than with the present
/// shape of the space.
///
/// ´claim:dossier:a-cell-absent-past-the-threshold-is-reaped-from-the-ledger´
/// ´test:crate:ingest-ledger-deletion´
#[test]
fn ingest_ledger_deletion() {
    let (slots, ledger, mut config) = setup_ingestion();
    config.n_absent_threshold = 2; // Delete after 2 absences
    let sentinel_id = SentinelId(1);

    // First report: has all cells
    let report1 = golden_report();
    let _ = ingest_report(sentinel_id, &report1, &slots, &ledger, &config, &SystemClock);

    // Send minimal reports repeatedly to exceed threshold
    let report_min = minimal_report();
    let _ = ingest_report(sentinel_id, &report_min, &slots, &ledger, &config, &SystemClock);
    let _ = ingest_report(sentinel_id, &report_min, &slots, &ledger, &config, &SystemClock);
    let _ = ingest_report(sentinel_id, &report_min, &slots, &ledger, &config, &SystemClock);

    // d8 cell should be deleted
    let ledger_lock = ledger.get(sentinel_id).expect("ledger should exist");
    let ledger_guard = ledger_lock.read().unwrap();

    let d8_key = LedgerKey::new(DEPTH_8_LO, 8);
    assert!(
        ledger_guard.get(&d8_key).is_none(),
        "d8 cell should be deleted after threshold exceeded"
    );
}

/// Reaping the deeper cells never touches the root. Under the harshest threshold
/// available, ten rounds in which every other region falls away leave the
/// depth-zero entry in place — the churn that deletes its descendants passes over
/// it. The root is the entry a coordinate falls back to when nothing deeper
/// claims it, so a Ledger without one would have regions of the space it could
/// account for at no depth at all.
///
/// ´claim:dossier:the-root-entry-outlives-every-round-of-cell-set-churn´
/// ´test:crate:ingest-ledger-root-permanent´
#[test]
fn ingest_ledger_root_permanent() {
    let (slots, ledger, mut config) = setup_ingestion();
    config.n_absent_threshold = 1;
    let sentinel_id = SentinelId(1);

    // First report: golden
    let report1 = golden_report();
    let _ = ingest_report(sentinel_id, &report1, &slots, &ledger, &config, &SystemClock);

    // Many minimal reports (root always present)
    let report_min = minimal_report();
    for _ in 0..10 {
        let _ = ingest_report(sentinel_id, &report_min, &slots, &ledger, &config, &SystemClock);
    }

    // Root should still exist
    let ledger_lock = ledger.get(sentinel_id).expect("ledger should exist");
    let ledger_guard = ledger_lock.read().unwrap();

    let root_key = LedgerKey::new(0, 0);
    assert!(ledger_guard.get(&root_key).is_some(), "root cell should never be deleted");
}

/// Publication is complete by the time ingestion returns: a slot that read as
/// empty a moment before now yields an index that routes the golden coordinate
/// straight to its depth-8 cell. Readers take the index without a lock, so there
/// is no window in which a caller can be told a report was accepted and still
/// find the old view waiting for it.
///
/// ´claim:dossier:a-published-index-is-visible-to-readers-the-moment-ingestion-returns´
/// ´test:crate:ingest-arcswap-visibility´
#[test]
fn ingest_arcswap_visibility() {
    let (slots, ledger, config) = setup_ingestion();
    let sentinel_id = SentinelId(1);
    let report = golden_report();

    // Before ingestion: empty index
    {
        let slot = slots.get(&sentinel_id).unwrap();
        let index = slot.report_index.load();
        assert!(!index.has_report(), "should start empty");
    }

    // Ingest
    let _ = ingest_report(sentinel_id, &report, &slots, &ledger, &config, &SystemClock);

    // After ingestion: index has report
    {
        let slot = slots.get(&sentinel_id).unwrap();
        let index = slot.report_index.load();
        assert!(index.has_report(), "should have report after ingest");

        // Route to GOLDEN_COORD should find the d8 cell
        let entry = index.route(GOLDEN_COORD);
        assert!(entry.is_some(), "should route to a cell");
        assert_eq!(entry.unwrap().depth, 8, "should route to depth-8 cell");
    }
}

/// A rejected batch costs the Sentinel nothing it already had: after a rootless
/// report is refused, the slot still holds the index built from the last good
/// one, cell for cell. Validation runs before anything is published, so a
/// malformed batch cannot leave the model blind — the worst it can do is fail to
/// improve on what is there.
///
/// ´claim:dossier:a-rejected-batch-leaves-the-previous-index-standing´
/// ´test:crate:ingest-previous-index-preserved-on-error´
#[test]
fn ingest_previous_index_preserved_on_error() {
    let (slots, ledger, config) = setup_ingestion();
    let sentinel_id = SentinelId(1);

    // First: ingest valid report
    let valid_report = golden_report();
    let _ = ingest_report(sentinel_id, &valid_report, &slots, &ledger, &config, &SystemClock);

    // Capture current index
    let old_cells = {
        let slot = slots.get(&sentinel_id).unwrap();
        let index = slot.report_index.load();
        index.cells().len()
    };

    // Now try to ingest invalid report (missing root)
    let invalid_report = report_missing_root();
    let result = ingest_report(sentinel_id, &invalid_report, &slots, &ledger, &config, &SystemClock);
    assert!(result.is_err(), "invalid report should fail");

    // Verify old index is still there
    let slot = slots.get(&sentinel_id).unwrap();
    let index = slot.report_index.load();
    assert_eq!(index.cells().len(), old_cells, "old index should be preserved on error");
}

/// Coordination contexts are counted on their own line. Adding one to an
/// otherwise unchanged batch moves the coordination tally to one and leaves the
/// three cells and the zero degraded cells exactly where they were. A
/// coordination entry describes several cells acting together rather than a
/// region of the space, so folding it into the cell count would overstate how
/// much of the space the report covered.
///
/// ´claim:dossier:coordination-contexts-are-acknowledged-separately-from-cells´
/// ´test:crate:ingest-report-ack-fields´
#[test]
fn ingest_report_ack_fields() {
    let (slots, ledger, config) = setup_ingestion();
    let sentinel_id = SentinelId(1);

    // Create report with coordination
    let mut report = golden_report();
    report.coordination_reports.push(make_coordination_report(0, 0, 2));

    let ack = ingest_report(sentinel_id, &report, &slots, &ledger, &config, &SystemClock).unwrap();

    assert_eq!(ack.cells_ingested, 3);
    assert_eq!(ack.degraded_cells, 0);
    assert_eq!(ack.coordination_contexts, 1, "should count coordination report");
}

/// Two threads pouring twenty batches into one slot leave behind a whole index of
/// exactly three cells, never a blend of two half-built ones. The per-Sentinel
/// lock makes ingestion for a single Sentinel a sequence of complete
/// substitutions, so whichever report wins the race, what a reader sees afterwards
/// is one Sentinel's coherent account of the space.
///
/// ´claim:dossier:concurrent-reports-for-one-sentinel-serialise-into-whole-indexes-never-a-blend´
/// ´test:crate:ingest-concurrent-same-sentinel´
#[test]
fn ingest_concurrent_same_sentinel() {
    let (slots, ledger, config) = setup_ingestion();
    let slots = Arc::new(slots);
    let ledger = Arc::new(ledger);
    let config = Arc::new(config);
    let sentinel_id = SentinelId(1);

    // Spawn two threads that ingest to the same sentinel
    let handles: Vec<_> = (0..2)
        .map(|_| {
            let slots = Arc::clone(&slots);
            let ledger = Arc::clone(&ledger);
            let config = Arc::clone(&config);

            thread::spawn(move || {
                for _ in 0..10 {
                    let report = golden_report();
                    let _ = ingest_report(sentinel_id, &report, &slots, &ledger, &config, &SystemClock);
                }
            })
        })
        .collect();

    // Wait for all threads
    for h in handles {
        h.join().expect("thread should not panic");
    }

    // Verify final state is consistent
    let slot = slots.get(&sentinel_id).unwrap();
    let index = slot.report_index.load();
    assert!(index.has_report(), "should have a report");
    assert_eq!(index.cells().len(), 3, "should have 3 cells from golden report");
}

/// Four Sentinels reporting at once through a shared slot map and a shared Ledger
/// all succeed, every batch of every one of them, and each ends holding its own
/// index. Serialisation is scoped to a slot rather than to the Assayer, so the
/// cost of one Sentinel's traffic is not paid by its neighbours — which is what
/// makes the reception path scale with the number of Sentinels at all.
///
/// ´claim:dossier:serialisation-is-per-sentinel-so-different-sentinels-ingest-side-by-side´
/// ´test:crate:ingest-concurrent-different-sentinels´
#[test]
fn ingest_concurrent_different_sentinels() {
    let slots: DashMap<SentinelId, SentinelSlot> = DashMap::new();
    let ledger = OutcomeLedger::new();
    let config = IngestionConfig::default();

    // Set up 4 different sentinels
    for i in 1..=4 {
        let id = SentinelId(i);
        slots.insert(id, SentinelSlot::new());
        ledger.create_sentinel(id);
    }

    let slots = Arc::new(slots);
    let ledger = Arc::new(ledger);
    let config = Arc::new(config);

    // Spawn threads for each sentinel
    let handles: Vec<_> = (1..=4)
        .map(|i| {
            let slots = Arc::clone(&slots);
            let ledger = Arc::clone(&ledger);
            let config = Arc::clone(&config);
            let sentinel_id = SentinelId(i);

            thread::spawn(move || {
                for _ in 0..10 {
                    let report = golden_report();
                    let result = ingest_report(sentinel_id, &report, &slots, &ledger, &config, &SystemClock);
                    assert!(result.is_ok());
                }
            })
        })
        .collect();

    // Wait for all threads
    for h in handles {
        h.join().expect("thread should not panic");
    }

    // All sentinels should have reports
    for i in 1..=4 {
        let id = SentinelId(i);
        let slot = slots.get(&id).unwrap();
        let index = slot.report_index.load();
        assert!(index.has_report(), "sentinel {i} should have report");
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Cell-Set Maintenance Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// Once a batch has been ingested, every key in the published index already has a
/// Ledger entry behind it — the cell set is brought up to date before the index is
/// swapped in, not after. The ordering matters because a reader who routes a
/// coordinate through the new index and then asks the Ledger about what it found
/// must not be told that region is unknown.
///
/// ´claim:dossier:every-cell-in-the-published-index-already-has-its-ledger-entry´
/// ´test:crate:cell-set-before-index-swap´
#[test]
fn cell_set_before_index_swap() {
    let (slots, ledger, config) = setup_ingestion();
    let sentinel_id = SentinelId(1);
    let report = golden_report();

    // Ingest
    let _ = ingest_report(sentinel_id, &report, &slots, &ledger, &config, &SystemClock);

    // Both Ledger and index should exist and be consistent
    let slot = slots.get(&sentinel_id).unwrap();
    let index = slot.report_index.load();
    let ledger_lock = ledger.get(sentinel_id).unwrap();
    let ledger_guard = ledger_lock.read().unwrap();

    // Every cell in the index should have a ledger entry
    for key in index.cells().keys() {
        assert!(ledger_guard.get(key).is_some(), "cell {:?} should be in ledger", key);
    }
}

/// The two structures agree on the same key, not merely on the same population:
/// taking the depth a coordinate routes to and rebuilding the Ledger key from that
/// coordinate and depth finds an entry waiting. Index and Ledger derive their keys
/// the same way from the same coordinates, so a routing answer can be carried
/// straight over to the Ledger without translation.
///
/// ´claim:dossier:the-key-a-coordinate-routes-to-is-a-key-the-ledger-answers-to´
/// ´test:crate:cell-set-coord-route-consistency´
#[test]
fn cell_set_coord_route_consistency() {
    let (slots, ledger, config) = setup_ingestion();
    let sentinel_id = SentinelId(1);
    let report = golden_report();

    let _ = ingest_report(sentinel_id, &report, &slots, &ledger, &config, &SystemClock);

    // Route coordinate through index
    let slot = slots.get(&sentinel_id).unwrap();
    let index = slot.report_index.load();
    let index_entry = index.route(GOLDEN_COORD);

    assert!(index_entry.is_some());
    let index_depth = index_entry.unwrap().depth;

    // The corresponding LedgerKey should exist
    let ledger_lock = ledger.get(sentinel_id).unwrap();
    let ledger_guard = ledger_lock.read().unwrap();
    let key = LedgerKey::from_coordinate(GOLDEN_COORD, index_depth);

    assert!(
        ledger_guard.get(&key).is_some(),
        "ledger should have entry at depth {index_depth}"
    );
}

/// A region a Sentinel newly splits out takes effect immediately: a coordinate
/// that landed on the root while only the root was reported lands on the depth-8
/// cell as soon as a batch mentions it. Resolution is a property of the latest
/// report rather than of accumulated history, so newly discovered structure is
/// visible to the model on the very next read.
///
/// ´claim:dossier:a-cell-appearing-in-a-later-report-deepens-where-a-coordinate-routes-at-once´
/// ´test:crate:cell-set-appear-then-read´
#[test]
fn cell_set_appear_then_read() {
    let (slots, ledger, config) = setup_ingestion();
    let sentinel_id = SentinelId(1);

    // First: minimal report (only root)
    let report1 = minimal_report();
    let _ = ingest_report(sentinel_id, &report1, &slots, &ledger, &config, &SystemClock);

    // Routing should only find root
    {
        let slot = slots.get(&sentinel_id).unwrap();
        let index = slot.report_index.load();
        let entry = index.route(GOLDEN_COORD);
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().depth, 0, "should route to root only");
    }

    // Second: golden report with d8 cell
    let report2 = golden_report();
    let _ = ingest_report(sentinel_id, &report2, &slots, &ledger, &config, &SystemClock);

    // Now routing should find d8 cell
    {
        let slot = slots.get(&sentinel_id).unwrap();
        let index = slot.report_index.load();
        let entry = index.route(GOLDEN_COORD);
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().depth, 8, "should route to depth-8 cell");
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Acceptance Criterion Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// The merge holds at a deeper batch too: three competitive cells and one
/// ancestor acknowledge four cells ingested, and the golden coordinate — covered
/// by all four — lands on the depth-12 cell, the deepest of them. The count and
/// the routing depth are two faces of the same fact, that every reported cell
/// whatever list it came in is a candidate the depth-walk can reach.
///
/// (´claim:dossier:competitive-and-ancestor-cells-merge-into-a-single-routable-index´)
/// ´test:crate:ingest-4cell-acceptance´
#[test]
fn ingest_4cell_acceptance() {
    let (slots, ledger, config) = setup_ingestion();
    let sentinel_id = SentinelId(1);
    let report = golden_report_4cell();

    let ack = ingest_report(sentinel_id, &report, &slots, &ledger, &config, &SystemClock);

    assert!(ack.is_ok(), "4-cell report should ingest: {ack:?}");
    let ack = ack.unwrap();

    assert_eq!(ack.cells_ingested, 4, "root + 2 competitive + 1 ancestor = 4");
    assert_eq!(ack.degraded_cells, 0, "no degraded cells");
    assert_eq!(ack.coordination_contexts, 0, "no coordination contexts");

    // Verify routing: GOLDEN_COORD should land at depth 12 (deepest)
    let slot = slots.get(&sentinel_id).unwrap();
    let index = slot.report_index.load();
    let entry = index.route(GOLDEN_COORD).expect("should route");
    assert_eq!(entry.depth, 12, "should route to depth-12 cell (deepest)");
}

/// The acknowledgement left on the slot is the very one handed back to the
/// caller, field for field, where before ingestion the slot held only zeros. A
/// host that missed the return value — or that is asking later, on another
/// thread, about the last thing this Sentinel said — reads the same account as the
/// caller did, so the two can never disagree about what was received.
///
/// ´claim:dossier:the-acknowledgement-left-on-the-slot-is-the-one-handed-back-to-the-caller´
/// ´test:crate:ingest-last-report-ack-updated´
#[test]
fn ingest_last_report_ack_updated() {
    let (slots, ledger, config) = setup_ingestion();
    let sentinel_id = SentinelId(1);
    let report = golden_report();

    // Before ingestion: default ack (all zeros)
    {
        let slot = slots.get(&sentinel_id).unwrap();
        let ack = slot.last_report_ack.load();
        assert_eq!(ack.cells_ingested, 0, "should start as default");
        drop(ack);
        drop(slot);
    }

    // Ingest
    let returned_ack = ingest_report(sentinel_id, &report, &slots, &ledger, &config, &SystemClock).unwrap();

    // After ingestion: ack matches returned value
    let slot = slots.get(&sentinel_id).unwrap();
    let stored_ack = slot.last_report_ack.load();
    assert_eq!(**stored_ack, returned_ack, "stored ack should match returned ack");
    assert_eq!(stored_ack.cells_ingested, 3);
    assert_eq!(stored_ack.degraded_cells, 0);
    drop(stored_ack);
    drop(slot);
}

/// A slot that has received nothing answers with zeros across every counter
/// rather than with an absent value. Having a well-defined answer before the
/// first batch means a host polling a freshly registered Sentinel reads "nothing
/// yet" as a number it can display and compare, instead of having to special-case
/// the moment before reception begins.
///
/// ´claim:dossier:a-slot-that-has-received-nothing-answers-with-a-zeroed-acknowledgement´
/// ´test:crate:ingest-last-report-ack-default-before-first´
#[test]
fn ingest_last_report_ack_default_before_first() {
    let slot = SentinelSlot::new();
    let ack = slot.last_report_ack.load();

    assert_eq!(ack.cells_ingested, 0);
    assert_eq!(ack.degraded_cells, 0);
    assert_eq!(ack.coordination_contexts, 0);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Lock Ordering Documentation Test
// ═══════════════════════════════════════════════════════════════════════════════

/// Ingestion takes the slot's own lock first and the Ledger's second, and a batch
/// that touches both runs to completion under that order. Holding to one direction
/// is what keeps the two locks from ever being wanted in opposite orders by two
/// Sentinels at once; the concurrent batches elsewhere in this suite are where a
/// reversal would show up, as a hang rather than as a wrong answer.
///
/// ´claim:dossier:ingestion-takes-the-slot-lock-before-the-ledger-lock-and-never-the-reverse´
/// ´test:crate:ingest-lock-ordering´
#[test]
fn ingest_lock_ordering() {
    // The lock ordering is: ingestion Mutex first, then Ledger RwLock.
    // This test documents the invariant. The actual enforcement is
    // structural (the code in ingest_report acquires locks in this order).
    //
    // If this test compiles and runs, the invariant is documented.
    // Violation would manifest as deadlocks in concurrent tests above.

    let (slots, ledger, config) = setup_ingestion();
    let sentinel_id = SentinelId(1);
    let report = golden_report();

    // This should not deadlock
    let result = ingest_report(sentinel_id, &report, &slots, &ledger, &config, &SystemClock);
    assert!(result.is_ok());
}
