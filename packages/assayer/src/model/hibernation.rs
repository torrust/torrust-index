// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! The hibernation archive: what a hibernating deregistration keeps, and on
//! what terms a re-registration may have it back
//! (´alg:registry:hibernation´), (´cav:limitation:hibernation´).
//!
//! Hibernation is a host-chosen variant of deregistration, not a replacement
//! for it. The models are marginalised exactly as the destructive path
//! marginalises them (´alg:registry:sentinel-deregistration´); what the
//! variant adds is a copy of the departing entity's *own* parameter block —
//! its mean sub-vector, the precision and covariance rows and columns that
//! couple its features to each other, and its standardisation entries — kept
//! against its identifier so a later registration of the same identifier can
//! re-extend with the aged block instead of the prior.
//!
//! Nothing about the entity's couplings to anything else is kept, because
//! extension always writes zero off-diagonal blocks and there is nowhere for
//! them to go (´cav:limitation:hibernation´).
//!
//! # Custody
//!
//! The archive is a field of the working copy, so the single steward is its
//! only writer (´dec:concurrency:single-steward´) and it crosses a restart in
//! the whole-state checkpoint alongside the models it describes
//! (´dec:durability:checkpoint-journal´). It is not part of the published
//! snapshot: no reader on the assessment or label path has anything to ask of
//! a block belonging to an entity that is not registered.
//!
//! # Cross-References
//!
//! - (´alg:registry:hibernation´) — the archived payload and the restore's decay
//! - (´cav:limitation:hibernation´) — what hibernation does not preserve
//! - (´rem:registry:axis-hibernation´) — an outcome axis under the same protocol
//! - (´entry:construction:hibernation´) — the archive contract this module carries
//! - (´dec:construction:destructive-deregistration´) — the default this varies from

use indexmap::IndexMap;

use crate::linalg::symmetric::SymmetricMatrix;
use crate::types::{ModelId, OutcomeAxisId, PersistentTimestamp, SentinelId};

// ═══════════════════════════════════════════════════════════════════════════════
// Record generation
// ═══════════════════════════════════════════════════════════════════════════════

/// The archive record's own generation.
///
/// The record travels inside the checkpoint and shares its encoding, but it
/// carries a generation of its own because the two answer different questions:
/// the checkpoint's generation says whether this process can read the file at
/// all, and this one says whether a record inside a file it *can* read
/// describes a block laid out the way this build expects. A record from
/// another generation is refused and the registration extends at the prior,
/// which is the fallback the algorithm already names for a block that can no
/// longer be used (´alg:registry:hibernation´).
///
/// ´const:assayer:hibernation-record-generation´ (´alg:const:version´)
/// ´const:assayer:hibernation-record-generation-version-1´
pub const HIBERNATION_RECORD_VERSION: u32 = 1;

// ═══════════════════════════════════════════════════════════════════════════════
// The archived block
// ═══════════════════════════════════════════════════════════════════════════════

/// One model's archived self-structure block.
///
/// The three pieces the algorithm names, restricted to the entity's own
/// positions: the mean sub-vector, and the square blocks of the precision and
/// the covariance that those positions form among themselves
/// (´alg:registry:hibernation´).
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HibernatedBlock {
    /// The mean sub-vector `μ_S`, in the block's own order.
    pub mean: Vec<f64>,
    /// The precision block `B_SS`.
    pub precision: SymmetricMatrix,
    /// The covariance block `Σ_SS`.
    pub covariance: SymmetricMatrix,
}

impl HibernatedBlock {
    /// The block's width, which is the entity's own feature count.
    #[must_use]
    pub const fn width(&self) -> usize {
        self.mean.len()
    }

    /// Whether the three pieces describe one block of one width.
    #[must_use]
    pub fn is_well_formed(&self) -> bool {
        let k = self.mean.len();
        self.precision.dim() == k && self.covariance.dim() == k
    }

    /// The smallest precision the block holds on its diagonal.
    ///
    /// This is what the replenishment floor is read against when the restore
    /// decides whether the aged block still carries more than the prior would
    /// (´req:gaussian:prior-replenishment-floor´).
    #[must_use]
    pub fn minimum_precision(&self) -> f64 {
        if self.precision.dim() == 0 {
            return f64::INFINITY;
        }
        self.precision.diagonal_min_max().0
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// The archive record
// ═══════════════════════════════════════════════════════════════════════════════

/// What an archived block's positions are, and how many of them there are.
///
/// The whole of the agreement between an archive and a later registration.
/// The entity's own feature count is not a property of the entity alone: a
/// Sentinel's slot widens with every spatial outcome axis, and an axis's block
/// widens with every identity dimension and every Sentinel. A record whose
/// layout differs from the one the registration is creating describes
/// positions that no longer exist in that arrangement, and writing it would
/// put a learned weight at a coordinate that means something else — the
/// specification's own condition that the entity's configuration has changed
/// (´alg:registry:hibernation´).
///
/// The layout carries counts rather than a class vector because the two ends
/// can compute the counts from the same authorities without a rebuild between
/// them: the deregistration reads them off the live map, and the registration
/// computes them to size its own extension.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum HibernationLayout {
    /// A Sentinel's slot: the occupancy indicator and the extraction width,
    /// the latter set by how many spatial outcome axes were registered.
    Sentinel {
        /// The slot's total width, `1 + q`.
        width: usize,
        /// Spatial outcome axes registered when the block was archived.
        spatial_axes: usize,
    },
    /// An outcome axis's own features: a triple in each identity dimension's
    /// block, and where the spatial policy is on, a pair in each Sentinel's
    /// slot (´alg:registry:axis-registration´).
    OutcomeAxis {
        /// The block's total width.
        width: usize,
        /// Identity dimensions registered when the block was archived.
        identity_dimensions: usize,
        /// Sentinels registered when the block was archived.
        sentinels: usize,
        /// Whether the axis carried per-Sentinel outcome-memory features
        /// (´disc:registry:spatial-policy´).
        spatial: bool,
    },
}

impl HibernationLayout {
    /// The number of positions the block covers.
    #[must_use]
    pub const fn width(&self) -> usize {
        match *self {
            Self::Sentinel { width, .. } | Self::OutcomeAxis { width, .. } => width,
        }
    }
}

/// Everything kept for one hibernating entity.
///
/// The record is keyed by the entity's identifier in the archive and never
/// carries the identifier itself, so there is one place a key can be wrong
/// rather than two that can disagree.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HibernationRecord {
    /// The generation this record was written at.
    pub version: u32,
    /// The wall-clock reading at which the block was archived. The expiry is
    /// measured from here (´dec:clock:two-domains´).
    pub archived_at: PersistentTimestamp,
    /// The label sequence the steward had reached when the block was
    /// archived. The labels processed since are the label-indexed half of the
    /// restore's ageing (´alg:registry:hibernation´).
    pub archived_label_seq: u64,
    /// What the archived positions are, and how many.
    pub layout: HibernationLayout,
    /// The archived standardisation means, one per archived position.
    pub feature_means: Vec<f64>,
    /// The archived standardisation variances, one per archived position.
    pub feature_variances: Vec<f64>,
    /// The archived block of every full-dimension model, by model.
    ///
    /// The anchor is absent by construction: it is a fixed-dimension
    /// projection that no lifecycle event extends or marginalises
    /// (´dec:substrate:anchor-invariant´).
    pub blocks: Vec<(ModelId, HibernatedBlock)>,
}

/// Why a restore did not happen.
///
/// Each variant is a refusal rather than a failure: the registration proceeds
/// and extends at the prior, which is the path a registration without an
/// archive takes anyway (´alg:registry:hibernation´).
///
/// Every variant is a property of the record as a whole, which is what makes
/// one answer for the whole record the right shape. The replenishment floor is
/// not among them and could not be: ageing is a property of one model's block
/// under that model's own label-indexed rate, so the same archived block can
/// stand above the floor for a model that forgets slowly and sink below it for
/// one that forgets fast (´req:gaussian:prior-replenishment-floor´), and a
/// single verdict on the record would have to be wrong about one of them. The
/// floor is therefore asked per model where the block is transplanted, in
/// `WorkingCopy::restore_self_structure`, and its answer is counted into
/// [`RestoreReport::at_prior`] rather than returned here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RestoreRefusal {
    /// The record was written at another generation of the record format.
    Generation,
    /// The archive's wall-clock lifetime had run out before the registration
    /// arrived.
    Expired,
    /// The registration creates positions the archived block does not
    /// describe: a different width, or the same width under a different
    /// arrangement of Sentinels, axes, and identity dimensions.
    LayoutChanged,
    /// The record is internally inconsistent: a block whose three pieces do
    /// not agree on one width, or whose width is not the class vector's.
    Malformed,
    /// An archived precision block does not factor, so it is not a matrix the
    /// model may hold and transplanting it would write a negative eigenvalue
    /// into the live posterior (´dec:posterior:cascade-never-fails´).
    NotPositiveDefinite {
        /// The index, within the archived block, of the first pivot the
        /// factorisation could not take. It is a coordinate of the block
        /// rather than of the model the block was going to be written into.
        pivot: usize,
    },
}

impl HibernationRecord {
    /// The archived width, taken from the layout that fixes it.
    #[must_use]
    pub const fn width(&self) -> usize {
        self.layout.width()
    }

    /// Whether the record's own pieces agree on one width.
    #[must_use]
    pub fn is_well_formed(&self) -> bool {
        let k = self.width();
        k > 0
            && self.feature_means.len() == k
            && self.feature_variances.len() == k
            && !self.blocks.is_empty()
            && self.blocks.iter().all(|(_, b)| b.is_well_formed() && b.width() == k)
    }

    /// Hours of wall clock elapsed since the block was archived.
    #[must_use]
    pub fn hours_since_archived(&self, now: &PersistentTimestamp) -> f64 {
        now.hours_since(&self.archived_at)
    }

    /// Labels the steward processed while the entity was away.
    ///
    /// The subtraction saturates: a restore reading a sequence behind the
    /// archived one is a clock that went backwards, and the honest answer is
    /// that no labels are known to have passed rather than a negative count
    /// wrapped into an enormous one.
    #[must_use]
    pub const fn labels_since_archived(&self, label_seq: u64) -> u64 {
        label_seq.saturating_sub(self.archived_label_seq)
    }

    /// The combined ageing factor for one model's block.
    ///
    /// The product of the two clocks the algorithm names: the time-indexed
    /// rate over the elapsed wall-clock interval and the label-indexed rate
    /// over the labels processed meanwhile (´alg:registry:hibernation´).
    /// Both halves flow through the shared decay functions
    /// (´dec:clock:shared-functions´).
    #[must_use]
    pub fn decay(&self, now: &PersistentTimestamp, label_seq: u64, gamma_time: f64, gamma_label: f64) -> f64 {
        let elapsed = crate::numerics::decay_factor(gamma_time, self.hours_since_archived(now));
        let labels = crate::numerics::label_decay_factor(gamma_label, self.labels_since_archived(label_seq));
        elapsed * labels
    }

    /// Whether this record may be restored onto the positions a registration
    /// is creating, and if not, why not.
    ///
    /// Admission is about the record as a whole: its generation, its internal
    /// consistency, its age, whether the arrangement it was archived under is
    /// the arrangement being created, and whether every block it carries is a
    /// matrix the model may hold. Whether any individual model's block has
    /// aged past the replenishment floor is decided per model afterwards,
    /// because the models forget at different rates and one that has forgotten
    /// everything says nothing about another that has not.
    ///
    /// Definiteness is decided here rather than there, and the difference is
    /// what the two questions are about. Forgetting is a property of one model
    /// against elapsed time, so it is asked per model at the moment of
    /// transplant. An indefinite block is a property of the *record* — the
    /// archive crossed the persistence boundary and came back carrying a
    /// matrix no arithmetic in this package could have produced — and the
    /// per-model ageing cannot change the answer, because scaling by a
    /// positive factor multiplies every eigenvalue by it and moves none
    /// across zero. A record with one corrupt block is a corrupt record, and
    /// admitting its other blocks would be transplanting parameters from an
    /// artefact already known to be damaged.
    ///
    /// # Errors
    ///
    /// Returns the refusal that applies, which the caller reports and then
    /// extends at the prior.
    pub fn admit(&self, arriving: HibernationLayout, elapsed_hours: f64, expiry_hours: f64) -> Result<(), RestoreRefusal> {
        if self.version != HIBERNATION_RECORD_VERSION {
            return Err(RestoreRefusal::Generation);
        }
        if !self.is_well_formed() {
            return Err(RestoreRefusal::Malformed);
        }
        if elapsed_hours > expiry_hours {
            return Err(RestoreRefusal::Expired);
        }
        if self.layout != arriving {
            return Err(RestoreRefusal::LayoutChanged);
        }
        for (_, block) in &self.blocks {
            carries_a_definite_precision(block)?;
        }
        Ok(())
    }
}

impl HibernationRecord {
    /// The archived block for one model, where the record carries one.
    ///
    /// A model the record does not name gets the prior: an outcome axis
    /// registered after the entity hibernated never held a block for it, and
    /// there is nothing to age.
    #[must_use]
    pub fn block(&self, model: &ModelId) -> Option<&HibernatedBlock> {
        self.blocks.iter().find(|(id, _)| id == model).map(|(_, block)| block)
    }
}

/// What a restore did, model by model.
///
/// Reported rather than inferred: a host reading a returning Sentinel's
/// weights needs to know whether they are the archived ones aged or the
/// prior, and the two are indistinguishable from the parameters alone
/// (´dec:health:reports-never-gates´).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RestoreReport {
    /// Models whose block was written from the archive.
    pub restored: usize,
    /// Models that took the extension's prior instead, because the record
    /// held no block for them or the block had aged past the floor.
    pub at_prior: usize,
}

impl RestoreReport {
    /// Whether any model took the archived block.
    #[must_use]
    pub const fn any_restored(&self) -> bool {
        self.restored > 0
    }
}

/// Whether an aged block still carries more than the prior would.
///
/// The test is the replenishment floor read forward: the live path applies the
/// per-label rate and then holds every precision diagonal at or above
/// `lambda_floor` (´req:gaussian:prior-replenishment-floor´), so a block whose
/// smallest diagonal would land below the floor after ageing has been forgotten
/// in the only sense the model has. Refusing it is also what keeps the
/// covariance bounded: the covariance block is scaled by the reciprocal of the
/// same factor, and a factor small enough to sink the precision below the floor
/// is a factor large enough to make the covariance unusable when inverted.
#[must_use]
pub fn survives_floor(block: &HibernatedBlock, decay: f64, lambda_floor: f64) -> bool {
    if decay <= 0.0 || !decay.is_finite() {
        return false;
    }
    let minimum = block.minimum_precision();
    minimum.is_finite() && decay * minimum >= lambda_floor
}

/// Whether an archived block's precision is a matrix the model may hold.
///
/// This stands beside [`survives_floor`] and answers a different question,
/// which is why it is a second function rather than a widening of that one.
/// The floor test asks whether an entity has been forgotten; its reading is
/// the block's minimum diagonal because that is the quantity the replenishment
/// floor is stated in (´req:gaussian:prior-replenishment-floor´). This test
/// asks whether the block is positive definite, and the diagonal cannot answer
/// it: a strictly positive minimum diagonal is necessary for definiteness and
/// is not sufficient for it (´req:gaussian:positive-definiteness´). A block of
/// ones on the diagonal and twos off it passes the floor test at any floor
/// below one and carries an eigenvalue of minus one.
///
/// It matters here because the transplant is exact. The block is written over
/// positions whose couplings the extension left at zero, so the matrix is
/// block diagonal there and its spectrum is the union of the block's with the
/// rest's: an indefinite archived block puts its negative eigenvalue into the
/// live posterior directly, and the block type derives the serialisation
/// traits, so the archive is a persisted artefact on the same footing as the
/// checkpoint.
///
/// The verdict is a plain Cholesky attempt, which is the cheapest available
/// witness of definiteness and the same instrument the checkpoint restore
/// takes. It reads the precision alone: the invariant the model promises is
/// carried by `B`, and a covariance block inconsistent with a definite `B` is
/// caught where inconsistency is measurable, at the next rebuild's adoption
/// test, rather than guessed at here.
///
/// # Errors
///
/// Returns [`RestoreRefusal::NotPositiveDefinite`] carrying the pivot the
/// factorisation stopped at. Like every variant beside it, it is a refusal
/// rather than a failure: the registration extends at the prior, which is a
/// definite block by construction (´alg:registry:hibernation´).
pub fn carries_a_definite_precision(block: &HibernatedBlock) -> Result<(), RestoreRefusal> {
    // A zero-width block has no matrix to factor; the verdict is vacuous
    // rather than favourable.
    if block.precision.dim() == 0 {
        return Ok(());
    }
    crate::linalg::bridge::cholesky(&block.precision).map_err(|error| {
        let crate::linalg::bridge::LltError::NonPositivePivot { index } = error;
        RestoreRefusal::NotPositiveDefinite { pivot: index }
    })?;
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════════
// The archive
// ═══════════════════════════════════════════════════════════════════════════════

/// The Assayer's hibernation archive, keyed by entity identifier.
///
/// Sentinels and outcome axes are kept apart because their identifier spaces
/// are: two registries may issue the same number, and one map would let a
/// Sentinel's block be handed to an axis that happens to share its
/// identifier's value.
///
/// A record leaves the archive when the identifier it is keyed by is
/// registered again, whether or not the record was usable: a re-registration
/// ends the archive's usability, so the record is taken rather than read.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HibernationArchive {
    /// Archived Sentinel blocks, in archival order.
    sentinels: IndexMap<SentinelId, HibernationRecord>,
    /// Archived outcome-axis blocks, in archival order.
    axes: IndexMap<OutcomeAxisId, HibernationRecord>,
}

impl HibernationArchive {
    /// Stores a Sentinel's record, replacing any record already held for that
    /// identifier.
    pub fn store_sentinel(&mut self, id: SentinelId, record: HibernationRecord) {
        self.sentinels.insert(id, record);
    }

    /// Stores an outcome axis's record, replacing any record already held for
    /// that identifier.
    pub fn store_axis(&mut self, id: OutcomeAxisId, record: HibernationRecord) {
        self.axes.insert(id, record);
    }

    /// Takes a Sentinel's record out of the archive.
    ///
    /// Taking rather than reading is the restore window: the record is gone
    /// once its identifier has been registered again, so a second
    /// registration of the same identifier finds nothing and extends at the
    /// prior (´alg:registry:hibernation´).
    pub fn take_sentinel(&mut self, id: SentinelId) -> Option<HibernationRecord> {
        self.sentinels.shift_remove(&id)
    }

    /// Takes an outcome axis's record out of the archive, on the same terms
    /// as a Sentinel's.
    pub fn take_axis(&mut self, id: OutcomeAxisId) -> Option<HibernationRecord> {
        self.axes.shift_remove(&id)
    }

    /// Whether a Sentinel's record is currently held.
    #[must_use]
    pub fn holds_sentinel(&self, id: SentinelId) -> bool {
        self.sentinels.contains_key(&id)
    }

    /// Whether an outcome axis's record is currently held.
    #[must_use]
    pub fn holds_axis(&self, id: OutcomeAxisId) -> bool {
        self.axes.contains_key(&id)
    }

    /// Reads a Sentinel's record without taking it, for reporting.
    #[must_use]
    pub fn sentinel(&self, id: SentinelId) -> Option<&HibernationRecord> {
        self.sentinels.get(&id)
    }

    /// Reads an outcome axis's record without taking it, for reporting.
    #[must_use]
    pub fn axis(&self, id: OutcomeAxisId) -> Option<&HibernationRecord> {
        self.axes.get(&id)
    }

    /// How many records the archive holds, across both registries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.sentinels.len() + self.axes.len()
    }

    /// Whether the archive holds nothing.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.sentinels.is_empty() && self.axes.is_empty()
    }

    /// Drops every record whose wall-clock lifetime has run out.
    ///
    /// Expiry is enforced without a clock of its own: this is called from the
    /// steward's own lifecycle pass, which is the only place the archive is
    /// written, so an expired record is reclaimed at the next lifecycle event
    /// rather than by a thread that exists to wait for it
    /// (´dec:concurrency:single-steward´). A record that expires and is never
    /// swept is still never restored, because the restore reads the same
    /// clock; the sweep is what stops the storage from outliving the
    /// permission to use it.
    ///
    /// Returns how many records were dropped.
    pub fn expire(&mut self, now: &PersistentTimestamp, expiry_hours: f64) -> usize {
        let before = self.len();
        self.sentinels
            .retain(|_, record| record.hours_since_archived(now) <= expiry_hours);
        self.axes.retain(|_, record| record.hours_since_archived(now) <= expiry_hours);
        before - self.len()
    }
}
