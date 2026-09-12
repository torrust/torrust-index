// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`risk_scorer_uses_uncertainty`] | guidance | The risk-informative score is the assessment's own probability-space uncertainty, taken as it stands. A label buys the most when the model was least sure, so ranking by the doubt the model already recorded needs no separate notion of informativeness — the estimate carries it. |
//! | [`investigation_scorer_uses_anchor_weight`] | guidance | The investigation score is the anchor weight — how far the estimate leant on locally-learned evidence rather than the inherited model — and it is the same score whether or not the caller asks for proxy scoring. Labelling where the anchor dominates is what eventually lets the inherited model take over there; the proxy switch is accepted at the interface but, as things stand, selects nothing different. |
//! | [`starvation_scorer_requires_divergence_and_starvation`] | guidance | Starvation relief asks for both conditions at once, multiplied together: how far the cell's adverse rate exceeds the eligible base rate, times how far short of a full recent-label window the cell has fallen. Either factor at zero zeroes the score, so an unremarkable cell is not chased merely for being unlabelled, and a genuinely divergent cell that has already had its window's worth of labels stops asking for more. |
//! | [`starvation_scorer_reads_routed_ledger_entry`] | guidance | A request's coordinate is routed through the Ledger to the cell that actually covers it, and that cell's own history — its adverse rate and its recent label count — is what gets scored. The score therefore reflects the neighbourhood this entity belongs to rather than the tree as a whole, which is the only way starvation can be localised to the region that is short of labels. |
//! | [`select_candidates_sorts_descending_and_respects_budget`] | guidance | A budget buys the best candidates, not the first ones found: the selection is ordered by score from highest down and then cut at the budget, so a lower-scoring entry offered earlier does not displace a better one offered later. Labelling capacity is the scarce resource the whole guidance surface exists to allocate, and spending it in arrival order would waste it. |
//! | [`cross_membership_marks_other_selected_categories`] | guidance | An assessment selected by more than one category is told so in each place it appears, each entry naming the other categories and never itself. The three categories are scored independently and may well converge on the same request; a host spending its labelling effort needs to see that one label would satisfy several appetites at once rather than paying for the same assessment three times. |

//! Core label guidance query and scoring.
//!
//! The guidance surface ranks live pending assessments in three categories
//! ranked and truncated independently (´dec:surface:three-categories´):
//! risk-informative, investigation candidates, and starvation relief. It is a
//! read-only Core query initiated by the host; it does not send work to the
//! model-owner thread.
//!
//! # Cross-References
//!
//! - (´sec:guidance:core´) — the interface, its three criteria, and the policy
//!   keeping the lists independent
//! - (´dec:surface:read-only-guidance´) — guidance answers from published state
//!   and returns without an error channel

use std::collections::{HashMap, HashSet};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::Assayer;
use crate::assessment::AssessmentSharedState;
use crate::ledger::{SentinelLedger, read_route};
use crate::pending::{PendingAssessment, SentinelExtraction};
use crate::types::{AssessmentId, EntityKey, PersistentTimestamp, SentinelId};

/// Recent eligible label window used by starvation-relief scoring.
///
/// The window `N_ledger` ships at its tabulated default of two hundred
/// (´tab:config:ledger´), which is the count the starvation-relief score
/// divides by (´def:ledger:starvation-score´).
///
/// ´const:assayer:starvation-relief-window´ (´alg:const:scalar´)
/// ´const:assayer:starvation-relief-window-scalar-200p0´
const RECENT_ELIGIBLE_LABEL_WINDOW: f64 = 200.0;

/// Integer form of [`RECENT_ELIGIBLE_LABEL_WINDOW`] for lossless count capping.
///
/// It is the same tabulated window, read as a count (´tab:config:ledger´).
///
/// ´const:assayer:starvation-relief-window-integer´ (´alg:const:count´)
/// ´const:assayer:starvation-relief-window-integer-count-200´
const RECENT_ELIGIBLE_LABEL_WINDOW_COUNT: u64 = 200;

/// Maximum read-lock acquisitions per Sentinel in one guidance call.
///
/// Each acquisition reads at most one routed Ledger entry, keeping individual
/// lock holds short while bounding the total read-side work per call.
///
/// Unlike the collector's figures, this one bounds the work and not the piece:
/// the cells past it are skipped rather than read under a fresh acquisition
/// (´dec:concurrency:bounded-traversal´), which can leave a relief score
/// understated (´cav:concurrency:guidance-truncation´). It matches the
/// collector's scan figure in spelling only, and carries no host-facing dial.
///
/// ´const:assayer:guidance-read-lock-bound´ (´alg:const:count´)
/// ´const:assayer:guidance-read-lock-bound-count-512´
const MAX_LEDGER_READ_LOCKS_PER_SENTINEL: usize = 512;

/// Budget for each label-guidance category.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LabelBudget {
    /// Maximum risk-informative candidates to return.
    pub risk_informative: usize,
    /// Maximum investigation candidates to return.
    pub investigation_candidates: usize,
    /// Maximum starvation-relief candidates to return.
    pub starvation_relief: usize,
}

impl LabelBudget {
    /// Returns `true` when every category budget is zero.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.risk_informative == 0 && self.investigation_candidates == 0 && self.starvation_relief == 0
    }
}

/// Tunable parameters for [`Assayer::request_labels`](crate::Assayer::request_labels).
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LabelGuidanceParams {
    /// Maximum pending entries cloned from the pending buffer.
    pub scan_limit: usize,
    /// Use anchor-weight proxy scoring for investigation candidates.
    pub investigation_proxy: bool,
    /// Minimum positive starvation score needed for starvation-relief output.
    pub starvation_threshold: f64,
}

impl Default for LabelGuidanceParams {
    fn default() -> Self {
        Self {
            scan_limit: 10_000,
            investigation_proxy: true,
            starvation_threshold: 0.01,
        }
    }
}

impl LabelGuidanceParams {
    /// Returns a sanitised copy suitable for infallible guidance execution.
    #[must_use]
    const fn normalised(self) -> Self {
        Self {
            scan_limit: self.scan_limit,
            investigation_proxy: self.investigation_proxy,
            starvation_threshold: finite_nonnegative(self.starvation_threshold),
        }
    }
}

/// Result set returned by [`Assayer::request_labels`](crate::Assayer::request_labels).
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LabelRequests {
    /// Pending requests with the highest Core classification uncertainty.
    pub risk_informative: Vec<LabelCandidate>,
    /// Pending requests where investigation would most reduce anchor reliance.
    pub investigation_candidates: Vec<LabelCandidate>,
    /// Pending requests routed through starved, high-divergence Ledger cells.
    pub starvation_relief: Vec<LabelCandidate>,
}

impl LabelRequests {
    /// Returns `true` when all category lists are empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.risk_informative.is_empty() && self.investigation_candidates.is_empty() && self.starvation_relief.is_empty()
    }
}

/// A pending assessment recommended for labelling.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LabelCandidate {
    /// Assessment to label through [`Assayer::label`](crate::Assayer::label).
    pub assessment_id: AssessmentId,
    /// Entity associated with the pending assessment.
    pub entity: EntityKey,
    /// Wall-clock timestamp captured when the assessment was made.
    pub timestamp: PersistentTimestamp,
    /// Category-local score used to rank this candidate.
    pub score: f64,
    /// Other guidance categories that also selected this assessment.
    pub also_in: Vec<LabelCategory>,
}

/// Label-guidance category marker.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[non_exhaustive]
pub enum LabelCategory {
    /// Candidate selected by uncertainty scoring.
    RiskInformative,
    /// Candidate selected by investigation scoring.
    InvestigationCandidate,
    /// Candidate selected by Ledger starvation scoring.
    StarvationRelief,
}

/// Category-local score before stale-entry validation.
#[derive(Clone, Copy, Debug)]
struct ScoredCandidate {
    /// Pending assessment ID.
    assessment_id: AssessmentId,
    /// Category-local score.
    score: f64,
}

/// Read-only implementation of the public label-guidance query.
#[must_use]
pub fn request_labels(assayer: &Assayer, budget: LabelBudget, params: LabelGuidanceParams) -> LabelRequests {
    let params = params.normalised();
    if budget.is_empty() || params.scan_limit == 0 {
        return LabelRequests::default();
    }

    let pending_snapshot = assayer.pending_buffer.guidance_snapshot(params.scan_limit);
    if pending_snapshot.is_empty() {
        return LabelRequests::default();
    }

    let snapshot = assayer.shared.published.load();
    let p_positive_eligible = finite_rate(snapshot.p_positive_eligible);
    // Starvation decay is measured against the timestamps the pending
    // entries carry, which come from the engine's clock. Reading the wall
    // clock here would compare two time domains: under an injected clock the
    // gap is the distance between virtual time and wall time, which the
    // elapsed reader clamps at its ceiling, and every entry then scores as
    // maximally starved (´dec:clock:two-domains´).
    let now = assayer.now_persistent();

    let mut risk_scores = score_pending_entries(&pending_snapshot, score_risk_informative);
    let mut investigation_scores = score_pending_entries(&pending_snapshot, |entry| {
        score_investigation_candidate(entry, params.investigation_proxy)
    });
    let mut starvation_scores = score_starvation_entries(assayer, &pending_snapshot, p_positive_eligible, params, &now);

    let mut requests = LabelRequests {
        risk_informative: select_candidates(
            assayer,
            &mut risk_scores,
            budget.risk_informative,
            LabelCategory::RiskInformative,
        ),
        investigation_candidates: select_candidates(
            assayer,
            &mut investigation_scores,
            budget.investigation_candidates,
            LabelCategory::InvestigationCandidate,
        ),
        starvation_relief: select_candidates(
            assayer,
            &mut starvation_scores,
            budget.starvation_relief,
            LabelCategory::StarvationRelief,
        ),
    };

    annotate_cross_membership(&mut requests);
    requests
}

/// Scores entries with a category scorer.
fn score_pending_entries(
    pending_snapshot: &[PendingAssessment],
    scorer: impl Fn(&PendingAssessment) -> f64,
) -> Vec<ScoredCandidate> {
    pending_snapshot
        .iter()
        .map(|entry| ScoredCandidate {
            assessment_id: entry.id,
            score: finite_nonnegative(scorer(entry)),
        })
        .collect()
}

/// Scores the risk-informative category.
#[must_use]
const fn score_risk_informative(entry: &PendingAssessment) -> f64 {
    finite_nonnegative(entry.risk_basis.uncertainty)
}

/// Scores the investigation-candidate category.
#[must_use]
const fn score_investigation_candidate(entry: &PendingAssessment, _investigation_proxy: bool) -> f64 {
    finite_nonnegative(entry.risk_basis.anchor_weight)
}

/// Scores all starvation-relief candidates under the read-lock budget.
fn score_starvation_entries(
    assayer: &Assayer,
    pending_snapshot: &[PendingAssessment],
    p_positive_eligible: f64,
    params: LabelGuidanceParams,
    now: &PersistentTimestamp,
) -> Vec<ScoredCandidate> {
    let mut lock_counts: HashMap<SentinelId, usize> = HashMap::new();
    let mut scores = Vec::new();

    for entry in pending_snapshot {
        let score = score_starvation_candidate(assayer, entry, p_positive_eligible, now, &mut lock_counts);
        if score > 0.0 && score >= params.starvation_threshold {
            scores.push(ScoredCandidate {
                assessment_id: entry.id,
                score,
            });
        }
    }

    scores
}

/// Scores one pending entry for starvation relief.
fn score_starvation_candidate(
    assayer: &Assayer,
    entry: &PendingAssessment,
    p_positive_eligible: f64,
    now: &PersistentTimestamp,
    lock_counts: &mut HashMap<SentinelId, usize>,
) -> f64 {
    let mut best_score: f64 = 0.0;

    for (sentinel_id, extraction) in &entry.sentinel_extractions {
        if !is_guidance_routable(extraction) || !claim_ledger_read_lock(*sentinel_id, lock_counts) {
            continue;
        }

        let Some(ledger_lock) = assayer.outcome_ledger.get_arc(*sentinel_id) else {
            continue;
        };

        let ledger = ledger_lock.read().unwrap_or_else(std::sync::PoisonError::into_inner);
        best_score = best_score.max(score_starvation_from_ledger(
            &ledger,
            extraction.coordinate,
            p_positive_eligible,
            assayer.config.temporal.gamma_t_ledger,
            now,
        ));
    }

    best_score
}

/// Returns `true` when an extraction represents a real routed Sentinel cell.
#[must_use]
const fn is_guidance_routable(extraction: &SentinelExtraction) -> bool {
    extraction.occupancy && !extraction.features.is_empty()
}

/// Consumes one Sentinel read-lock budget slot when available.
fn claim_ledger_read_lock(sentinel_id: SentinelId, lock_counts: &mut HashMap<SentinelId, usize>) -> bool {
    let count = lock_counts.entry(sentinel_id).or_insert(0);
    if *count >= MAX_LEDGER_READ_LOCKS_PER_SENTINEL {
        return false;
    }
    *count += 1;
    true
}

/// Scores a routed Ledger cell for starvation relief.
#[must_use]
fn score_starvation_from_ledger(
    ledger: &SentinelLedger,
    coordinate: u128,
    p_positive_eligible: f64,
    gamma_t_ledger: f64,
    now: &PersistentTimestamp,
) -> f64 {
    if !ledger.has_root() {
        return 0.0;
    }

    let entry = read_route(ledger, coordinate);
    let view = entry.read_decayed(gamma_t_ledger, now);
    score_starvation_cell(view.bad_rate, entry.recent_eligible_count, p_positive_eligible)
}

/// Scores one Ledger cell using divergence multiplied by starvation.
#[must_use]
fn score_starvation_cell(bad_rate: f64, recent_eligible_count: u64, p_positive_eligible: f64) -> f64 {
    let divergence = (finite_rate(bad_rate) - finite_rate(p_positive_eligible)).max(0.0);
    let capped_count = recent_eligible_count.min(RECENT_ELIGIBLE_LABEL_WINDOW_COUNT);
    let capped_count = u32::try_from(capped_count).expect("guidance label window fits in u32");
    let starvation = (1.0 - (f64::from(capped_count) / RECENT_ELIGIBLE_LABEL_WINDOW)).max(0.0);
    finite_nonnegative(divergence * starvation)
}

/// Selects sorted, still-live candidates for one category.
fn select_candidates(
    assayer: &Assayer,
    scored: &mut [ScoredCandidate],
    limit: usize,
    category: LabelCategory,
) -> Vec<LabelCandidate> {
    if limit == 0 || scored.is_empty() {
        return Vec::new();
    }

    scored.sort_by(|left, right| {
        right
            .score
            .total_cmp(&left.score)
            .then_with(|| left.assessment_id.cmp(&right.assessment_id))
    });

    let mut selected = Vec::with_capacity(limit.min(scored.len()));
    let mut seen = HashSet::with_capacity(limit.min(scored.len()));

    for candidate in scored.iter() {
        if selected.len() >= limit {
            break;
        }
        if !seen.insert(candidate.assessment_id) {
            continue;
        }
        if let Some(entry) = assayer.pending_buffer.get_for_guidance(candidate.assessment_id) {
            selected.push(label_candidate_from_entry(&entry, candidate.score, category));
        }
    }

    selected
}

/// Builds a public candidate from a live pending entry.
fn label_candidate_from_entry(entry: &PendingAssessment, score: f64, _category: LabelCategory) -> LabelCandidate {
    LabelCandidate {
        assessment_id: entry.id,
        entity: entry.entity.clone(),
        timestamp: entry.persistent_timestamp,
        score,
        also_in: Vec::new(),
    }
}

/// Populates `also_in` using the final per-category selections.
fn annotate_cross_membership(requests: &mut LabelRequests) {
    let mut membership: HashMap<AssessmentId, HashSet<LabelCategory>> = HashMap::new();

    record_membership(&requests.risk_informative, LabelCategory::RiskInformative, &mut membership);
    record_membership(
        &requests.investigation_candidates,
        LabelCategory::InvestigationCandidate,
        &mut membership,
    );
    record_membership(&requests.starvation_relief, LabelCategory::StarvationRelief, &mut membership);

    apply_membership(&mut requests.risk_informative, LabelCategory::RiskInformative, &membership);
    apply_membership(
        &mut requests.investigation_candidates,
        LabelCategory::InvestigationCandidate,
        &membership,
    );
    apply_membership(&mut requests.starvation_relief, LabelCategory::StarvationRelief, &membership);
}

/// Records category membership for a list of candidates.
fn record_membership(
    candidates: &[LabelCandidate],
    category: LabelCategory,
    membership: &mut HashMap<AssessmentId, HashSet<LabelCategory>>,
) {
    for candidate in candidates {
        membership.entry(candidate.assessment_id).or_default().insert(category);
    }
}

/// Applies cross-category memberships to one candidate list.
fn apply_membership(
    candidates: &mut [LabelCandidate],
    own_category: LabelCategory,
    membership: &HashMap<AssessmentId, HashSet<LabelCategory>>,
) {
    for candidate in candidates {
        candidate.also_in = ordered_memberships(candidate.assessment_id, own_category, membership);
    }
}

/// Returns categories for `assessment_id`, excluding the current list.
fn ordered_memberships(
    assessment_id: AssessmentId,
    own_category: LabelCategory,
    membership: &HashMap<AssessmentId, HashSet<LabelCategory>>,
) -> Vec<LabelCategory> {
    let Some(categories) = membership.get(&assessment_id) else {
        return Vec::new();
    };

    [
        LabelCategory::RiskInformative,
        LabelCategory::InvestigationCandidate,
        LabelCategory::StarvationRelief,
    ]
    .into_iter()
    .filter(|category| *category != own_category && categories.contains(category))
    .collect()
}

/// Returns a finite non-negative score, mapping invalid values to zero.
#[must_use]
const fn finite_nonnegative(value: f64) -> f64 {
    if value.is_finite() { value.max(0.0) } else { 0.0 }
}

/// Returns a finite probability-like value in `[0, 1]`.
#[must_use]
const fn finite_rate(value: f64) -> f64 {
    if value.is_finite() { value.clamp(0.0, 1.0) } else { 0.0 }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::time::Instant;

    use super::*;
    use crate::health::DegradationContext;
    use crate::ledger::LedgerEntry;
    use crate::pending::PendingRiskBasis;

    fn pending(id: u64, uncertainty: f64, anchor_weight: f64) -> PendingAssessment {
        PendingAssessment {
            spatial_axis_ids: Vec::new(),
            id: AssessmentId(id),
            timestamp: Instant::now(),
            persistent_timestamp: PersistentTimestamp::now(),
            entity: EntityKey::new(id.to_le_bytes().to_vec()),
            sentinel_extractions: HashMap::new(),
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

    /// The risk-informative score is the assessment's own probability-space
    /// uncertainty, taken as it stands. A label buys the most when the model
    /// was least sure, so ranking by the doubt the model already recorded needs
    /// no separate notion of informativeness — the estimate carries it.
    ///
    /// ´claim:guidance:the-risk-informative-score-is-the-assessments-own-recorded-uncertainty´
    /// ´test:unit:risk-scorer-uses-uncertainty´
    #[test]
    fn risk_scorer_uses_uncertainty() {
        let entry = pending(1, 0.42, 0.1);
        assert!((score_risk_informative(&entry) - 0.42).abs() <= f64::EPSILON);
    }

    /// The investigation score is the anchor weight — how far the estimate
    /// leant on locally-learned evidence rather than the inherited model — and
    /// it is the same score whether or not the caller asks for proxy scoring.
    /// Labelling where the anchor dominates is what eventually lets the
    /// inherited model take over there; the proxy switch is accepted at the
    /// interface but, as things stand, selects nothing different.
    ///
    /// ´claim:guidance:the-investigation-score-is-the-anchor-weight-and-the-proxy-switch-selects-the-same-scoring´
    /// ´test:unit:investigation-scorer-uses-anchor-weight´
    #[test]
    fn investigation_scorer_uses_anchor_weight() {
        let entry = pending(1, 0.1, 0.73);
        assert!((score_investigation_candidate(&entry, true) - 0.73).abs() <= f64::EPSILON);
        assert!((score_investigation_candidate(&entry, false) - 0.73).abs() <= f64::EPSILON);
    }

    /// Starvation relief asks for both conditions at once, multiplied together:
    /// how far the cell's adverse rate exceeds the eligible base rate, times
    /// how far short of a full recent-label window the cell has fallen. Either
    /// factor at zero zeroes the score, so an unremarkable cell is not chased
    /// merely for being unlabelled, and a genuinely divergent cell that has
    /// already had its window's worth of labels stops asking for more.
    ///
    /// ´claim:guidance:starvation-relief-multiplies-divergence-by-starvation-so-either-alone-scores-nothing´
    /// ´test:unit:starvation-scorer-requires-divergence-and-starvation´
    #[test]
    fn starvation_scorer_requires_divergence_and_starvation() {
        let score = score_starvation_cell(0.15, 3, 0.05);
        assert!((score - 0.0985).abs() < 1e-12);

        assert!(score_starvation_cell(0.03, 3, 0.05).abs() <= f64::EPSILON);
        assert!(score_starvation_cell(0.15, 200, 0.05).abs() <= f64::EPSILON);
    }

    /// A request's coordinate is routed through the Ledger to the cell that
    /// actually covers it, and that cell's own history — its adverse rate and
    /// its recent label count — is what gets scored. The score therefore
    /// reflects the neighbourhood this entity belongs to rather than the tree
    /// as a whole, which is the only way starvation can be localised to the
    /// region that is short of labels.
    ///
    /// ´claim:guidance:a-coordinate-is-routed-to-the-ledger-cell-covering-it-and-scored-from-that-cells-own-history´
    /// ´test:unit:starvation-scorer-reads-routed-ledger-entry´
    #[test]
    fn starvation_scorer_reads_routed_ledger_entry() {
        let mut ledger = SentinelLedger::new();
        ledger.ensure_root();
        let mut entry = LedgerEntry::new_neutral();
        entry.ewma_bad_rate = 0.15;
        entry.recent_eligible_count = 3;
        ledger.insert(crate::types::LedgerKey::new(0, 8), entry);

        let score = score_starvation_from_ledger(&ledger, 1, 0.05, 1.0, &PersistentTimestamp::now());
        assert!((score - 0.0985).abs() < 1e-12);
    }

    /// A budget buys the best candidates, not the first ones found: the
    /// selection is ordered by score from highest down and then cut at the
    /// budget, so a lower-scoring entry offered earlier does not displace a
    /// better one offered later. Labelling capacity is the scarce resource the
    /// whole guidance surface exists to allocate, and spending it in arrival
    /// order would waste it.
    ///
    /// ´claim:guidance:a-budget-buys-the-highest-scoring-candidates-rather-than-the-first-ones-found´
    /// ´test:unit:select-candidates-sorts-descending-and-respects-budget´
    #[test]
    fn select_candidates_sorts_descending_and_respects_budget() {
        let mut scored = vec![
            ScoredCandidate {
                assessment_id: AssessmentId(3),
                score: 0.3,
            },
            ScoredCandidate {
                assessment_id: AssessmentId(1),
                score: 0.7,
            },
            ScoredCandidate {
                assessment_id: AssessmentId(2),
                score: 0.5,
            },
        ];

        let assayer = crate::Assayer::build(crate::AssayerConfig {
            // The capacity is host-set with no default; the fixture declares it.
            infrastructure: crate::testing::test_infrastructure(),
            ..Default::default()
        })
        .expect("no persistence configured");
        assayer.pending_buffer.insert(pending(1, 0.7, 0.0));
        assayer.pending_buffer.insert(pending(2, 0.5, 0.0));
        assayer.pending_buffer.insert(pending(3, 0.3, 0.0));

        let selected = select_candidates(&assayer, &mut scored, 2, LabelCategory::RiskInformative);
        assert_eq!(selected.len(), 2);
        assert_eq!(selected[0].assessment_id, AssessmentId(1));
        assert_eq!(selected[1].assessment_id, AssessmentId(2));
    }

    /// An assessment selected by more than one category is told so in each
    /// place it appears, each entry naming the other categories and never
    /// itself. The three categories are scored independently and may well
    /// converge on the same request; a host spending its labelling effort needs
    /// to see that one label would satisfy several appetites at once rather
    /// than paying for the same assessment three times.
    ///
    /// ´claim:guidance:an-assessment-selected-by-several-categories-is-marked-in-each-with-the-others-but-not-itself´
    /// ´test:unit:cross-membership-marks-other-selected-categories´
    #[test]
    fn cross_membership_marks_other_selected_categories() {
        let mut requests = LabelRequests {
            risk_informative: vec![LabelCandidate {
                assessment_id: AssessmentId(1),
                entity: EntityKey::new(vec![1]),
                timestamp: PersistentTimestamp::now(),
                score: 0.8,
                also_in: Vec::new(),
            }],
            investigation_candidates: vec![LabelCandidate {
                assessment_id: AssessmentId(1),
                entity: EntityKey::new(vec![1]),
                timestamp: PersistentTimestamp::now(),
                score: 0.4,
                also_in: Vec::new(),
            }],
            starvation_relief: Vec::new(),
        };

        annotate_cross_membership(&mut requests);

        assert_eq!(
            requests.risk_informative[0].also_in,
            vec![LabelCategory::InvestigationCandidate]
        );
        assert_eq!(
            requests.investigation_candidates[0].also_in,
            vec![LabelCategory::RiskInformative]
        );
    }
}
