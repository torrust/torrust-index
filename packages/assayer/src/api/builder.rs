// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! `AssayerBuilder` — validated construction of an `Assayer`.
//!
//! # 9-Phase Build Process
//!
//! 1. Validate signal schema, parameters, templates
//! 2. Persistence setup (create directories, test write access)
//! 3. Checkpoint attempt (load, version check, structural mismatch → cold start)
//! 4. Initialise state (`WorkingCopy::cold_start()` or restore + replay)
//! 5. Publish snapshots (`ArcSwap`)
//! 6. Create infrastructure (pending, signal cache, concordance, etc.)
//! 7. Create channels (label, command, health event)
//! 8. Spawn threads (model owner, identity maintenance, checkpoint scheduler)
//! 9. Assemble and return `Assayer`
//!
//! # Cross-References
//!
//! - The builder validates eagerly (´dec:construction:eager-validation´)

use std::sync::Arc;

use tracing::info;

use crate::Assayer;
use crate::config::types::AssayerConfig;
use crate::error::BuildError;
use crate::feature::interaction::{FeatureSelector, InteractionTemplate};
use crate::signal::{SignalDeclaration, SignalSchemaIndex};
use crate::testing::{Clock, SystemClock};

// ═══════════════════════════════════════════════════════════════════════════════
// AssayerBuilder
// ═══════════════════════════════════════════════════════════════════════════════

/// Builder for constructing an [`Assayer`] with validated configuration.
///
/// # Usage
///
/// ```ignore
/// let builder = Assayer::builder(config)
///     .signal_schema(&declarations);
/// let assayer = builder.build()?;
/// ```
pub struct AssayerBuilder {
    /// Configuration.
    config: AssayerConfig,
    /// Signal declarations (set via `signal_schema()`).
    signal_declarations: Option<Vec<SignalDeclaration>>,
    // TODO(2026-05-17) ´todo:code:make-this-option-vec-so-an´: Make this `Option<Vec<_>>` so an
    // omitted call applies canonical Type-4 templates while an explicit
    // empty vector opts out, which is a choice the eager builder validation
    // would then have to make (´dec:construction:eager-validation´).
    /// Interaction templates.
    interaction_templates: Vec<InteractionTemplate>,
    /// Source of time for the built engine.
    ///
    /// `None` selects the production default, [`SystemClock`].
    clock: Option<Arc<dyn Clock>>,
}

impl AssayerBuilder {
    /// Creates a new builder with the given configuration.
    pub(crate) const fn new(config: AssayerConfig) -> Self {
        Self {
            config,
            signal_declarations: None,
            interaction_templates: Vec::new(),
            clock: None,
        }
    }

    /// Overrides the engine's source of time.
    ///
    /// Production leaves this unset and gets `SystemClock`. Tests
    /// pass a `VirtualClock` so that
    /// report staleness and decay intervals become reproducible.
    ///
    /// The clock belongs to one engine instance; sharing one between
    /// engines is meaningful only when the test intends them to share
    /// a present.
    #[must_use]
    pub fn clock(mut self, clock: Arc<dyn Clock>) -> Self {
        self.clock = Some(clock);
        self
    }

    /// Sets the signal schema from declarations.
    ///
    /// Overwrites any previously set schema.
    #[must_use]
    pub fn signal_schema(mut self, declarations: &[SignalDeclaration]) -> Self {
        self.signal_declarations = Some(declarations.to_vec());
        self
    }

    /// Sets interaction templates.
    ///
    /// Overwrites any previously set templates.
    #[must_use]
    pub fn interaction_templates(mut self, templates: Vec<InteractionTemplate>) -> Self {
        self.interaction_templates = templates;
        self
    }

    /// Builds and starts the `Assayer`.
    ///
    /// Executes the 9-phase build process:
    ///
    /// 1. **Validate** — Schema, parameters, templates
    /// 2. **Persistence setup** — Create directories, test write access
    /// 3. **Checkpoint attempt** — Load, version check, mismatch → cold start
    /// 4. **Initialise state** — Cold start or restore + replay
    /// 5. **Publish snapshots** — `ArcSwap` initialisation
    /// 6. **Create infrastructure** — Pending buffer, signal cache, etc.
    /// 7. **Create channels** — Label, command, health event channels
    /// 8. **Spawn threads** — Model owner, identity maintenance, checkpoint
    /// 9. **Assemble** — Return `Assayer`
    ///
    /// # Errors
    ///
    /// Returns `BuildError` on validation failure or persistence setup
    /// failure, and on a restore that was read and then refused: a checkpoint
    /// whose stored precision matrix the model may not hold
    /// ([`BuildError::CheckpointNotPositiveDefinite`]), one whose stored state
    /// disagrees with its own declared width
    /// ([`BuildError::CheckpointDimensionMismatch`]), or a journal beside it
    /// that cannot be read for a reason the durability protocol does not
    /// absorb ([`BuildError::JournalUnreadable`]). Those three are refusals
    /// rather than cold starts because the host still has an action to take,
    /// and can only take it if construction says so.
    pub fn build(self) -> Result<Assayer, BuildError> {
        // ─── Phase 1: Validate ───────────────────────────────────────────
        let signal_declarations = self.signal_declarations.ok_or(BuildError::NoSignalSchema)?;

        // Validate the signal schema: duplicate names and shape parameters
        // the encoding cannot honour are refused here.
        let schema = SignalSchemaIndex::from_declarations(&signal_declarations)?;

        // Validate interaction templates against the construction-time
        // layout the schema fixes (´dec:construction:eager-validation´).
        for (idx, template) in self.interaction_templates.iter().enumerate() {
            if let Err(reason) = validate_interaction_template(template, schema.total_width()) {
                return Err(BuildError::InvalidInteractionTemplate {
                    template_index: idx,
                    reason,
                });
            }
        }

        // Validate all numerical parameters (´dec:construction:eager-validation´).
        validate_numerical_parameters(&self.config)?;

        // ─── Phase 2: Persistence setup ──────────────────────────────────
        if let Some(ref persistence) = self.config.persistence {
            setup_persistence_dirs(persistence)?;
        }

        // ─── Phases 3–9: Delegate to existing build() ────────────────────
        info!(
            instance_id = %self.config.instance_id,
            signals = signal_declarations.len(),
            interaction_templates = self.interaction_templates.len(),
            "AssayerBuilder: validation complete, building Assayer"
        );

        let clock: Arc<dyn Clock> = self.clock.unwrap_or_else(|| Arc::new(SystemClock));
        Assayer::build_from_builder(self.config, &signal_declarations, &self.interaction_templates, clock)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Interaction Template Validation
// ═══════════════════════════════════════════════════════════════════════════════

/// Validates a single interaction template against the construction-time
/// layout (´dec:construction:eager-validation´).
///
/// Every offset a template carries is checked against the block that
/// resolves it: a Sentinel-slot offset against the base extraction width
/// — the width every slot has before any spatial axis extends it — an
/// aggregate offset against the aggregate block, and a context offset
/// against the fixed context region a cold start lays out (bias,
/// aggregates, signals). A template naming a position outside those
/// bounds would reach the assessment path, which has already promised it
/// cannot fail (´dec:degradation:infallible-core´).
///
/// TODO(2026-05-22) ´todo:code:validate-the-canonical´: validate the canonical
/// `FeatureSelector`-based template contract's remaining checks — signal
/// names against the declared schema and `CompetitiveIndicator`
/// placement, which the legacy offset-based enum cannot express — once
/// the operand migration lands. Templates name their operands
/// semantically (´dec:vector:semantic-templates´), and this is the pass
/// that validates them eagerly (´dec:construction:eager-validation´).
fn validate_interaction_template(template: &InteractionTemplate, p_sig: usize) -> Result<(), String> {
    use crate::feature::dimension_map::{AGGREGATE_FEATURE_COUNT, ID_CROSS_DIM_FEATURE_COUNT, SENTINEL_Q_BASE};

    // The context region that exists at construction: bias, the aggregate
    // block, and the declared signal block.
    let context_width = 1 + AGGREGATE_FEATURE_COUNT + p_sig;

    let check_slot = |offset: usize| {
        if offset < SENTINEL_Q_BASE {
            Ok(())
        } else {
            Err(format!(
                "sentinel feature offset {offset} exceeds the base extraction width {SENTINEL_Q_BASE}"
            ))
        }
    };
    let check_agg = |offset: usize| {
        if offset < AGGREGATE_FEATURE_COUNT {
            Ok(())
        } else {
            Err(format!(
                "aggregate offset {offset} exceeds the aggregate block width {AGGREGATE_FEATURE_COUNT}"
            ))
        }
    };
    // A context operand names a block and an offset within it, so it is
    // checked against that block rather than against a flat width
    // (´alg:dimension:compilation-pipeline´). The identity block is absent at
    // construction and its offsets are checked against the block's own fixed
    // width; the entity-relative selectors name no context and are refused.
    let check_context = |selector: FeatureSelector| match selector {
        FeatureSelector::AggregateFeature(offset) => {
            if offset < AGGREGATE_FEATURE_COUNT {
                Ok(())
            } else {
                Err(format!(
                    "context aggregate offset {offset} exceeds the aggregate block width {AGGREGATE_FEATURE_COUNT}"
                ))
            }
        }
        FeatureSelector::IdentityFeature(offset) => {
            if offset < ID_CROSS_DIM_FEATURE_COUNT {
                Ok(())
            } else {
                Err(format!(
                    "context identity offset {offset} exceeds the cross-dimension block width {ID_CROSS_DIM_FEATURE_COUNT}"
                ))
            }
        }
        FeatureSelector::DeclaredSignal(offset) => {
            if offset < p_sig {
                Ok(())
            } else {
                Err(format!(
                    "context signal offset {offset} exceeds the declared signal width {p_sig}"
                ))
            }
        }
        FeatureSelector::SlotFeature(_) | FeatureSelector::OwningCompetitiveIndicator => {
            Err("a context operand must name a block, not the entity the template expands over".to_string())
        }
    };
    let _ = context_width;

    match template {
        InteractionTemplate::Type1 {
            sentinel_feature_offset,
            context,
        } => {
            check_slot(*sentinel_feature_offset)?;
            check_context(*context)
        }
        InteractionTemplate::Type2 {
            sentinel_a,
            sentinel_b,
            feature_offset,
        } => {
            if sentinel_a == sentinel_b {
                return Err(format!(
                    "named pair template names the same Sentinel twice ({sentinel_a:?}); a named pair is two Sentinels"
                ));
            }
            check_slot(*feature_offset)
        }
        InteractionTemplate::Type3 { feature_offset } => check_slot(*feature_offset),
        InteractionTemplate::Type4 { agg_offset, context } => {
            check_agg(*agg_offset)?;
            check_context(*context)
        }
        InteractionTemplate::Type5 { context } => check_context(*context),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Numerical Parameter Validation (´dec:construction:eager-validation´)
// ═══════════════════════════════════════════════════════════════════════════════

/// Validates every scalar parameter the configuration module owns against
/// the constraint column of the configuration chapter's tables
/// (´chap:spec:configuration´) — the column is part of the specification,
/// so a value outside it is a configuration error, not a tuning choice.
/// The re-exported external configurations are validated here too: every
/// floating-point field reachable from `AssayerConfig` passes through this
/// function, including the ones the tabulated columns describe as internal
/// to an algorithm.
///
/// Every one of those fields is tested for finiteness before any bound is
/// applied to it. A bound alone does not reject a value that is not a number:
/// every ordered comparison with NaN is false, so a NaN satisfies both ends
/// of an interval at once and then propagates through the model arithmetic
/// construction claimed to have protected. Finiteness is therefore part of
/// each field's constraint rather than a pass of its own, carried either by
/// the shared helpers below or by an explicit test ahead of a comparison the
/// helpers do not express.
///
/// Returns `BuildError::InvalidParameter` on the first violation.
#[allow(clippy::too_many_lines)] // Justified: single flat validation covering all config parameters
#[allow(clippy::cast_precision_loss)] // Justified: reported values are diagnostic only
fn validate_numerical_parameters(config: &AssayerConfig) -> Result<(), BuildError> {
    /// Checks that a decay rate is a finite value in the open interval (0, 1).
    ///
    /// The finiteness test is not redundant beside the two bounds: every
    /// ordered comparison with a value that is not a number is false, so a
    /// NaN rate satisfies `val > 0.0` and `val < 1.0` at once and would pass
    /// a pair of bounds alone.
    fn check_rate(val: f64, path: &'static str) -> Result<(), BuildError> {
        if !val.is_finite() || val <= 0.0 || val >= 1.0 {
            return Err(BuildError::InvalidParameter {
                path,
                value: val,
                reason: "must be a finite value in (0, 1)",
            });
        }
        Ok(())
    }

    /// Checks that a count is at least one.
    const fn check_at_least_one(val: usize, path: &'static str) -> Result<(), BuildError> {
        if val == 0 {
            return Err(BuildError::InvalidParameter {
                path,
                value: 0.0,
                reason: "must be >= 1",
            });
        }
        Ok(())
    }

    /// Checks that a value is strictly positive and finite.
    fn check_positive(val: f64, path: &'static str) -> Result<(), BuildError> {
        if !val.is_finite() || val <= 0.0 {
            return Err(BuildError::InvalidParameter {
                path,
                value: val,
                reason: "must be > 0",
            });
        }
        Ok(())
    }

    /// Checks that a value is non-negative and finite.
    fn check_non_negative(val: f64, path: &'static str) -> Result<(), BuildError> {
        if !val.is_finite() || val < 0.0 {
            return Err(BuildError::InvalidParameter {
                path,
                value: val,
                reason: "must be >= 0",
            });
        }
        Ok(())
    }

    // ── Model decay rates (´tab:config:risk-model´) ──
    check_rate(config.model.gamma_opr, "model.gamma_opr")?;
    check_rate(config.model.gamma_inh, "model.gamma_inh")?;
    // The sister and anchor rate's tabulated constraint is the open
    // interval above the operational rate, not merely (0, 1): a sister
    // forgetting faster than the operational model inverts the two
    // regimes' timescales.
    if config.model.gamma_inh <= config.model.gamma_opr {
        return Err(BuildError::InvalidParameter {
            path: "model.gamma_inh",
            value: config.model.gamma_inh,
            reason: "must be > gamma_opr",
        });
    }
    check_rate(config.model.gamma_kappa, "model.gamma_kappa")?;

    // ── Temporal decay rates (´tab:config:temporal´) ──
    check_rate(config.temporal.gamma_t_core, "temporal.gamma_t_core")?;
    check_rate(config.temporal.gamma_t_ledger, "temporal.gamma_t_ledger")?;
    check_rate(config.temporal.gamma_t_identity, "temporal.gamma_t_identity")?;

    // ── Health monitoring (´tab:config:monitoring´) ──
    check_non_negative(config.monitoring.kappa_drift, "monitoring.kappa_drift")?;
    check_positive(config.monitoring.h_threshold, "monitoring.h_threshold")?;
    if config.monitoring.min_auc_class_count < 5 {
        return Err(BuildError::InvalidParameter {
            path: "monitoring.min_auc_class_count",
            value: config.monitoring.min_auc_class_count as f64,
            reason: "must be >= 5",
        });
    }
    if config.monitoring.recent_discrimination_window < 50 {
        return Err(BuildError::InvalidParameter {
            path: "monitoring.recent_discrimination_window",
            value: config.monitoring.recent_discrimination_window as f64,
            reason: "must be >= 50",
        });
    }
    check_positive(
        config.monitoring.sync_error_threshold_per_dimension,
        "monitoring.sync_error_threshold_per_dimension",
    )?;
    if config.monitoring.min_concordance_labels < 3 {
        return Err(BuildError::InvalidParameter {
            path: "monitoring.min_concordance_labels",
            value: config.monitoring.min_concordance_labels as f64,
            reason: "must be >= 3",
        });
    }
    if !config.monitoring.concordance_deficit_threshold.is_finite()
        || config.monitoring.concordance_deficit_threshold <= 0.0
        || config.monitoring.concordance_deficit_threshold >= 0.5
    {
        return Err(BuildError::InvalidParameter {
            path: "monitoring.concordance_deficit_threshold",
            value: config.monitoring.concordance_deficit_threshold,
            reason: "must be in (0, 0.5)",
        });
    }
    check_positive(config.monitoring.strong_alarm_threshold, "monitoring.strong_alarm_threshold")?;
    check_non_negative(config.monitoring.quiet_alarm_threshold, "monitoring.quiet_alarm_threshold")?;
    check_non_negative(
        config.monitoring.alarm_outcome_noise_allowance,
        "monitoring.alarm_outcome_noise_allowance",
    )?;
    check_positive(
        config.monitoring.feature_stability_threshold,
        "monitoring.feature_stability_threshold",
    )?;
    check_rate(config.monitoring.maturity_threshold, "monitoring.maturity_threshold")?;
    if config.monitoring.resolution_adequacy_threshold == 0 {
        return Err(BuildError::InvalidParameter {
            path: "monitoring.resolution_adequacy_threshold",
            value: 0.0,
            reason: "must be >= 1",
        });
    }
    check_rate(
        config.monitoring.feedback_latency_ewma_rate,
        "monitoring.feedback_latency_ewma_rate",
    )?;
    if config.monitoring.ledger_materiality_threshold == 0 {
        return Err(BuildError::InvalidParameter {
            path: "monitoring.ledger_materiality_threshold",
            value: 0.0,
            reason: "must be >= 1",
        });
    }
    check_rate(
        config.monitoring.attenuation_materiality_floor,
        "monitoring.attenuation_materiality_floor",
    )?;

    // ── Prior precision (´tab:config:risk-model´) ──
    if !config.model.lambda_prior.is_finite() || config.model.lambda_prior <= 0.0 {
        return Err(BuildError::InvalidParameter {
            path: "model.lambda_prior",
            value: config.model.lambda_prior,
            reason: "must be a finite value > 0",
        });
    }
    if !config.model.lambda_floor.is_finite()
        || config.model.lambda_floor <= 0.0
        || config.model.lambda_floor > config.model.lambda_prior
    {
        return Err(BuildError::InvalidParameter {
            path: "model.lambda_floor",
            value: config.model.lambda_floor,
            reason: "must be a finite value > 0 and <= lambda_prior",
        });
    }

    // ── Leverage and weight ceilings (´tab:config:risk-model´) ──
    if !config.model.c_leverage.is_finite() || config.model.c_leverage < 1.0 {
        return Err(BuildError::InvalidParameter {
            path: "model.c_leverage",
            value: config.model.c_leverage,
            reason: "must be a finite value >= 1",
        });
    }
    if !config.model.w_ceiling.is_finite() || config.model.w_ceiling < 1.0 {
        return Err(BuildError::InvalidParameter {
            path: "model.w_ceiling",
            value: config.model.w_ceiling,
            reason: "must be a finite value >= 1",
        });
    }

    // ── P+ initial (´def:weighting:initial-value´) ──
    check_rate(config.model.p_plus_init, "model.p_plus_init")?;

    // ── Cholesky (´tab:config:risk-model´) ──
    if config.cholesky.n_recompute < 100 {
        return Err(BuildError::InvalidParameter {
            path: "cholesky.n_recompute",
            value: f64::from(config.cholesky.n_recompute),
            reason: "must be >= 100",
        });
    }
    if !config.cholesky.kappa_growth_factor.is_finite() || config.cholesky.kappa_growth_factor <= 1.0 {
        return Err(BuildError::InvalidParameter {
            path: "cholesky.kappa_growth_factor",
            value: config.cholesky.kappa_growth_factor,
            reason: "must be a finite value > 1",
        });
    }
    // TODO(2026-05-23) ´todo:code:validate´: Validate
    // `cholesky.sync_error_threshold > 0` once the field is added to
    // `CholeskyConfig` (´def:config:synchronisation-threshold´).

    // ── Standardisation (´tab:config:standardisation´) ──
    check_rate(config.standardisation.gamma_std, "standardisation.gamma_std")?;
    // The standardising divisor is `sqrt(v) + epsilon` against a variance the
    // floor already holds above zero, so a zero guard is admissible and a
    // negative one is not: it can cancel the floor and leave a divisor at or
    // below zero, which inverts the sign of every standardised feature it
    // touches.
    check_non_negative(config.standardisation.epsilon, "standardisation.epsilon")?;
    // The clip is stated in standard deviations. At zero every feature
    // collapses to its mean and the model sees a constant; below zero the
    // clip bounds invert.
    check_positive(config.standardisation.n_clip, "standardisation.n_clip")?;
    // The variance floor is what keeps that divisor away from zero, so a
    // floor of zero or less is a floor that does not hold.
    check_positive(config.standardisation.v_floor, "standardisation.v_floor")?;

    // ── Concordance window sizes and thresholds (´tab:construction:parameters´) ──
    check_at_least_one(config.concordance.window_capacity, "concordance.window_capacity")?;
    check_at_least_one(
        config.concordance.recalibration_interval,
        "concordance.recalibration_interval",
    )?;
    // A target percentile is a value confined to the open unit interval,
    // like the eight rates this helper already covers.
    check_rate(config.concordance.percentile, "concordance.percentile")?;

    // ── Ledger (´tab:config:ledger´) ──
    check_rate(config.ledger.lambda_l, "ledger.lambda_l")?;
    // The counter the deletion predicate reads is eight bits wide
    // (´alg:ledger:entry-deletion´), so a threshold above 255 is one the count
    // can never reach: the range ends where the counter does rather than
    // admitting a value that would silently narrow.
    if !(1..=u32::from(u8::MAX)).contains(&config.ledger.n_absent) {
        return Err(BuildError::InvalidParameter {
            path: "ledger.n_absent",
            value: f64::from(config.ledger.n_absent),
            reason: crate::config::types::N_ABSENT_RANGE_REASON,
        });
    }
    if config.ledger.gc_horizon_days == 0 {
        return Err(BuildError::InvalidParameter {
            path: "ledger.gc_horizon_days",
            value: 0.0,
            reason: "must be >= 1",
        });
    }
    check_positive(config.ledger.gc_floor, "ledger.gc_floor")?;

    // ── Hibernation archive lifetime (´tab:construction:parameters´) ──
    // Both ends are refused for the same reason and neither is a preference.
    // A lifetime of zero days is a store nothing can ever be restored from,
    // which is the destructive path wearing the hibernating path's name
    // (´dec:construction:destructive-deregistration´); a host that wants that
    // has it already by not hibernating. A lifetime past the elapsed-time
    // clamp is a store that never expires, because every elapsed reading is
    // clamped at a year before the comparison sees it
    // (´dec:clock:embedded-clamp´) — the figure would be accepted, recorded,
    // and then mean something other than what it says.
    if config.hibernation.expiry_days == 0 {
        return Err(BuildError::InvalidParameter {
            path: "hibernation.expiry_days",
            value: 0.0,
            reason: "must be >= 1",
        });
    }
    if f64::from(config.hibernation.expiry_days) * 24.0 > crate::types::MAX_DECAY_HOURS {
        return Err(BuildError::InvalidParameter {
            path: "hibernation.expiry_days",
            value: f64::from(config.hibernation.expiry_days),
            reason: "must be <= 365; a longer lifetime than the elapsed-time clamp can never be reached",
        });
    }

    // ── Schur marginalisation (´tab:config:risk-model´) ──
    // The table gives the factor's domain as strictly positive, and the audit
    // said why the boundary matters at both ends: zero removes the lift
    // entirely, a negative offset can cross an eigenvalue of the block being
    // regularised and destroy its factorability, and a value that is not a
    // number poisons the matrix it is added to. These are the first two Schur
    // values a deployment can actually set, so this is the first pass in which
    // refusing them means anything.
    check_positive(config.schur.epsilon_schur, "schur.epsilon_schur")?;
    if !config.schur.posture_condition_ceiling.is_finite() || config.schur.posture_condition_ceiling <= 1.0 {
        return Err(BuildError::InvalidParameter {
            path: "schur.posture_condition_ceiling",
            value: config.schur.posture_condition_ceiling,
            reason: "must be > 1",
        });
    }

    // ── Convergence thresholds (´tab:config:monitoring´) ──
    // Diagnostic-only, and validated regardless: a wrong value here
    // misleads a reader by design. The cross-stage relation this pass
    // used to check — that the warming, converging and converged
    // triples increase — went with the eight stage figures it compared,
    // deleted because no path read them and the specification names no
    // stage they belong to, and the two Companion figures went with
    // them: the sufficiency floor is the host's and is no longer built
    // here (´dec:challenge:sufficiency-floor-owned-here´).
    check_positive(
        config.convergence.platt_converged_delta_cal,
        "convergence.platt_converged_delta_cal",
    )?;
    if config.convergence.platt_converged_refit_count == 0 {
        return Err(BuildError::InvalidParameter {
            path: "convergence.platt_converged_refit_count",
            value: 0.0,
            reason: "must be >= 1",
        });
    }

    // ── Blend statistics window ──
    check_at_least_one(config.blend_statistics.window_capacity, "blend_statistics.window_capacity")?;
    check_at_least_one(config.blend_statistics.publish_interval, "blend_statistics.publish_interval")?;

    // ── Persistence (´tab:construction:parameters´) ──
    if let Some(ref persistence) = config.persistence
        && persistence.checkpoint_interval.is_zero()
    {
        return Err(BuildError::InvalidParameter {
            path: "persistence.checkpoint_interval",
            value: 0.0,
            reason: "must be > 0",
        });
    }

    // ── Platt κ bounds (´tab:config:calibration´) ──
    if !config.platt.kappa_min.is_finite() || config.platt.kappa_min <= 0.0 {
        return Err(BuildError::InvalidParameter {
            path: "platt.kappa_min",
            value: config.platt.kappa_min,
            reason: "must be a finite value > 0",
        });
    }
    if !config.platt.kappa_max.is_finite() || config.platt.kappa_max <= config.platt.kappa_min {
        return Err(BuildError::InvalidParameter {
            path: "platt.kappa_max",
            value: config.platt.kappa_max,
            reason: "must be a finite value > kappa_min",
        });
    }
    // The search runs between the two bounds, so a starting slope outside
    // them is a slope the fit can never return to: the first refit would
    // compare against a value the configuration has already excluded.
    if !config.platt.kappa_initial.is_finite()
        || config.platt.kappa_initial < config.platt.kappa_min
        || config.platt.kappa_initial > config.platt.kappa_max
    {
        return Err(BuildError::InvalidParameter {
            path: "platt.kappa_initial",
            value: config.platt.kappa_initial,
            reason: "must be a finite value in [kappa_min, kappa_max]",
        });
    }
    check_rate(config.platt.gamma_cal, "platt.gamma_cal")?;
    // The golden-section search stops when the bracket narrows past this
    // tolerance. A tolerance of zero or below is a stopping test that never
    // fires, leaving the iteration cap as the only exit and spending every
    // iteration of it on every refit.
    check_positive(config.platt.golden_tolerance, "platt.golden_tolerance")?;
    // The drift accumulator resets when the fitted slope has moved by more
    // than this much. At zero every refit resets it, which is the same as
    // having no accumulator at all.
    check_positive(config.platt.delta_cal_threshold, "platt.delta_cal_threshold")?;

    // ── Standardisation bootstrap (´alg:standardisation:sentinel-bootstrap´) ──
    if config.standardisation.n_boot < 10 {
        return Err(BuildError::InvalidParameter {
            path: "standardisation.n_boot",
            value: f64::from(config.standardisation.n_boot),
            reason: "must be >= 10",
        });
    }
    if !config.standardisation.alpha_boot.is_finite()
        || config.standardisation.alpha_boot <= 0.0
        || config.standardisation.alpha_boot > 1.0
    {
        return Err(BuildError::InvalidParameter {
            path: "standardisation.alpha_boot",
            value: config.standardisation.alpha_boot,
            reason: "must be a finite value in (0, 1]",
        });
    }

    // ── Infrastructure capacities ──
    // The pending-buffer capacity is computed from these two
    // (´req:runtime:buffer-capacity´), so they carry its ≥ 1 constraint.
    if !config.infrastructure.expected_peak_request_rate.is_finite() || config.infrastructure.expected_peak_request_rate <= 0.0 {
        return Err(BuildError::InvalidParameter {
            path: "infrastructure.expected_peak_request_rate",
            value: config.infrastructure.expected_peak_request_rate,
            reason: "must be a positive finite rate",
        });
    }
    if config.infrastructure.expected_label_latency_secs == 0 {
        return Err(BuildError::InvalidParameter {
            path: "infrastructure.expected_label_latency_secs",
            value: 0.0,
            reason: "must be >= 1",
        });
    }
    if config.infrastructure.expiry_horizon_secs == 0 {
        return Err(BuildError::InvalidParameter {
            path: "infrastructure.expiry_horizon_secs",
            value: 0.0,
            reason: "must be >= 1",
        });
    }
    check_at_least_one(
        config.infrastructure.signal_cache_capacity,
        "infrastructure.signal_cache_capacity",
    )?;
    // The label queue's tabulated constraint is a floor of one hundred
    // (´tab:config:concurrency´), not merely non-zero: the queue absorbs
    // label bursts under the drop-oldest overflow policy, and a
    // few-slot queue drops evidence at ordinary submission rates.
    if config.infrastructure.label_channel_capacity < 100 {
        return Err(BuildError::InvalidParameter {
            path: "infrastructure.label_channel_capacity",
            value: config.infrastructure.label_channel_capacity as f64,
            reason: "must be >= 100",
        });
    }
    check_at_least_one(
        config.infrastructure.health_event_capacity,
        "infrastructure.health_event_capacity",
    )?;
    check_at_least_one(
        config.infrastructure.identity_observation_capacity,
        "infrastructure.identity_observation_capacity",
    )?;
    if config.infrastructure.identity_registration_timeout_secs == 0 {
        return Err(BuildError::InvalidParameter {
            path: "infrastructure.identity_registration_timeout_secs",
            value: 0.0,
            reason: "must be >= 1",
        });
    }
    // The command channel's capacity is host-set with no default: an
    // undeclared capacity is refused here, on the fallible surface, so
    // the value is a decision made at every deployment
    // (´tab:construction:parameters´).
    match config.infrastructure.command_channel_capacity {
        None => {
            return Err(BuildError::InvalidParameter {
                path: "infrastructure.command_channel_capacity",
                value: f64::NAN,
                reason: "must be set by the host; the command channel's capacity has no default",
            });
        }
        Some(0) => {
            return Err(BuildError::InvalidParameter {
                path: "infrastructure.command_channel_capacity",
                value: 0.0,
                reason: "must be >= 1",
            });
        }
        Some(_) => {}
    }

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════════
// Persistence Setup
// ═══════════════════════════════════════════════════════════════════════════════

/// Creates persistence directories and tests write access.
#[cfg(feature = "serde")]
fn setup_persistence_dirs(persistence: &crate::config::types::PersistenceConfig) -> Result<(), BuildError> {
    use std::fs;

    // Create checkpoint directory.
    if let Some(parent) = persistence.checkpoint_path().parent() {
        fs::create_dir_all(parent).map_err(BuildError::PersistenceSetupFailed)?;
    }

    // Create journal directory.
    if let Some(parent) = persistence.journal_path().parent() {
        fs::create_dir_all(parent).map_err(BuildError::PersistenceSetupFailed)?;
    }

    Ok(())
}

/// Refuses a persistence configuration when the serde feature is not enabled.
///
/// There is nothing to set up: the checkpoint and journal codecs are compiled
/// behind the feature, so this build has no writer to point at the directories.
/// The configuration cannot be honoured and is refused where the host can still
/// act on it (´dec:construction:eager-validation´).
#[cfg(not(feature = "serde"))]
const fn setup_persistence_dirs(_persistence: &crate::config::types::PersistenceConfig) -> Result<(), BuildError> {
    Err(BuildError::PersistenceUnsupported)
}
