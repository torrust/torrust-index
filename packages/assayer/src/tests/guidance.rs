// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`request_labels_empty_budget_returns_empty`] | guidance | A budget of zero everywhere buys nothing, even with a strong candidate sitting in the buffer. The budget is the host's statement of how much labelling effort it has, and guidance never recommends more than was asked for — a host with no capacity this minute can call the query freely without receiving work it cannot do. |
//! | [`request_labels_ranks_seeded_pending_snapshot`] | guidance | All three categories rank the live pending buffer end to end: the more uncertain assessment leads the risk-informative list and the less uncertain one follows, the same assessment leads on anchor weight, and the one routed through a starved, divergent Ledger cell comes back under starvation relief carrying the score that cell's own divergence and label deficit produce. The categories are scored independently against real state, so guidance reflects what the system currently holds rather than a precomputed list. |
//! | [`request_labels_does_not_deduplicate_across_categories`] | guidance | cites (´claim:guidance:an-assessment-selected-by-several-categories-is-marked-in-each-with-the-others-but-not-itself´) |
//! | [`request_labels_respects_scan_limit`] | guidance | The scan limit bounds how much of the pending buffer guidance will look at, independently of how generous the budget is: a limit of one leaves a single candidate to rank however many are actually buffered. The buffer can hold enormous numbers of entries and the query runs on the caller's thread, so the host needs a knob that caps the work rather than only the output. |

//! Crate-level tests for Core label guidance (´sec:guidance:core´).
//!
//! # Cross-References
//!
//! - (´sec:guidance:core´) — the three criteria the query scores its candidates
//!   by: risk-informative, investigation, and starvation relief
//! - (´dec:surface:read-only-guidance´) — the read-only, infallible, synchronous
//!   query this surface is, which is why a budget of zero simply buys nothing

use std::collections::HashMap;
use std::time::Instant;

use crate::guidance::{LabelBudget, LabelCategory, LabelGuidanceParams};
use crate::health::DegradationContext;
use crate::ledger::{LedgerEntry, SentinelLedger};
use crate::pending::{PendingAssessment, PendingRiskBasis, SentinelExtraction};
use crate::types::{AssessmentId, EntityKey, LedgerKey, PersistentTimestamp, SentinelId};
use crate::{Assayer, AssayerConfig};

/// Sentinel used by the seeded starvation-relief fixtures.
const STARVED_SENTINEL: SentinelId = SentinelId(7);

/// Coordinate routed through the starved Ledger cell.
const STARVED_COORDINATE: u128 = 1;

/// Builds a small Assayer with a low eligible base rate for starvation tests.
fn guidance_assayer() -> Assayer {
    let mut config = AssayerConfig {
        instance_id: "guidance-test".to_owned(),
        // The capacity is host-set with no default; the fixture declares it.
        infrastructure: crate::testing::test_infrastructure(),
        ..Default::default()
    };
    config.model.p_plus_init = 0.05;
    Assayer::build(config).expect("no persistence configured")
}

/// Creates a pending assessment with optional Sentinel routing metadata.
fn pending_assessment(
    id: u64,
    uncertainty: f64,
    anchor_weight: f64,
    routed_sentinel: Option<(SentinelId, u128)>,
) -> PendingAssessment {
    let mut sentinel_extractions = HashMap::new();
    if let Some((sentinel_id, coordinate)) = routed_sentinel {
        sentinel_extractions.insert(sentinel_id, SentinelExtraction::new(coordinate, vec![1.0], true));
    }

    PendingAssessment {
        spatial_axis_ids: Vec::new(),
        id: AssessmentId(id),
        timestamp: Instant::now(),
        persistent_timestamp: PersistentTimestamp::now(),
        entity: EntityKey::new(id.to_le_bytes().to_vec()),
        sentinel_extractions,
        identity_coordinates: HashMap::new(),
        identity_active_cells: HashMap::new(),
        active_sentinels: Vec::new(),
        reporting_sentinels: Vec::new(),
        entity_base_features: HashMap::new(),
        entity_axis_features: HashMap::new(),
        signal_features: crate::pending::StoredFeatures::default(),
        risk_basis: PendingRiskBasis::new(0.5, uncertainty, anchor_weight, 0.0),
        outcome_predictions: HashMap::new(),
        degradation: DegradationContext::default(),
        report_origin: None,
    }
}

/// Installs one Ledger entry with starvation-relief score near `0.0985`.
fn install_starved_ledger(assayer: &Assayer) {
    let mut ledger = SentinelLedger::new();
    ledger.ensure_root();

    let mut entry = LedgerEntry::new_neutral();
    entry.ewma_bad_rate = 0.15;
    entry.recent_eligible_count = 3;
    ledger.insert(LedgerKey::new(0, 8), entry);

    assayer.outcome_ledger.create_sentinel(STARVED_SENTINEL);
    let ledger_lock = assayer
        .outcome_ledger
        .get_arc(STARVED_SENTINEL)
        .expect("created Sentinel ledger");
    *ledger_lock.write().expect("ledger write lock") = ledger;
}

/// A budget of zero everywhere buys nothing, even with a strong candidate
/// sitting in the buffer. The budget is the host's statement of how much
/// labelling effort it has, and guidance never recommends more than was asked
/// for — a host with no capacity this minute can call the query freely without
/// receiving work it cannot do.
///
/// ´claim:guidance:a-zero-budget-yields-no-candidates-however-strong-the-pending-ones-are´
/// ´test:crate:request-labels-empty-budget-returns-empty´
#[test]
fn request_labels_empty_budget_returns_empty() {
    let assayer = guidance_assayer();
    assayer.pending_buffer.insert(pending_assessment(1, 0.9, 0.8, None));

    let requests = assayer.request_labels(LabelBudget::default(), LabelGuidanceParams::default());

    assert!(requests.is_empty());
}

/// All three categories rank the live pending buffer end to end: the more
/// uncertain assessment leads the risk-informative list and the less uncertain
/// one follows, the same assessment leads on anchor weight, and the one routed
/// through a starved, divergent Ledger cell comes back under starvation relief
/// carrying the score that cell's own divergence and label deficit produce.
/// The categories are scored independently against real state, so guidance
/// reflects what the system currently holds rather than a precomputed list.
///
/// ´claim:guidance:all-three-categories-rank-the-live-pending-buffer-against-real-model-and-ledger-state´
/// ´test:crate:request-labels-ranks-seeded-pending-snapshot´
#[test]
fn request_labels_ranks_seeded_pending_snapshot() {
    let assayer = guidance_assayer();
    install_starved_ledger(&assayer);

    assayer
        .pending_buffer
        .insert(pending_assessment(1, 0.9, 0.8, Some((STARVED_SENTINEL, STARVED_COORDINATE))));
    assayer.pending_buffer.insert(pending_assessment(2, 0.4, 0.2, None));

    let requests = assayer.request_labels(
        LabelBudget {
            risk_informative: 2,
            investigation_candidates: 2,
            starvation_relief: 1,
        },
        LabelGuidanceParams {
            starvation_threshold: 0.01,
            ..LabelGuidanceParams::default()
        },
    );

    assert_eq!(requests.risk_informative[0].assessment_id, AssessmentId(1));
    assert_eq!(requests.risk_informative[1].assessment_id, AssessmentId(2));
    assert_eq!(requests.investigation_candidates[0].assessment_id, AssessmentId(1));
    assert_eq!(requests.starvation_relief[0].assessment_id, AssessmentId(1));
    assert!((requests.starvation_relief[0].score - 0.0985).abs() < 1e-4);
}

/// One assessment can win all three categories at once and is returned in all
/// three, each occurrence naming the other two it also belongs to. Each
/// category's budget is honoured on its own terms rather than being eroded by
/// what another category already took, and the cross-references are what tell
/// the host that a single label here would answer three separate needs.
///
/// (´claim:guidance:an-assessment-selected-by-several-categories-is-marked-in-each-with-the-others-but-not-itself´)
/// ´test:crate:request-labels-does-not-deduplicate-across-categories´
#[test]
fn request_labels_does_not_deduplicate_across_categories() {
    let assayer = guidance_assayer();
    install_starved_ledger(&assayer);
    assayer
        .pending_buffer
        .insert(pending_assessment(1, 0.9, 0.8, Some((STARVED_SENTINEL, STARVED_COORDINATE))));

    let requests = assayer.request_labels(
        LabelBudget {
            risk_informative: 1,
            investigation_candidates: 1,
            starvation_relief: 1,
        },
        LabelGuidanceParams::default(),
    );

    assert_eq!(requests.risk_informative.len(), 1);
    assert_eq!(requests.investigation_candidates.len(), 1);
    assert_eq!(requests.starvation_relief.len(), 1);
    assert_eq!(requests.risk_informative[0].assessment_id, AssessmentId(1));
    assert_eq!(requests.investigation_candidates[0].assessment_id, AssessmentId(1));
    assert_eq!(requests.starvation_relief[0].assessment_id, AssessmentId(1));
    assert_eq!(
        requests.risk_informative[0].also_in,
        vec![LabelCategory::InvestigationCandidate, LabelCategory::StarvationRelief]
    );
}

/// The scan limit bounds how much of the pending buffer guidance will look at,
/// independently of how generous the budget is: a limit of one leaves a single
/// candidate to rank however many are actually buffered. The buffer can hold
/// enormous numbers of entries and the query runs on the caller's thread, so
/// the host needs a knob that caps the work rather than only the output.
///
/// ´claim:guidance:the-scan-limit-caps-the-work-guidance-does-independently-of-the-budget-it-fills´
/// ´test:crate:request-labels-respects-scan-limit´
#[test]
fn request_labels_respects_scan_limit() {
    let assayer = guidance_assayer();
    assayer.pending_buffer.insert(pending_assessment(1, 0.9, 0.8, None));
    assayer.pending_buffer.insert(pending_assessment(2, 0.4, 0.2, None));

    let requests = assayer.request_labels(
        LabelBudget {
            risk_informative: 10,
            investigation_candidates: 0,
            starvation_relief: 0,
        },
        LabelGuidanceParams {
            scan_limit: 1,
            ..LabelGuidanceParams::default()
        },
    );

    assert_eq!(requests.risk_informative.len(), 1);
}
