// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Named numerical tolerances used by the assertion helpers.
//!
//! Centralising the epsilons here means a scenario test never spells
//! out `1e-6` in its body — it says "near" and the harness chooses
//! the right bound. Scenarios that are specifically *about* a tolerance
//! — matching the crossover the landscape places (´eq:landscape:crossover´),
//! for instance — import the relevant field by name.

/// Collection of per-domain tolerances.
///
/// Fields are grouped by the invariant they bound. Update in concert
/// with the spec — the numeric values here are the single source of
/// truth the harness uses.
#[derive(Debug, Clone, Copy)]
pub struct Tolerances {
    /// Generic floating-point comparison, one-shot quantities.
    pub default: f64,

    /// EWMA values (bad rate, compressed outcome, …).
    pub ewma: f64,

    /// AUC / ROC statistics.
    pub auc: f64,

    /// Tolerance on matching the reported crossover to the logit the
    /// reward arithmetic places it at (´eq:landscape:crossover´).
    pub crossover: f64,

    /// Precision-matrix synchronisation bound — `‖BΣ − I‖_F` — **per unit of
    /// model width**. The ceiling itself is this times the dimension; take it
    /// from [`Tolerances::precision_sync_ceiling`] rather than reading the
    /// coefficient as a bound.
    pub precision_sync_per_dimension: f64,

    /// Standardisation mismatch between store-time and label-time.
    pub standardisation: f64,

    /// Platt sharpness drift between successive refits, in **relative
    /// log-sharpness**: the bound is on `|ln κ₂ − ln κ₁|`, not on `|κ₂ − κ₁|`.
    ///
    /// Sharpness is a scale parameter, so the question a drift bound answers
    /// is what proportion the sharpness moved by, and an absolute bound
    /// answers it differently at every sharpness. The figure below admits a
    /// four-percent move and rejects a ten-percent one at any κ; the absolute
    /// `0.1` it replaced did so only near this fixture's own optimum of about
    /// two, and would have admitted a ten-percent move at a sharpness of one
    /// while rejecting a one-percent move at a sharpness of twenty.
    pub platt_log_sharpness: f64,

    /// Exact-equality requirement (e.g. bit-identical snapshots).
    ///
    /// Exposed as a field rather than hard-coded `0.0` so that tests
    /// read uniformly: `tol.bit_identical` instead of a bare literal.
    pub bit_identical: f64,
}

impl Tolerances {
    /// The synchronisation-error ceiling at a given model width.
    ///
    /// The specification states the threshold as a coefficient times the
    /// dimension rather than as a flat figure
    /// (´def:monitoring:synchronisation-error´), so a narrow model is held to a
    /// proportionately tighter bound than a wide one and no width is admitted
    /// more departure than the specification would admit it.
    #[must_use]
    pub fn precision_sync_ceiling(&self, dimension: usize) -> f64 {
        #[allow(clippy::cast_precision_loss)]
        let width = dimension as f64;
        self.precision_sync_per_dimension * width
    }
}

/// Default tolerances used by scenarios that don't override them.
///
/// The fields do not all have the same standing, and the table that declares
/// them says which each has (´tab:assayer:harness-scenario-tolerances´): two are
/// figures the specification states, one is the harness's reading of a
/// specification that gives a condition rather than a ceiling, four are the
/// harness's own, and one is exact equality rather than a tolerance. Every field
/// is read by an assertion (´cav:assayer:harness-unconsumed-tolerances´). The
/// harness declares the ones that are its own because nothing outside it does
/// (´dec:assayer:harness-declares´).
///
/// ´const:assayer:scenario-tolerance-defaults´ (´alg:const:form´)
/// ´const:assayer:scenario-tolerance-defaults-form-x57224f5b´
pub const DEFAULT_TOLERANCES: Tolerances = Tolerances {
    default: 1e-9,
    ewma: 1e-6,
    auc: 5e-3,
    crossover: 1e-6,
    precision_sync_per_dimension: 1e-6,
    standardisation: 0.14,
    platt_log_sharpness: 0.06,
    bit_identical: 0.0,
};

impl Default for Tolerances {
    fn default() -> Self {
        DEFAULT_TOLERANCES
    }
}
