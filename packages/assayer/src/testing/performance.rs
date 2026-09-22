// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Guarded performance populations built through the shared scenario.
//!
//! [`PERFORMANCE_CASES`] is the single declaration of the six workloads. The
//! constructors in this module translate those rows into [`World`] scenarios,
//! and every translation returns only after checking the state it established.

use std::collections::HashSet;
use std::path::PathBuf;

use torrust_sentinel::BatchReport;

use super::{
    Clock as _, CompetitiveCellId, CompetitiveCellSpec, FeatureSelector, InteractionTemplate, LabelSpec, PreSeedSpec,
    RuntimeLayout, VirtualClock, World, WorldBuildError, WorldFixtureError, make_cell_report, minimal_report,
};
use crate::api::{PreSeedEntry, PreSeedResult};
use crate::config::types::AssayerConfig;
use crate::error::{LabelError, LifecycleError, ReportError};
use crate::resonance::channel::{ChannelPolicy, RewardParameters};
use crate::signal::{Persistence, SignalDeclaration, SignalShape};
use crate::types::{Action, IdentityBudget, PersistentTimestamp};

/// A performance workload selected by the next benchmark target.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PerformanceCase {
    /// Fresh engine with no established population.
    Cold,
    /// Settled reference-width population.
    ReferenceSteady,
    /// Dense signal and outcome-axis label publication.
    DenseLabelProfile,
    /// Synchronous bulk pre-seed input.
    PreseedThousand,
    /// Settled reference population for health queries.
    HealthReference,
    /// Trained reference population before one Sentinel leaves.
    ReferenceRemoval,
}

/// Population and preparation declared for one case.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PopulationDeclaration {
    /// Reporting Sentinels present before measurement.
    pub reporting_sentinels: usize,
    /// Spatial outcome axes present before measurement.
    pub spatial_outcome_axes: usize,
    /// Non-spatial outcome axes present before measurement.
    pub non_spatial_outcome_axes: usize,
    /// Identity dimensions present before measurement.
    pub identity_dimensions: usize,
    /// Competitive cells established in each identity dimension.
    pub competitive_cells_per_identity: usize,
    /// Real assessments used to establish each competitive set.
    pub identity_observations_per_dimension: usize,
    /// Labels applied before the measured population.
    pub warmup_labels: usize,
    /// Labels in the measured population, or settled before a non-label sample.
    pub labels: usize,
    /// Entries in the synchronous pre-seed population.
    pub preseed_entries: usize,
}

/// Dimension declaration guarded by a case constructor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DimensionDeclaration {
    /// Complete published block layout.
    pub runtime: RuntimeLayout,
    /// Width removed with the departing Sentinel, when applicable.
    pub departing_block_width: Option<usize>,
    /// Width retained after removal, when applicable.
    pub retained_width: Option<usize>,
    /// Full-health model rows required by the case, when applicable.
    pub health_model_rows: Option<usize>,
}

/// Declared label or pre-seed composition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MixDeclaration {
    /// Total labels or entries in the declared population.
    pub items: usize,
    /// Adverse labels or entries.
    pub adverse: usize,
    /// Benign labels or entries.
    pub benign: usize,
    /// Items eligible for the model-update path.
    pub eligible: usize,
    /// Items marked as ground truth.
    pub ground_truth: usize,
    /// Outcome axes present on every item.
    pub outcome_axis_width: usize,
    /// Distinct entities represented by the population.
    pub entity_cardinality: usize,
    /// Period selecting adverse entries; `None` denotes an empty population.
    pub adverse_stride: Option<usize>,
}

/// Synthetic report declaration for populated reference cases.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SyntheticReportDeclaration {
    /// Number of reporting Sentinels.
    pub sentinel_count: usize,
    /// Published feature width of each Sentinel slot.
    pub slot_width: usize,
    /// Structurally acknowledged cells in each report.
    pub cell_count: usize,
    /// Outcome-axis width visible beside each report.
    pub outcome_axis_width: usize,
}

/// One row of the performance case table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PerformanceCaseDeclaration {
    /// Case selected by this row.
    pub case: PerformanceCase,
    /// Population and preparation.
    pub population: PopulationDeclaration,
    /// Published dimensions and lifecycle widths.
    pub dimensions: DimensionDeclaration,
    /// Label or entry composition.
    pub mix: MixDeclaration,
    /// Synthetic report shape, when the case reports.
    pub reports: Option<SyntheticReportDeclaration>,
    /// Ordered channel actions admitted by the case.
    pub channel_policy: [Action; 3],
}

const PERFORMANCE_CHANNEL_POLICY: [Action; 3] = [Action::Allow, Action::Challenge, Action::Block];

const COLD_LAYOUT: RuntimeLayout = RuntimeLayout {
    dimension_map_width: 16,
    bias_width: 1,
    aggregate_width: 15,
    identity_dimensions_width: 0,
    identity_cross_dimension_width: 0,
    signal_width: 0,
    sentinel_slots_width: 0,
    interaction_width: 0,
    competitive_cells_width: 0,
};

const REFERENCE_LAYOUT: RuntimeLayout = RuntimeLayout {
    dimension_map_width: 638,
    bias_width: 1,
    aggregate_width: 15,
    identity_dimensions_width: 22,
    identity_cross_dimension_width: 8,
    signal_width: 0,
    sentinel_slots_width: 504,
    interaction_width: 68,
    competitive_cells_width: 20,
};

const DENSE_LAYOUT: RuntimeLayout = RuntimeLayout {
    dimension_map_width: 646,
    bias_width: 1,
    aggregate_width: 15,
    identity_dimensions_width: 0,
    identity_cross_dimension_width: 0,
    signal_width: 630,
    sentinel_slots_width: 0,
    interaction_width: 0,
    competitive_cells_width: 0,
};

/// The only declaration of the six performance workloads.
pub const PERFORMANCE_CASES: [PerformanceCaseDeclaration; 6] = [
    PerformanceCaseDeclaration {
        case: PerformanceCase::Cold,
        population: PopulationDeclaration {
            reporting_sentinels: 0,
            spatial_outcome_axes: 0,
            non_spatial_outcome_axes: 0,
            identity_dimensions: 0,
            competitive_cells_per_identity: 0,
            identity_observations_per_dimension: 0,
            warmup_labels: 0,
            labels: 0,
            preseed_entries: 0,
        },
        dimensions: DimensionDeclaration {
            runtime: COLD_LAYOUT,
            departing_block_width: None,
            retained_width: None,
            health_model_rows: None,
        },
        mix: MixDeclaration {
            items: 0,
            adverse: 0,
            benign: 0,
            eligible: 0,
            ground_truth: 0,
            outcome_axis_width: 0,
            entity_cardinality: 0,
            adverse_stride: None,
        },
        reports: None,
        channel_policy: PERFORMANCE_CHANNEL_POLICY,
    },
    PerformanceCaseDeclaration {
        case: PerformanceCase::ReferenceSteady,
        population: PopulationDeclaration {
            reporting_sentinels: 8,
            spatial_outcome_axes: 1,
            non_spatial_outcome_axes: 0,
            identity_dimensions: 2,
            competitive_cells_per_identity: 10,
            identity_observations_per_dimension: 110,
            warmup_labels: 0,
            labels: 1_276,
            preseed_entries: 0,
        },
        dimensions: DimensionDeclaration {
            runtime: REFERENCE_LAYOUT,
            departing_block_width: Some(68),
            retained_width: None,
            health_model_rows: None,
        },
        mix: MixDeclaration {
            items: 1_276,
            adverse: 638,
            benign: 638,
            eligible: 1_276,
            ground_truth: 1_276,
            outcome_axis_width: 1,
            entity_cardinality: 1,
            adverse_stride: Some(2),
        },
        reports: Some(SyntheticReportDeclaration {
            sentinel_count: 8,
            slot_width: 63,
            cell_count: 1,
            outcome_axis_width: 1,
        }),
        channel_policy: PERFORMANCE_CHANNEL_POLICY,
    },
    PerformanceCaseDeclaration {
        case: PerformanceCase::DenseLabelProfile,
        population: PopulationDeclaration {
            reporting_sentinels: 0,
            spatial_outcome_axes: 0,
            non_spatial_outcome_axes: 5,
            identity_dimensions: 0,
            competitive_cells_per_identity: 0,
            identity_observations_per_dimension: 0,
            warmup_labels: 8,
            labels: 128,
            preseed_entries: 0,
        },
        dimensions: DimensionDeclaration {
            runtime: DENSE_LAYOUT,
            departing_block_width: None,
            retained_width: None,
            health_model_rows: None,
        },
        mix: MixDeclaration {
            items: 136,
            adverse: 136,
            benign: 0,
            eligible: 136,
            ground_truth: 136,
            outcome_axis_width: 5,
            entity_cardinality: 1,
            adverse_stride: Some(1),
        },
        reports: None,
        channel_policy: PERFORMANCE_CHANNEL_POLICY,
    },
    PerformanceCaseDeclaration {
        case: PerformanceCase::PreseedThousand,
        population: PopulationDeclaration {
            reporting_sentinels: 0,
            spatial_outcome_axes: 0,
            non_spatial_outcome_axes: 0,
            identity_dimensions: 0,
            competitive_cells_per_identity: 0,
            identity_observations_per_dimension: 0,
            warmup_labels: 0,
            labels: 0,
            preseed_entries: 1_000,
        },
        dimensions: DimensionDeclaration {
            runtime: COLD_LAYOUT,
            departing_block_width: None,
            retained_width: None,
            health_model_rows: None,
        },
        mix: MixDeclaration {
            items: 1_000,
            adverse: 200,
            benign: 800,
            eligible: 1_000,
            ground_truth: 1_000,
            outcome_axis_width: 0,
            entity_cardinality: 1_000,
            adverse_stride: Some(5),
        },
        reports: None,
        channel_policy: PERFORMANCE_CHANNEL_POLICY,
    },
    PerformanceCaseDeclaration {
        case: PerformanceCase::HealthReference,
        population: PopulationDeclaration {
            reporting_sentinels: 8,
            spatial_outcome_axes: 1,
            non_spatial_outcome_axes: 0,
            identity_dimensions: 2,
            competitive_cells_per_identity: 10,
            identity_observations_per_dimension: 110,
            warmup_labels: 0,
            labels: 1_276,
            preseed_entries: 0,
        },
        dimensions: DimensionDeclaration {
            runtime: REFERENCE_LAYOUT,
            departing_block_width: Some(68),
            retained_width: None,
            health_model_rows: Some(4),
        },
        mix: MixDeclaration {
            items: 1_276,
            adverse: 638,
            benign: 638,
            eligible: 1_276,
            ground_truth: 1_276,
            outcome_axis_width: 1,
            entity_cardinality: 1,
            adverse_stride: Some(2),
        },
        reports: Some(SyntheticReportDeclaration {
            sentinel_count: 8,
            slot_width: 63,
            cell_count: 1,
            outcome_axis_width: 1,
        }),
        channel_policy: PERFORMANCE_CHANNEL_POLICY,
    },
    PerformanceCaseDeclaration {
        case: PerformanceCase::ReferenceRemoval,
        population: PopulationDeclaration {
            reporting_sentinels: 8,
            spatial_outcome_axes: 1,
            non_spatial_outcome_axes: 0,
            identity_dimensions: 2,
            competitive_cells_per_identity: 10,
            identity_observations_per_dimension: 110,
            warmup_labels: 0,
            labels: 50,
            preseed_entries: 0,
        },
        dimensions: DimensionDeclaration {
            runtime: REFERENCE_LAYOUT,
            departing_block_width: Some(68),
            retained_width: Some(570),
            health_model_rows: None,
        },
        mix: MixDeclaration {
            items: 50,
            adverse: 25,
            benign: 25,
            eligible: 50,
            ground_truth: 50,
            outcome_axis_width: 1,
            entity_cardinality: 1,
            adverse_stride: Some(2),
        },
        reports: Some(SyntheticReportDeclaration {
            sentinel_count: 8,
            slot_width: 63,
            cell_count: 1,
            outcome_axis_width: 1,
        }),
        channel_policy: PERFORMANCE_CHANNEL_POLICY,
    },
];

impl PerformanceCase {
    /// Returns this case's row from [`PERFORMANCE_CASES`].
    #[must_use]
    pub fn declaration(self) -> &'static PerformanceCaseDeclaration {
        PERFORMANCE_CASES
            .iter()
            .find(|row| row.case == self)
            .expect("every performance case has exactly one declared row")
    }

    /// Constructs and guards this case's real-engine world.
    ///
    /// # Errors
    ///
    /// Returns [`PerformanceFixtureError`] when construction, preparation, or
    /// any measured precondition differs from the selected row.
    pub fn world(self, instance_id: &str) -> Result<World, PerformanceFixtureError> {
        match self {
            Self::Cold => build_cold_world(self, instance_id),
            Self::ReferenceSteady | Self::HealthReference => build_settled_reference_world(self, instance_id),
            Self::DenseLabelProfile => build_dense_world(self, instance_id),
            Self::PreseedThousand => build_preseed_world(self, instance_id),
            Self::ReferenceRemoval => reference_removal_world(instance_id),
        }
    }
}

/// Counts and composition measured for a label or pre-seed population.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PopulationSummary {
    /// Total labels or entries.
    pub items: usize,
    /// Adverse items.
    pub adverse: usize,
    /// Benign items.
    pub benign: usize,
    /// Eligible items.
    pub eligible: usize,
    /// Ground-truth items.
    pub ground_truth: usize,
    /// Outcome values carried across all items.
    pub outcome_values: usize,
    /// Distinct entities represented.
    pub entity_cardinality: usize,
}

/// State observed after installing synthetic reports.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SyntheticReportObservation {
    /// Registered Sentinels with an acknowledged report.
    pub sentinel_count: usize,
    /// Width observed for each Sentinel slot.
    pub slot_width: usize,
    /// Cells acknowledged in each report.
    pub cell_count: usize,
    /// Registered outcome axes.
    pub outcome_axis_width: usize,
    /// Reports acknowledged structurally.
    pub acknowledgements: usize,
    /// Cells whose values required sanitisation.
    pub degraded_cells: usize,
}

/// Health fields that must remain clean before measurement.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HealthObservation {
    /// Signal values sanitised by assessment.
    pub signals_sanitised: u64,
    /// Signal shapes rejected by assessment.
    pub signals_shape_mismatched: u64,
    /// Unknown signal values ignored by assessment.
    pub signals_unknown: u64,
    /// Sentinel slots repaired after non-finite extraction.
    pub sentinel_slots_zeroed: u64,
    /// Assembled features repaired before model input.
    pub features_sanitised: u64,
    /// Model reads that fell back to their prior.
    pub model_fallbacks: u64,
    /// Whether a Cholesky cascade reached its terminus.
    pub cascade_terminus: bool,
    /// Whether the label owner stopped accepting work.
    pub label_path_stopped: bool,
    /// Health events lost to a full channel.
    pub health_events_dropped: u64,
}

/// Preconditions observed on a sacrificial reference removal.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ReferenceRemovalObservation {
    /// Width before removal.
    pub starting_width: usize,
    /// Coordinates owned by the departing Sentinel and its interactions.
    pub departing_block_width: usize,
    /// Eligible labels applied before removal.
    pub trained_labels: u64,
    /// Width after removal.
    pub retained_width: usize,
    /// Marginalisation events emitted by the removal.
    pub marginalisation_events: u64,
    /// Marginalisations that discarded their correction.
    pub corrections_skipped: u64,
}

/// Typed failures from performance fixture construction and guards.
#[derive(Debug)]
pub enum PerformanceFixtureError {
    /// A world could not be built.
    WorldBuild(WorldBuildError),
    /// A scenario fixture or runtime layout guard failed.
    WorldFixture(WorldFixtureError),
    /// A lifecycle registration or removal failed.
    Lifecycle(LifecycleError),
    /// A Sentinel report was rejected.
    Report(ReportError),
    /// A label or pre-seed operation failed.
    Label(LabelError),
    /// The fixed clock did not begin at the declared origin.
    ClockOrigin {
        /// Declared origin.
        expected: PersistentTimestamp,
        /// Measured origin.
        observed: PersistentTimestamp,
    },
    /// The fixed clock moved during a sample.
    ClockMoved {
        /// Reading at sample entry.
        before: PersistentTimestamp,
        /// Reading at sample exit.
        after: PersistentTimestamp,
    },
    /// Durable paths were configured for a no-op persistence case.
    PersistenceConfigured {
        /// Configured checkpoint directory.
        checkpoint_dir: PathBuf,
        /// Configured journal directory.
        journal_dir: PathBuf,
    },
    /// The constructed channel action order differed from its row.
    ChannelPolicyMismatch {
        /// Actions declared by the case table.
        expected: Vec<Action>,
        /// Actions observed on the constructed policy.
        observed: Vec<Action>,
    },
    /// A seeded population differed from its row.
    PopulationMismatch {
        /// Summary derived from the row.
        expected: PopulationSummary,
        /// Summary measured from generated items or completed publication.
        observed: PopulationSummary,
    },
    /// A synthetic report population differed from its declaration.
    SyntheticReportMismatch {
        /// Declaration translated into observable counts.
        expected: SyntheticReportObservation,
        /// Counts measured after ingestion.
        observed: SyntheticReportObservation,
    },
    /// A synthetic report declaration exceeded the fixture vocabulary.
    SyntheticReportUnsupported {
        /// Requested Sentinel count.
        sentinel_count: usize,
        /// Requested cell count.
        cell_count: usize,
    },
    /// Health was not clean after preparation.
    HealthNotClean {
        /// Every field checked by the guard.
        observed: HealthObservation,
    },
    /// The full health report omitted or added model rows.
    HealthModelRows {
        /// Declared row count.
        expected: usize,
        /// Measured row count.
        observed: usize,
    },
    /// A pre-seed call did not complete the whole guarded population.
    PreSeedIncomplete {
        /// Declared entry count.
        expected: usize,
        /// Entries reported as processed.
        processed: usize,
        /// Entries reported as failed.
        failed: usize,
    },
    /// Reference removal did not satisfy every lifecycle precondition.
    ReferenceRemovalMismatch {
        /// Preconditions derived from the case row.
        expected: ReferenceRemovalObservation,
        /// Preconditions measured on the sacrificial removal.
        observed: ReferenceRemovalObservation,
    },
}

impl From<WorldBuildError> for PerformanceFixtureError {
    fn from(error: WorldBuildError) -> Self {
        Self::WorldBuild(error)
    }
}

impl From<WorldFixtureError> for PerformanceFixtureError {
    fn from(error: WorldFixtureError) -> Self {
        Self::WorldFixture(error)
    }
}

impl From<LifecycleError> for PerformanceFixtureError {
    fn from(error: LifecycleError) -> Self {
        Self::Lifecycle(error)
    }
}

impl From<ReportError> for PerformanceFixtureError {
    fn from(error: ReportError) -> Self {
        Self::Report(error)
    }
}

impl From<LabelError> for PerformanceFixtureError {
    fn from(error: LabelError) -> Self {
        Self::Label(error)
    }
}

impl std::fmt::Display for PerformanceFixtureError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WorldBuild(error) => write!(formatter, "performance world construction failed: {error}"),
            Self::WorldFixture(error) => write!(formatter, "performance world guard failed: {error}"),
            Self::Lifecycle(error) => write!(formatter, "performance lifecycle setup failed: {error}"),
            Self::Report(error) => write!(formatter, "synthetic report setup failed: {error}"),
            Self::Label(error) => write!(formatter, "seeded population setup failed: {error}"),
            Self::ClockOrigin { expected, observed } => {
                write!(formatter, "fixed clock began at {observed:?}, expected {expected:?}")
            }
            Self::ClockMoved { before, after } => {
                write!(formatter, "fixed clock moved from {before:?} to {after:?} during the sample")
            }
            Self::PersistenceConfigured {
                checkpoint_dir,
                journal_dir,
            } => {
                formatter.write_str("no-op persistence case configured checkpoint ")?;
                std::fmt::Debug::fmt(checkpoint_dir, formatter)?;
                formatter.write_str(" and journal ")?;
                std::fmt::Debug::fmt(journal_dir, formatter)
            }
            Self::ChannelPolicyMismatch { expected, observed } => {
                write!(
                    formatter,
                    "channel actions {observed:?} differ from declared actions {expected:?}"
                )
            }
            Self::PopulationMismatch { expected, observed } => {
                write!(formatter, "population {observed:?} differs from declaration {expected:?}")
            }
            Self::SyntheticReportMismatch { expected, observed } => {
                write!(
                    formatter,
                    "synthetic reports {observed:?} differ from declaration {expected:?}"
                )
            }
            Self::SyntheticReportUnsupported {
                sentinel_count,
                cell_count,
            } => write!(
                formatter,
                "synthetic report declaration requests {sentinel_count} Sentinels and {cell_count} cells"
            ),
            Self::HealthNotClean { observed } => write!(formatter, "fixture health is not clean: {observed:?}"),
            Self::HealthModelRows { expected, observed } => {
                write!(formatter, "full health report has {observed} model rows, expected {expected}")
            }
            Self::PreSeedIncomplete {
                expected,
                processed,
                failed,
            } => write!(
                formatter,
                "pre-seed processed {processed} entries with {failed} failures, expected {expected} completed entries"
            ),
            Self::ReferenceRemovalMismatch { expected, observed } => {
                write!(
                    formatter,
                    "reference removal {observed:?} differs from preconditions {expected:?}"
                )
            }
        }
    }
}

impl std::error::Error for PerformanceFixtureError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::WorldBuild(error) => Some(error),
            Self::WorldFixture(error) => Some(error),
            Self::Lifecycle(error) => Some(error),
            Self::Report(error) => Some(error),
            Self::Label(error) => Some(error),
            Self::ClockOrigin { .. }
            | Self::ClockMoved { .. }
            | Self::PersistenceConfigured { .. }
            | Self::ChannelPolicyMismatch { .. }
            | Self::PopulationMismatch { .. }
            | Self::SyntheticReportMismatch { .. }
            | Self::SyntheticReportUnsupported { .. }
            | Self::HealthNotClean { .. }
            | Self::HealthModelRows { .. }
            | Self::PreSeedIncomplete { .. }
            | Self::ReferenceRemovalMismatch { .. } => None,
        }
    }
}

/// Runs one operation while guarding the existing world's fixed clock.
///
/// # Errors
///
/// Returns [`PerformanceFixtureError::ClockOrigin`] when the clock did not
/// begin at the harness epoch, or [`PerformanceFixtureError::ClockMoved`] when
/// the operation advanced it.
pub fn with_fixed_clock_sample<T>(world: &World, operation: impl FnOnce(&World) -> T) -> Result<T, PerformanceFixtureError> {
    let expected = PersistentTimestamp::new(
        i64::try_from(VirtualClock::EPOCH_SECS).expect("the harness epoch fits in a signed second count"),
        0,
    );
    let before = world.clock().now();
    if before != expected {
        return Err(PerformanceFixtureError::ClockOrigin {
            expected,
            observed: before,
        });
    }
    let output = operation(world);
    let after = world.clock().now();
    if after != before {
        return Err(PerformanceFixtureError::ClockMoved { before, after });
    }
    Ok(output)
}

/// Guards that a configuration selects the existing no-persistence mode.
///
/// # Errors
///
/// Returns [`PerformanceFixtureError::PersistenceConfigured`] with both
/// durable paths when persistence is enabled.
pub fn guard_no_persistence(config: &AssayerConfig) -> Result<(), PerformanceFixtureError> {
    if let Some(persistence) = &config.persistence {
        return Err(PerformanceFixtureError::PersistenceConfigured {
            checkpoint_dir: persistence.checkpoint_dir.clone(),
            journal_dir: persistence.journal_dir.clone(),
        });
    }
    Ok(())
}

/// Guards the three-action policy declared by every case row.
///
/// # Errors
///
/// Returns [`PerformanceFixtureError::ChannelPolicyMismatch`] with both action
/// sequences when the generated policy differs.
pub fn guard_channel_policy(declared: &[Action], policy: &ChannelPolicy) -> Result<(), PerformanceFixtureError> {
    let expected = declared.to_vec();
    if policy.actions != expected {
        return Err(PerformanceFixtureError::ChannelPolicyMismatch {
            expected,
            observed: policy.actions.clone(),
        });
    }
    Ok(())
}

/// Guards a population summary against its case-table mix.
///
/// # Errors
///
/// Returns [`PerformanceFixtureError::PopulationMismatch`] with all measured
/// counts when any count or mix differs.
pub fn guard_population(declaration: MixDeclaration, observed: PopulationSummary) -> Result<(), PerformanceFixtureError> {
    let expected = PopulationSummary {
        items: declaration.items,
        adverse: declaration.adverse,
        benign: declaration.benign,
        eligible: declaration.eligible,
        ground_truth: declaration.ground_truth,
        outcome_values: declaration.items * declaration.outcome_axis_width,
        entity_cardinality: declaration.entity_cardinality,
    };
    if observed != expected {
        return Err(PerformanceFixtureError::PopulationMismatch { expected, observed });
    }
    Ok(())
}

/// Guards every structural report acknowledgement and declared width.
///
/// # Errors
///
/// Returns [`PerformanceFixtureError::SyntheticReportMismatch`] with the full
/// expected and measured report observations on any difference.
pub fn guard_synthetic_reports(
    declaration: SyntheticReportDeclaration,
    observed: SyntheticReportObservation,
) -> Result<(), PerformanceFixtureError> {
    let expected = SyntheticReportObservation {
        sentinel_count: declaration.sentinel_count,
        slot_width: declaration.slot_width,
        cell_count: declaration.cell_count,
        outcome_axis_width: declaration.outcome_axis_width,
        acknowledgements: declaration.sentinel_count,
        degraded_cells: 0,
    };
    if observed != expected {
        return Err(PerformanceFixtureError::SyntheticReportMismatch { expected, observed });
    }
    Ok(())
}

/// Guards that preparation introduced no degradation or stopped path.
///
/// # Errors
///
/// Returns [`PerformanceFixtureError::HealthNotClean`] carrying every checked
/// field when any field is non-clean.
pub fn guard_clean_health(observed: HealthObservation) -> Result<(), PerformanceFixtureError> {
    if observed != HealthObservation::default() {
        return Err(PerformanceFixtureError::HealthNotClean { observed });
    }
    Ok(())
}

/// Guards full-health model-row coverage.
///
/// # Errors
///
/// Returns [`PerformanceFixtureError::HealthModelRows`] with both counts when
/// the complete report does not contain the declared model population.
pub const fn guard_health_model_rows(expected: usize, observed: usize) -> Result<(), PerformanceFixtureError> {
    if observed != expected {
        return Err(PerformanceFixtureError::HealthModelRows { expected, observed });
    }
    Ok(())
}

/// Guards completion of one synchronous pre-seed call.
///
/// # Errors
///
/// Returns [`PerformanceFixtureError::PreSeedIncomplete`] with the declared,
/// processed, and failed counts when the result is incomplete.
pub const fn guard_pre_seed_result(expected: usize, result: &PreSeedResult) -> Result<(), PerformanceFixtureError> {
    if result.processed != expected || result.failed != 0 {
        return Err(PerformanceFixtureError::PreSeedIncomplete {
            expected,
            processed: result.processed,
            failed: result.failed,
        });
    }
    Ok(())
}

/// Guards the complete reference-removal precondition set.
///
/// # Errors
///
/// Returns [`PerformanceFixtureError::ReferenceRemovalMismatch`] with all
/// measured values unless widths match, training is non-trivial, at least one
/// marginalisation ran, and no correction fell back.
pub fn guard_reference_removal(
    dimensions: DimensionDeclaration,
    declared_training: usize,
    observed: ReferenceRemovalObservation,
) -> Result<(), PerformanceFixtureError> {
    let expected = ReferenceRemovalObservation {
        starting_width: dimensions.runtime.dimension_map_width,
        departing_block_width: dimensions.departing_block_width.unwrap_or_default(),
        trained_labels: u64::try_from(declared_training).expect("declared training count fits in u64"),
        retained_width: dimensions.retained_width.unwrap_or_default(),
        marginalisation_events: 1,
        corrections_skipped: 0,
    };
    let valid = observed.starting_width == expected.starting_width
        && observed.departing_block_width == expected.departing_block_width
        && observed.trained_labels >= expected.trained_labels
        && observed.trained_labels > 0
        && observed.retained_width == expected.retained_width
        && observed.marginalisation_events >= expected.marginalisation_events
        && observed.corrections_skipped == expected.corrections_skipped;
    if !valid {
        return Err(PerformanceFixtureError::ReferenceRemovalMismatch { expected, observed });
    }
    Ok(())
}

fn performance_policy(declaration: &PerformanceCaseDeclaration) -> Result<ChannelPolicy, PerformanceFixtureError> {
    let policy = ChannelPolicy {
        actions: declaration.channel_policy.to_vec(),
        reward: RewardParameters::default(),
        ..ChannelPolicy::default()
    };
    guard_channel_policy(&declaration.channel_policy, &policy)?;
    Ok(policy)
}

fn performance_config(instance_id: &str) -> Result<AssayerConfig, PerformanceFixtureError> {
    let config = AssayerConfig {
        instance_id: instance_id.to_owned(),
        infrastructure: super::test_infrastructure(),
        ..Default::default()
    };
    guard_no_persistence(&config)?;
    Ok(config)
}

fn health_observation(world: &World) -> HealthObservation {
    let health = world.assayer().health_summary();
    HealthObservation {
        signals_sanitised: health.degradation.signals_sanitised,
        signals_shape_mismatched: health.degradation.signals_shape_mismatched,
        signals_unknown: health.degradation.signals_unknown,
        sentinel_slots_zeroed: health.degradation.sentinel_slots_zeroed,
        features_sanitised: health.degradation.features_sanitised,
        model_fallbacks: health.degradation.model_fallbacks,
        cascade_terminus: health.any_cascade_terminus,
        label_path_stopped: health.label_path_stopped,
        health_events_dropped: health.health_events_dropped,
    }
}

const SENTINEL_NAMES: [&str; 8] = ["S1", "S2", "S3", "S4", "S5", "S6", "S7", "S8"];
const AXIS_NAMES: [&str; 5] = ["outcome-1", "outcome-2", "outcome-3", "outcome-4", "outcome-5"];

fn reference_templates() -> Vec<InteractionTemplate> {
    let mut templates: Vec<_> = (0..5)
        .map(|offset| InteractionTemplate::type1(offset, FeatureSelector::AggregateFeature(0)))
        .collect();
    templates.extend((0..8).map(|offset| InteractionTemplate::type4(offset, FeatureSelector::AggregateFeature(0))));
    templates.push(InteractionTemplate::type5(FeatureSelector::AggregateFeature(0)));
    templates
}

fn reference_cells(entity: &str, declaration: PopulationDeclaration) -> CompetitiveCellSpec {
    let expected = vec![
        CompetitiveCellId::new(0, 0),
        CompetitiveCellId::new(0, 1),
        CompetitiveCellId::new(0, 2),
        CompetitiveCellId::new(0, 3),
        CompetitiveCellId::new(0, 4),
        CompetitiveCellId::new(0, 5),
        CompetitiveCellId::new((1_u128 << 123) - 1, 5),
        CompetitiveCellId::new((1_u128 << 124) - 1, 4),
        CompetitiveCellId::new((1_u128 << 126) - 1, 2),
        CompetitiveCellId::new((1_u128 << 127) - 1, 1),
    ];
    debug_assert_eq!(expected.len(), declaration.competitive_cells_per_identity);
    let mut budget = IdentityBudget::for_depth_cutoff(10);
    budget.max_cells = 19;
    budget.depth_create = 3;
    budget.depth_evict = 4;
    CompetitiveCellSpec::new(expected)
        .budget(budget)
        .observe_n(World::entity(entity), declaration.identity_observations_per_dimension)
}

/// Deterministically generated label population for one declared case.
#[derive(Clone, Copy, Debug)]
pub struct SeededLabelPopulation {
    /// Case-table row that owns every count and mix.
    declaration: &'static PerformanceCaseDeclaration,
}

impl SeededLabelPopulation {
    /// Builds and guards the label population declared for `case`.
    ///
    /// # Errors
    ///
    /// Returns [`PerformanceFixtureError::PopulationMismatch`] if generation
    /// does not reproduce every count and mix in the case row.
    pub fn for_case(case: PerformanceCase) -> Result<Self, PerformanceFixtureError> {
        let population = Self {
            declaration: case.declaration(),
        };
        guard_population(population.declaration.mix, population.summary())?;
        Ok(population)
    }

    /// Returns the complete generated population's measured composition.
    #[must_use]
    pub fn summary(&self) -> PopulationSummary {
        self.summary_for_range(0, self.declaration.mix.items)
    }

    /// Publishes the declared warm-up prefix and crosses the label barrier.
    ///
    /// # Errors
    ///
    /// Returns a typed fixture error when a label is refused, publication does
    /// not complete, or the applied counts and mix differ from the prefix.
    pub fn publish_warmup(&self, world: &World) -> Result<PopulationSummary, PerformanceFixtureError> {
        self.publish_range(world, 0, self.declaration.population.warmup_labels)
    }

    /// Publishes the declared measured population and crosses the label barrier.
    ///
    /// The caller decides whether the call itself is inside a measured region;
    /// this method keeps its completion barrier with the work.
    ///
    /// # Errors
    ///
    /// Returns a typed fixture error when a label is refused, publication does
    /// not complete, or the applied counts and mix differ from the row.
    pub fn publish_measured(&self, world: &World) -> Result<PopulationSummary, PerformanceFixtureError> {
        self.publish_range(
            world,
            self.declaration.population.warmup_labels,
            self.declaration.population.labels,
        )
    }

    fn summary_for_range(self, start: usize, count: usize) -> PopulationSummary {
        let mut summary = PopulationSummary::default();
        let mut entities = HashSet::new();
        for index in start..start + count {
            let adverse = self.is_adverse(index);
            summary.items += 1;
            summary.adverse += usize::from(adverse);
            summary.benign += usize::from(!adverse);
            summary.eligible += usize::from(index < self.declaration.mix.ground_truth);
            summary.ground_truth += usize::from(index < self.declaration.mix.ground_truth);
            summary.outcome_values += self.declaration.mix.outcome_axis_width;
            entities.insert(self.entity_index(index));
        }
        summary.entity_cardinality = entities.len();
        summary
    }

    fn is_adverse(self, index: usize) -> bool {
        self.declaration
            .mix
            .adverse_stride
            .is_some_and(|stride| index.is_multiple_of(stride))
    }

    const fn entity_index(self, index: usize) -> usize {
        if self.declaration.mix.entity_cardinality == 0 {
            0
        } else {
            index % self.declaration.mix.entity_cardinality
        }
    }

    fn publish_range(self, world: &World, start: usize, count: usize) -> Result<PopulationSummary, PerformanceFixtureError> {
        let expected = self.summary_for_range(start, count);
        let before = world.assayer().health_summary();
        let mut observed = PopulationSummary::default();
        let mut entities = HashSet::new();

        for index in start..start + count {
            let entity_index = self.entity_index(index);
            let entity = format!("performance-entity-{entity_index}");
            entities.insert(entity_index);
            let mut request = world.request("default", &entity);
            if self.declaration.dimensions.runtime.signal_width > 0 {
                let values: Vec<f64> = (0..self.declaration.dimensions.runtime.signal_width)
                    .map(|position| if position.is_multiple_of(2) { 0.25 } else { -0.25 })
                    .collect();
                request = request.with_signal("dense-signal", values);
            }
            for name in SENTINEL_NAMES.iter().take(self.declaration.population.reporting_sentinels) {
                let sentinel = world.sentinel(*name).expect("declared reporting Sentinel is registered");
                request = request.with_sentinel(sentinel, super::GOLDEN_COORD);
            }

            let assessment = world.core_assess(request);
            let adverse = self.is_adverse(index);
            let mut spec = if adverse {
                LabelSpec::adverse(assessment.id)
            } else {
                LabelSpec::benign(assessment.id)
            };
            if index < self.declaration.mix.ground_truth {
                spec = spec.ground_truth();
            }
            for name in AXIS_NAMES.iter().take(self.declaration.mix.outcome_axis_width) {
                let axis = world.axis(*name).expect("declared outcome axis is registered");
                spec = spec.outcome(axis, if adverse { 0.75 } else { -0.75 });
            }
            world.label(spec.build())?;
            observed.items += 1;
            observed.adverse += usize::from(adverse);
            observed.benign += usize::from(!adverse);
            observed.ground_truth += usize::from(index < self.declaration.mix.ground_truth);
            observed.outcome_values += self.declaration.mix.outcome_axis_width;
        }
        world.flush_labels()?;
        let after = world.assayer().health_summary();
        observed.eligible = usize::try_from(after.eligible_labels.saturating_sub(before.eligible_labels))
            .expect("eligible label delta fits in usize");
        observed.entity_cardinality = entities.len();
        if observed != expected {
            return Err(PerformanceFixtureError::PopulationMismatch { expected, observed });
        }
        Ok(observed)
    }
}

/// Deterministically generated pre-seed population for the bulk case.
#[derive(Clone, Copy, Debug)]
pub struct SeededPreSeedPopulation {
    /// Case-table row that owns every count and mix.
    declaration: &'static PerformanceCaseDeclaration,
}

impl SeededPreSeedPopulation {
    /// Builds and guards the pre-seed population declared for `case`.
    ///
    /// # Errors
    ///
    /// Returns [`PerformanceFixtureError::PopulationMismatch`] if generation
    /// does not reproduce every count and mix in the case row.
    pub fn for_case(case: PerformanceCase) -> Result<Self, PerformanceFixtureError> {
        let population = Self {
            declaration: case.declaration(),
        };
        guard_population(population.declaration.mix, population.summary())?;
        Ok(population)
    }

    /// Returns the complete generated population's measured composition.
    #[must_use]
    pub fn summary(&self) -> PopulationSummary {
        let labels = SeededLabelPopulation {
            declaration: self.declaration,
        };
        labels.summary()
    }

    /// Materialises the declared entries against the world's channel.
    ///
    /// # Errors
    ///
    /// Returns [`PerformanceFixtureError::PopulationMismatch`] if the
    /// materialised entries do not retain the declared mix.
    pub fn entries(&self, world: &World) -> Result<Vec<PreSeedEntry>, PerformanceFixtureError> {
        let channel = world
            .channel("default")
            .expect("performance world declares the default channel");
        let labels = SeededLabelPopulation {
            declaration: self.declaration,
        };
        let mut entries = Vec::with_capacity(self.declaration.population.preseed_entries);
        for index in 0..self.declaration.population.preseed_entries {
            let entity = World::entity(&format!("performance-preseed-{}", labels.entity_index(index)));
            let mut spec = if labels.is_adverse(index) {
                PreSeedSpec::adverse(channel, entity)
            } else {
                PreSeedSpec::benign(channel, entity)
            };
            if index < self.declaration.mix.ground_truth {
                spec = spec.ground_truth();
            }
            entries.push(spec.build());
        }
        guard_population(self.declaration.mix, self.summary())?;
        Ok(entries)
    }

    /// Applies the population through the synchronous production pre-seed API.
    ///
    /// # Errors
    ///
    /// Returns a typed fixture error if generation, submission, completion, or
    /// health differs from the declaration.
    pub fn apply(&self, world: &World) -> Result<PreSeedResult, PerformanceFixtureError> {
        let entries = self.entries(world)?;
        let result = world.assayer().pre_seed(&entries)?;
        guard_pre_seed_result(self.declaration.population.preseed_entries, &result)?;
        guard_clean_health(health_observation(world))?;
        Ok(result)
    }
}

fn synthetic_report(cell_count: usize) -> BatchReport<u128> {
    let mut report = minimal_report();
    for depth in 1..cell_count {
        let depth = u32::try_from(depth).expect("synthetic report depth fits in u32");
        report.cell_reports.push(make_cell_report(
            0,
            depth,
            100,
            2,
            8,
            0.5,
            0.1,
            [0.25; crate::types::SCORING_AXIS_COUNT],
            [0.125; crate::types::SCORING_AXIS_COUNT],
            true,
        ));
    }
    report.contour.cell_count = cell_count;
    report.contour.plateau_count = cell_count;
    report.analysis_set_summary.competitive_size = cell_count;
    report.analysis_set_summary.full_size = cell_count;
    report.analysis_set_summary.investment_set_size = cell_count;
    report.analysis_set_summary.depth_range = (
        0,
        u32::try_from(cell_count.saturating_sub(1)).expect("synthetic report depth fits in u32"),
    );
    report
}

/// Installs and guards a declared synthetic report population.
///
/// The caller registers the axes and Sentinels first; this fixture owns report
/// construction, synchronous structural acknowledgement, slot-width checks,
/// and the clean-health guard.
///
/// # Errors
///
/// Returns a typed report, layout, health, or declaration error with every
/// measured precondition.
pub fn install_synthetic_reports(
    world: &World,
    declaration: SyntheticReportDeclaration,
) -> Result<SyntheticReportObservation, PerformanceFixtureError> {
    if declaration.sentinel_count > SENTINEL_NAMES.len() || declaration.cell_count == 0 || declaration.cell_count > 128 {
        return Err(PerformanceFixtureError::SyntheticReportUnsupported {
            sentinel_count: declaration.sentinel_count,
            cell_count: declaration.cell_count,
        });
    }
    let layout = world.observed_runtime_layout()?;
    let slot_width = layout
        .sentinel_slots_width
        .checked_div(declaration.sentinel_count)
        .unwrap_or_default();
    let mut observation = SyntheticReportObservation {
        sentinel_count: 0,
        slot_width,
        cell_count: declaration.cell_count,
        outcome_axis_width: world.assayer().full_health_report().axes.len(),
        acknowledgements: 0,
        degraded_cells: 0,
    };
    for name in SENTINEL_NAMES.iter().take(declaration.sentinel_count) {
        let ack = world.receive_report(*name, synthetic_report(declaration.cell_count))?;
        observation.sentinel_count += 1;
        observation.acknowledgements += 1;
        observation.degraded_cells += ack.degraded_cells;
        if ack.cells_in_report != declaration.cell_count {
            observation.cell_count = ack.cells_in_report;
        }
    }
    guard_synthetic_reports(declaration, observation)?;
    guard_clean_health(health_observation(world))?;
    Ok(observation)
}

fn base_builder(case: PerformanceCase, instance_id: &str) -> Result<super::WorldBuilder, PerformanceFixtureError> {
    let declaration = case.declaration();
    Ok(World::builder(performance_config(instance_id)?)
        .channel("default", performance_policy(declaration)?)
        .expected_runtime_layout(declaration.dimensions.runtime)
        .seed(0xBEEF_CAFE))
}

fn build_cold_world(case: PerformanceCase, instance_id: &str) -> Result<World, PerformanceFixtureError> {
    let world = base_builder(case, instance_id)?.build()?;
    let population = SeededLabelPopulation::for_case(case)?;
    guard_population(case.declaration().mix, population.summary())?;
    world.runtime_layout()?;
    guard_clean_health(health_observation(&world))?;
    with_fixed_clock_sample(&world, |_| ())?;
    Ok(world)
}

fn build_preseed_world(case: PerformanceCase, instance_id: &str) -> Result<World, PerformanceFixtureError> {
    let world = base_builder(case, instance_id)?.build()?;
    let population = SeededPreSeedPopulation::for_case(case)?;
    drop(population.entries(&world)?);
    world.runtime_layout()?;
    guard_clean_health(health_observation(&world))?;
    with_fixed_clock_sample(&world, |_| ())?;
    Ok(world)
}

fn build_dense_world(case: PerformanceCase, instance_id: &str) -> Result<World, PerformanceFixtureError> {
    let declaration = case.declaration();
    let signal = SignalDeclaration::new(
        "dense-signal",
        SignalShape::Vector {
            len: declaration.dimensions.runtime.signal_width,
            clip: (-1.0, 1.0),
        },
        Persistence::Request,
    );
    let mut world = base_builder(case, instance_id)?.signal_schema(vec![signal]).build()?;
    for name in AXIS_NAMES.iter().take(declaration.population.non_spatial_outcome_axes) {
        world.register_axis(*name, false)?;
    }
    let population = SeededLabelPopulation::for_case(case)?;
    population.publish_warmup(&world)?;
    world.runtime_layout()?;
    guard_clean_health(health_observation(&world))?;
    with_fixed_clock_sample(&world, |_| ())?;
    Ok(world)
}

fn build_reference_reporting(case: PerformanceCase, instance_id: &str) -> Result<World, PerformanceFixtureError> {
    let declaration = case.declaration();
    let mut world = base_builder(case, instance_id)?
        .interaction_templates(reference_templates())
        .build()?;
    for name in AXIS_NAMES.iter().take(declaration.population.spatial_outcome_axes) {
        world.register_axis(*name, true)?;
    }
    for name in AXIS_NAMES
        .iter()
        .skip(declaration.population.spatial_outcome_axes)
        .take(declaration.population.non_spatial_outcome_axes)
    {
        world.register_axis(*name, false)?;
    }
    for name in SENTINEL_NAMES.iter().take(declaration.population.reporting_sentinels) {
        world.register_sentinel(*name)?;
    }
    if let Some(reports) = declaration.reports {
        install_synthetic_reports(&world, reports)?;
    }
    Ok(world)
}

fn complete_reference_identities(
    world: &mut World,
    declaration: &'static PerformanceCaseDeclaration,
) -> Result<(), PerformanceFixtureError> {
    for index in 0..declaration.population.identity_dimensions {
        let name = match index {
            0 => "identity-a",
            1 => "identity-b",
            _ => unreachable!("the case table declares at most two identity dimensions"),
        };
        world.register_identity_with_cells(name, reference_cells("performance-entity-0", declaration.population))?;
    }
    world.runtime_layout()?;
    Ok(())
}

fn build_settled_reference_world(case: PerformanceCase, instance_id: &str) -> Result<World, PerformanceFixtureError> {
    let declaration = case.declaration();
    let mut world = build_reference_reporting(case, instance_id)?;
    SeededLabelPopulation::for_case(case)?.publish_measured(&world)?;
    complete_reference_identities(&mut world, declaration)?;
    world.runtime_layout()?;
    guard_clean_health(health_observation(&world))?;
    if let Some(expected) = declaration.dimensions.health_model_rows {
        guard_health_model_rows(expected, world.assayer().full_health_report().precision.len())?;
    }
    with_fixed_clock_sample(&world, |_| ())?;
    Ok(world)
}

fn observe_reference_removal(world: &mut World) -> Result<ReferenceRemovalObservation, PerformanceFixtureError> {
    let starting_width = world.runtime_layout()?.dimension_map_width;
    let departing_block_width = world.sentinel_departing_block_width(SENTINEL_NAMES[0])?.unwrap_or_default();
    let before = world.assayer().full_health_report();
    world.deregister_sentinel(SENTINEL_NAMES[0])?;
    let channel = world
        .channel("default")
        .expect("performance world declares the default channel");
    let publication_entry = PreSeedSpec::benign(channel, World::entity("removal-health-publication"))
        .ground_truth()
        .build();
    let publication = world.assayer().pre_seed(&[publication_entry])?;
    guard_pre_seed_result(1, &publication)?;
    let after = world.assayer().full_health_report();
    let retained_width = world.observed_runtime_layout()?.dimension_map_width;
    Ok(ReferenceRemovalObservation {
        starting_width,
        departing_block_width,
        trained_labels: world.assayer().health_summary().eligible_labels,
        retained_width,
        marginalisation_events: after.marginalisation.events.saturating_sub(before.marginalisation.events),
        corrections_skipped: after
            .marginalisation
            .corrections_skipped
            .saturating_sub(before.marginalisation.corrections_skipped),
    })
}

/// Builds a trained reference world only after a sacrificial twin proves the
/// real lifecycle removal uses the declared widths and a non-fallback Schur
/// correction.
///
/// # Errors
///
/// Returns a typed construction, population, lifecycle, layout, or removal
/// guard error carrying every measured precondition.
pub fn reference_removal_world(instance_id: &str) -> Result<World, PerformanceFixtureError> {
    let case = PerformanceCase::ReferenceRemoval;
    let declaration = case.declaration();
    let population = SeededLabelPopulation::for_case(case)?;

    let mut validation = build_reference_reporting(case, &format!("{instance_id}-validation"))?;
    population.publish_measured(&validation)?;
    complete_reference_identities(&mut validation, declaration)?;
    let observation = observe_reference_removal(&mut validation)?;
    guard_reference_removal(declaration.dimensions, declaration.population.labels, observation)?;

    let mut world = build_reference_reporting(case, instance_id)?;
    population.publish_measured(&world)?;
    complete_reference_identities(&mut world, declaration)?;
    world.runtime_layout()?;
    guard_clean_health(health_observation(&world))?;
    with_fixed_clock_sample(&world, |_| ())?;
    Ok(world)
}
