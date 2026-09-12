// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Public lifecycle methods on `Assayer`.
//!
//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`mudlark_config_uses_registered_identity_thresholds`] | identity | Every graph threshold in the Mudlark configuration is copied from the public identity budget, while the shipped budget constructor reproduces the former derived values for the registration's competitive cutoff. |
//!
//! Eight `&self` calls over the six operations of registering and
//! deregistering Sentinels, outcome axes, and identity dimensions: the
//! Sentinel and axis removals each come in two dispositions, destructive and
//! hibernating (´alg:registry:hibernation´). Each call implements a
//! two-phase visibility model:
//!
//! - **Phase 1 (sync)** — Immediate structural changes visible to
//!   concurrent `assess()` calls (e.g. `DashMap` insert, slot creation).
//! - **Phase 2 (async)** — Model extension/marginalisation on the
//!   model-owner thread; visible at next snapshot publication.
//!
//! # Cross-References
//!
//! - Lifecycle is six operations (´dec:construction:six-methods´)
//! - Infrastructure becomes visible synchronously on return, the model
//!   asynchronously at the next publication (´dec:construction:two-phase-visibility´)

use std::sync::Arc;
use std::time::Duration;

use crossbeam_channel::TrySendError;
use dashmap::mapref::entry::Entry;
use tracing::debug;

use crate::Assayer;
use crate::error::LifecycleError;
use crate::identity::{IdentityDimension, IdentityDimensionInfra, MaintenanceCommand};
use crate::owner::commands::{
    IdentityDimensionRegistration, LifecycleEvent, LifecycleSubmission, ModelOwnerCommand, OutcomeAxisRegistration,
    SentinelRegistration,
};
use crate::report::SentinelSlot;
use crate::types::{DimensionId, OutcomeAxisId, SentinelId};

// ═══════════════════════════════════════════════════════════════════════════════
// Lifecycle Methods
// ═══════════════════════════════════════════════════════════════════════════════

impl Assayer {
    /// The bound the identity registration acknowledgement wait is held to.
    ///
    /// The host's configured figure, and the only place this wait's bound is
    /// formed — the wait carried a constant of its own until the surface was
    /// connected to it (´rem:construction:registration-timeout´),
    /// (´cav:construction:registration-timeout-shadowed´).
    pub(crate) const fn identity_registration_timeout(&self) -> Duration {
        Duration::from_secs(self.config.infrastructure.identity_registration_timeout_secs)
    }

    /// Registers a new Sentinel.
    ///
    /// **Phase 1 (sync):** Creates a `SentinelSlot` in the `DashMap`,
    /// immediately visible to `assess()` (zero features) and
    /// `receive_sentinel_report()` (reports accepted), with its
    /// bootstrap accumulator opened — step 1 of
    /// (´alg:standardisation:sentinel-bootstrap´).
    ///
    /// **Phase 2 (async):** The model-owner thread extends all models,
    /// rebuilds the `DimensionMap`, and extends standardisation.
    ///
    /// # Errors
    ///
    /// - [`LifecycleError::EmptyName`] — registration name is empty
    /// - [`LifecycleError::DuplicateId`] — Sentinel ID already registered
    /// - [`LifecycleError::ModelOwnerShutdown`] — model-owner thread exited
    /// - [`LifecycleError::CommandChannelFull`] — command channel at capacity
    ///
    /// # Cross-References
    ///
    /// - Sentinel registration, whose visibility is two-phase (´dec:construction:two-phase-visibility´)
    pub fn register_sentinel(&self, reg: SentinelRegistration) -> Result<(), LifecycleError> {
        if reg.name.is_empty() {
            return Err(LifecycleError::EmptyName { entity_type: "Sentinel" });
        }

        // Phase 1: Atomic check-and-insert via DashMap entry API
        match self.sentinel_slots.entry(reg.id) {
            Entry::Occupied(_) => {
                return Err(LifecycleError::DuplicateId {
                    entity_type: "Sentinel",
                    id: format!("{:?}", reg.id),
                });
            }
            Entry::Vacant(v) => {
                let slot = SentinelSlot::new();
                // Dimension 0: the accumulator re-sizes itself to the
                // slot's actual width on first observation, step 2's
                // reset rule (´alg:standardisation:sentinel-bootstrap´).
                slot.start_bootstrap(crate::feature::bootstrap::BootstrapAccumulator::new(
                    0,
                    self.config.standardisation.n_boot as usize,
                ));
                v.insert(slot);
            }
        }

        self.outcome_ledger.create_sentinel(reg.id);

        debug!(sentinel_id = ?reg.id, name = %reg.name, "sentinel registered (Phase 1)");

        // Phase 2: Send lifecycle event to model owner
        let sentinel_id = reg.id;
        if let Err(e) = self.send_lifecycle(LifecycleEvent::RegisterSentinel(reg)) {
            self.sentinel_slots.remove(&sentinel_id);
            drop(self.outcome_ledger.remove_sentinel(sentinel_id));
            return Err(e);
        }

        Ok(())
    }

    /// Deregisters a Sentinel.
    ///
    /// **Phase 1 (sync):** Removes the `SentinelSlot` from the `DashMap`.
    /// Concurrent `assess()` calls will no longer see this Sentinel.
    /// Reports for this Sentinel are rejected with `UnknownSentinel`.
    ///
    /// **Phase 2 (async):** The model-owner thread marginalises models,
    /// rebuilds the `DimensionMap`, and compacts standardisation.
    ///
    /// # Errors
    ///
    /// - [`LifecycleError::NotFound`] — Sentinel ID not registered
    /// - [`LifecycleError::ModelOwnerShutdown`] — model-owner thread exited
    /// - [`LifecycleError::CommandChannelFull`] — command channel at capacity
    ///
    /// # Cross-References
    ///
    /// - Sentinel deregistration, which is destructive (´dec:construction:destructive-deregistration´)
    pub fn deregister_sentinel(&self, id: SentinelId) -> Result<(), LifecycleError> {
        // Phase 1: Remove from DashMap (capture the slot for revert)
        let (_removed_id, removed_slot) = self.sentinel_slots.remove(&id).ok_or_else(|| LifecycleError::NotFound {
            entity_type: "Sentinel",
            id: format!("{id:?}"),
        })?;

        debug!(sentinel_id = ?id, "sentinel deregistered (Phase 1)");

        // Phase 2: Send lifecycle event to model owner
        if let Err(e) = self.send_lifecycle(LifecycleEvent::DeregisterSentinel(id)) {
            // Revert: re-insert the slot so the system stays consistent.
            self.sentinel_slots.insert(id, removed_slot);
            return Err(e);
        }

        Ok(())
    }

    /// Deregisters a Sentinel, keeping its own parameter block against its
    /// identifier.
    ///
    /// Identical to [`deregister_sentinel`](Self::deregister_sentinel) in
    /// everything the Sentinel's departure does to the models: the surviving
    /// full-dimension models are marginalised by the same Schur complement,
    /// the standardisation entries are compacted, and the Ledger is released.
    /// What this variant adds is a copy of the Sentinel's own slot block —
    /// its mean sub-vector and the precision and covariance rows and columns
    /// its features form among themselves — stored against its identifier
    /// inside the Assayer (´alg:registry:hibernation´).
    ///
    /// Registering the same identifier again before the archive expires
    /// re-extends the models with that block, aged by the elapsed wall-clock
    /// interval and by the labels processed meanwhile, instead of with the
    /// prior. Registering it after the archive expires, or under a different
    /// arrangement of spatial outcome axes, extends at the prior and says so
    /// in the log. The Sentinel's correlations with anything else are not
    /// preserved and are relearned from zero
    /// (´cav:limitation:hibernation´).
    ///
    /// A re-registration ends the archive's usability whether or not it used
    /// the block: the record is taken from the archive by the registration,
    /// not read from it. Hibernating the same identifier again is what puts a
    /// new record there.
    ///
    /// # Errors
    ///
    /// - [`LifecycleError::NotFound`] — Sentinel ID not registered
    /// - [`LifecycleError::ModelOwnerShutdown`] — model-owner thread exited
    /// - [`LifecycleError::CommandChannelFull`] — command channel at capacity
    ///
    /// # Cross-References
    ///
    /// - Hibernation as an optional lifecycle state (´alg:registry:hibernation´)
    /// - What hibernation does not preserve (´cav:limitation:hibernation´)
    /// - The destructive default this varies from (´dec:construction:destructive-deregistration´)
    pub fn hibernate_sentinel(&self, id: SentinelId) -> Result<(), LifecycleError> {
        // Phase 1: Remove from DashMap (capture the slot for revert)
        let (_removed_id, removed_slot) = self.sentinel_slots.remove(&id).ok_or_else(|| LifecycleError::NotFound {
            entity_type: "Sentinel",
            id: format!("{id:?}"),
        })?;

        debug!(sentinel_id = ?id, "sentinel hibernated (Phase 1)");

        // Phase 2: Send lifecycle event to model owner
        if let Err(e) = self.send_lifecycle(LifecycleEvent::HibernateSentinel(id)) {
            // Revert: re-insert the slot so the system stays consistent.
            self.sentinel_slots.insert(id, removed_slot);
            return Err(e);
        }

        Ok(())
    }

    /// Registers a new outcome axis.
    ///
    /// **Phase 1 (sync):** Atomic check-and-insert via the
    /// `registered_axes` guard.  This prevents concurrent callers from
    /// both passing the duplicate check before the model owner
    /// publishes.
    ///
    /// **Phase 2 (async):** The model-owner thread extends models and
    /// Ledger axis fields, creates an axis model, and rebuilds the
    /// `DimensionMap`. If `spatial_features: Enabled`, each existing
    /// Sentinel slot grows by 2 features (compressed + raw axis EWMA)
    /// and all `LedgerEntry` values gain per-axis EWMA fields.
    ///
    /// # Panics
    ///
    /// Panics if `registered_axes` lock is poisoned.
    ///
    /// # Errors
    ///
    /// - [`LifecycleError::EmptyName`] — registration name is empty
    /// - [`LifecycleError::DuplicateId`] — axis ID already registered
    /// - [`LifecycleError::ModelOwnerShutdown`] — model-owner thread exited
    /// - [`LifecycleError::CommandChannelFull`] — command channel at capacity
    ///
    /// # Cross-References
    ///
    /// - Outcome axis registration, whose visibility is two-phase (´dec:construction:two-phase-visibility´)
    pub fn register_outcome_axis(&self, reg: OutcomeAxisRegistration) -> Result<(), LifecycleError> {
        if reg.name.is_empty() {
            return Err(LifecycleError::EmptyName {
                entity_type: "OutcomeAxis",
            });
        }

        // Phase 1: Atomic check-and-insert under the Mutex.
        {
            let mut guard = self.registered_axes.lock().expect("registered_axes lock poisoned");
            if !guard.insert(reg.id) {
                return Err(LifecycleError::DuplicateId {
                    entity_type: "OutcomeAxis",
                    id: format!("{:?}", reg.id),
                });
            }
        }

        debug!(axis_id = ?reg.id, name = %reg.name, "outcome axis registered (Phase 1)");

        // Phase 2: Send lifecycle event to model owner
        let axis_id = reg.id;
        if let Err(e) = self.send_lifecycle(LifecycleEvent::RegisterOutcomeAxis(reg)) {
            // Revert: remove from registered set on send failure.
            self.registered_axes
                .lock()
                .expect("registered_axes lock poisoned")
                .remove(&axis_id);
            return Err(e);
        }

        Ok(())
    }

    /// Deregisters an outcome axis.
    ///
    /// **Phase 1 (sync):** Atomic remove from the `registered_axes`
    /// guard.
    ///
    /// **Phase 2 (async):** The model-owner thread marginalises models
    /// and destroys the axis model.
    ///
    /// # Panics
    ///
    /// Panics if `registered_axes` lock is poisoned.
    ///
    /// # Errors
    ///
    /// - [`LifecycleError::NotFound`] — axis ID not registered
    /// - [`LifecycleError::ModelOwnerShutdown`] — model-owner thread exited
    /// - [`LifecycleError::CommandChannelFull`] — command channel at capacity
    ///
    /// # Cross-References
    ///
    /// - Outcome axis deregistration, which is destructive (´dec:construction:destructive-deregistration´)
    pub fn deregister_outcome_axis(&self, id: OutcomeAxisId) -> Result<(), LifecycleError> {
        // Phase 1: Atomic remove — not-found if absent.
        {
            let mut guard = self.registered_axes.lock().expect("registered_axes lock poisoned");
            if !guard.remove(&id) {
                return Err(LifecycleError::NotFound {
                    entity_type: "OutcomeAxis",
                    id: format!("{id:?}"),
                });
            }
        }

        debug!(axis_id = ?id, "outcome axis deregistered (Phase 1)");

        // Phase 2: Send lifecycle event to model owner
        if let Err(e) = self.send_lifecycle(LifecycleEvent::DeregisterOutcomeAxis(id)) {
            // Revert: re-insert on send failure.
            self.registered_axes.lock().expect("registered_axes lock poisoned").insert(id);
            return Err(e);
        }

        Ok(())
    }

    /// Deregisters an outcome axis, keeping its own parameter block against
    /// its identifier.
    ///
    /// An axis hibernates under the protocol Sentinels hibernate under, with
    /// the same preservation scope (´rem:registry:axis-hibernation´): its own
    /// features — the triple it holds in each identity dimension's block, and
    /// where the spatial policy was on, the outcome-memory pair it holds in
    /// each Sentinel's slot — are archived from the models that survive its
    /// departure, and a later registration of the same identifier re-extends
    /// those positions with the aged block. Its prediction model is destroyed
    /// as it is under the destructive path, and a re-registration creates a
    /// fresh one at the prior: nothing about the axis's own beliefs over the
    /// whole feature space is kept, only the block its features occupy in
    /// everything else.
    ///
    /// The archive is refused, and the registration extends at the prior,
    /// where the arrangement has changed — a different identity-dimension
    /// count, a different Sentinel count, or a different spatial policy — for
    /// the reason the algorithm gives: the block would land on positions that
    /// no longer mean what they meant (´alg:registry:hibernation´).
    ///
    /// # Panics
    ///
    /// Panics if `registered_axes` lock is poisoned.
    ///
    /// # Errors
    ///
    /// - [`LifecycleError::NotFound`] — axis ID not registered
    /// - [`LifecycleError::ModelOwnerShutdown`] — model-owner thread exited
    /// - [`LifecycleError::CommandChannelFull`] — command channel at capacity
    ///
    /// # Cross-References
    ///
    /// - Axis hibernation, by reference to Sentinel hibernation (´rem:registry:axis-hibernation´)
    /// - The destructive default this varies from (´dec:construction:destructive-deregistration´)
    pub fn hibernate_outcome_axis(&self, id: OutcomeAxisId) -> Result<(), LifecycleError> {
        // Phase 1: Atomic remove — not-found if absent.
        {
            let mut guard = self.registered_axes.lock().expect("registered_axes lock poisoned");
            if !guard.remove(&id) {
                return Err(LifecycleError::NotFound {
                    entity_type: "OutcomeAxis",
                    id: format!("{id:?}"),
                });
            }
        }

        debug!(axis_id = ?id, "outcome axis hibernated (Phase 1)");

        // Phase 2: Send lifecycle event to model owner
        if let Err(e) = self.send_lifecycle(LifecycleEvent::HibernateOutcomeAxis(id)) {
            // Revert: re-insert on send failure.
            self.registered_axes.lock().expect("registered_axes lock poisoned").insert(id);
            return Err(e);
        }

        Ok(())
    }

    /// Registers a new identity dimension.
    ///
    /// **Phase 1 (sync, blocking):** Creates the observation channel,
    /// `IdentityDimensionInfra`, and sends a `CreateDimension` command
    /// to the identity maintenance thread. **Blocks** for up to
    /// `infrastructure.identity_registration_timeout_secs` waiting for
    /// acknowledgement.
    ///
    /// **Phase 2 (async):** The model-owner thread extends models.
    ///
    /// # Panics
    ///
    /// Panics if the identity dimensions lock is poisoned.
    ///
    /// # Errors
    ///
    /// - [`LifecycleError::EmptyName`] — registration name is empty
    /// - [`LifecycleError::InvalidDomainBits`] — `domain_bits` is outside `1..=128`
    /// - [`LifecycleError::DuplicateId`] — dimension ID already registered
    /// - [`LifecycleError::IdentityMaintenanceShutdown`] — maintenance thread
    ///   exited or timed out
    /// - [`LifecycleError::ModelOwnerShutdown`] — model-owner thread exited
    /// - [`LifecycleError::CommandChannelFull`] — command channel at capacity
    ///
    /// # Cross-References
    ///
    /// - Identity dimension registration, whose visibility is two-phase (´dec:construction:two-phase-visibility´)
    /// - A dedicated owner holds every identity graph (´dec:memory:graph-owner´)
    #[allow(clippy::needless_pass_by_value)] // Justified: registration structs are single-use; by-value is the public API convention
    pub fn register_identity_dimension(&self, reg: IdentityDimensionRegistration) -> Result<(), LifecycleError> {
        if reg.name.is_empty() {
            return Err(LifecycleError::EmptyName {
                entity_type: "IdentityDimension",
            });
        }

        if !(1..=128).contains(&reg.domain_bits) {
            return Err(LifecycleError::InvalidDomainBits {
                id: reg.id,
                domain_bits: reg.domain_bits,
            });
        }

        // Phase 1: Duplicate check.
        {
            let guard = self.identity_dimensions.read().expect("identity_dimensions lock poisoned");
            if guard.contains_key(&reg.id) {
                return Err(LifecycleError::DuplicateId {
                    entity_type: "IdentityDimension",
                    id: format!("{:?}", reg.id),
                });
            }
        }

        let identity_cmd_tx = self
            .identity_command_tx
            .as_ref()
            .ok_or(LifecycleError::IdentityMaintenanceShutdown)?;

        // Create observation channel
        let capacity = reg.budget.observation_capacity;
        let (obs_tx, obs_rx) = crossbeam_channel::bounded(capacity);

        // Create infrastructure
        let dimension = IdentityDimension::new(
            reg.id,
            &reg.name,
            &reg.description,
            &reg.coordinate_semantics,
            reg.domain_bits,
            reg.depth_cutoff,
            reg.encode,
        );
        let infra = Arc::new(IdentityDimensionInfra::new(dimension, obs_tx));

        // Build Mudlark config from budget
        let mudlark_config = mudlark_config_from_budget(&reg.budget);

        // Create ack channel
        let (ack_tx, ack_rx) = crossbeam_channel::bounded(1);

        // Send CreateDimension to maintenance thread
        if let Err(_e) = identity_cmd_tx.try_send(MaintenanceCommand::CreateDimension {
            config: mudlark_config,
            dimension_id: reg.id,
            depth_cutoff: reg.depth_cutoff,
            spatial_decay_rate: reg.budget.spatial_decay_rate,
            observation_rx: obs_rx,
            infra: Arc::clone(&infra),
            ack: ack_tx,
        }) {
            return Err(LifecycleError::IdentityMaintenanceShutdown);
        }

        // Block for ack, bounded by the host's configured figure.
        if ack_rx.recv_timeout(self.identity_registration_timeout()).is_err() {
            return Err(LifecycleError::IdentityMaintenanceShutdown);
        }

        debug!(dimension_id = ?reg.id, name = %reg.name, "identity dimension registered (Phase 1)");

        // Register infrastructure (double-check under write lock for
        // safety against the narrow duplicate-ID TOCTOU window).
        {
            let mut guard = self.identity_dimensions.write().expect("identity_dimensions lock poisoned");
            if guard.contains_key(&reg.id) {
                // Race: another thread registered this dimension while
                // we were blocking. The orphaned maintenance-thread
                // dimension is harmless (no observations will flow).
                return Err(LifecycleError::DuplicateId {
                    entity_type: "IdentityDimension",
                    id: format!("{:?}", reg.id),
                });
            }
            guard.insert(reg.id, infra);
        }

        // Phase 2: Send lifecycle event to model owner.
        // Only forward the field the model owner needs (id).
        // The full IdentityDimensionRegistration (including `encode`) was
        // consumed above to build the IdentityDimensionInfra.
        if let Err(e) = self.send_lifecycle(LifecycleEvent::RegisterIdentityDimension { id: reg.id }) {
            // Revert: remove infra.
            self.identity_dimensions
                .write()
                .expect("identity_dimensions lock poisoned")
                .remove(&reg.id);
            return Err(e);
        }

        Ok(())
    }

    /// Deregisters an identity dimension.
    ///
    /// **Phase 1 (sync, blocking):** Sends `DeregisterIdentityDimension`
    /// to the model-owner thread and **waits** for it to marginalise the
    /// dimension's features and publish a new snapshot.  This guarantees
    /// the model no longer references the dimension before the
    /// infrastructure is removed.
    ///
    /// **Phase 1b (sync):** Removes the infrastructure from the
    /// `identity_dimensions` map, making it invisible to `assess()`.
    ///
    /// **Phase 3 (fire-and-forget):** Sends `DestroyDimension` to the
    /// maintenance thread so it can release its per-dimension state.
    ///
    /// # Panics
    ///
    /// Panics if the identity dimensions lock is poisoned.
    ///
    /// # Errors
    ///
    /// - [`LifecycleError::NotFound`] — dimension ID not registered
    /// - [`LifecycleError::ModelOwnerShutdown`] — model-owner thread exited
    /// - [`LifecycleError::CommandChannelFull`] — command channel at capacity
    ///
    /// # Cross-References
    ///
    /// - Identity dimension deregistration, which blocks (´dec:construction:blocking-deregistration´)
    pub fn deregister_identity_dimension(&self, id: DimensionId) -> Result<(), LifecycleError> {
        // Phase 1: Not-found check.
        //
        // Note: there is a narrow TOCTOU window between this read-lock
        // check and the write-lock removal in Phase 2b.  A concurrent
        // deregister of the same dimension could slip through, but the
        // result is benign: the model owner processes two no-op
        // marginalisations and the second `remove` returns `None`.
        {
            let guard = self.identity_dimensions.read().expect("identity_dimensions lock poisoned");
            if !guard.contains_key(&id) {
                return Err(LifecycleError::NotFound {
                    entity_type: "IdentityDimension",
                    id: format!("{id:?}"),
                });
            }
        }

        debug!(dimension_id = ?id, "identity dimension deregistered (Phase 1)");

        // Phase 2: Model owner marginalises (synchronous — wait for
        // completion so that the model has stopped referencing the
        // dimension before we destroy the infrastructure).
        self.send_lifecycle_sync(LifecycleEvent::DeregisterIdentityDimension(id))?;

        // Phase 2b: Remove infrastructure — assessment readers will stop
        // seeing it.  Safe because the model owner has already
        // marginalised and published a new snapshot without this
        // dimension's features.
        {
            let mut guard = self.identity_dimensions.write().expect("identity_dimensions lock poisoned");
            guard.remove(&id);
        }

        // Phase 3: Send DestroyDimension to the maintenance thread
        // (non-blocking, fire-and-forget). The maintenance thread will
        // release its per-dimension competitive-set state. If the
        // channel is disconnected/full that's benign — the maintenance
        // thread will clean up on shutdown.
        if let Some(identity_cmd_tx) = self.identity_command_tx.as_ref() {
            drop(identity_cmd_tx.try_send(MaintenanceCommand::DestroyDimension { dimension_id: id }));
        }

        Ok(())
    }

    // ─── Internal Helpers ────────────────────────────────────────────────

    /// Sends a single lifecycle event to the model owner via the command
    /// channel.
    ///
    /// Wraps the event in a `LifecycleSubmission` with no completion
    /// channel (fire-and-forget for Phase 2).
    fn send_lifecycle(&self, event: LifecycleEvent) -> Result<(), LifecycleError> {
        let submission = LifecycleSubmission {
            events: vec![event],
            completion: None,
        };

        self.command_tx
            .try_send(ModelOwnerCommand::Lifecycle(submission))
            .map_err(|e| match e {
                TrySendError::Full(_) => LifecycleError::CommandChannelFull,
                TrySendError::Disconnected(_) => LifecycleError::ModelOwnerShutdown,
            })
    }

    /// Sends a single lifecycle event to the model owner and **waits**
    /// for the model-owner thread to process it and publish a new
    /// snapshot.
    ///
    /// Used by `deregister_identity_dimension` to guarantee that the
    /// model has marginalised the dimension's features before the
    /// infrastructure is removed from `identity_dimensions`.
    fn send_lifecycle_sync(&self, event: LifecycleEvent) -> Result<(), LifecycleError> {
        let (completion_tx, completion_rx) = crossbeam_channel::bounded(1);

        let submission = LifecycleSubmission {
            events: vec![event],
            completion: Some(completion_tx),
        };

        self.command_tx
            .try_send(ModelOwnerCommand::Lifecycle(submission))
            .map_err(|e| match e {
                TrySendError::Full(_) => LifecycleError::CommandChannelFull,
                TrySendError::Disconnected(_) => LifecycleError::ModelOwnerShutdown,
            })?;

        // Block until the model owner has processed the event.
        // This ensures the model snapshot no longer references the
        // dimension before infrastructure is removed.
        //
        // We race the per-submission completion channel against the
        // owner-thread liveness channel (`owner_alive_rx`, see
        // `Assayer::build`). If the owner thread panics mid-submission
        // — for example tripping a `debug_assert_eq!` — the completion
        // sender *should* be dropped during unwind, but in practice the
        // raw `recv()` has been observed to hang. Selecting on the
        // liveness receiver gives us a second, independent signal that
        // the owner is gone: its sole `Sender` lives on the owner's
        // stack, so it is dropped unconditionally on thread exit.
        let mut sel = crossbeam_channel::Select::new();
        let completion_idx = sel.recv(&completion_rx);
        let alive_idx = sel.recv(&self.owner_alive_rx);

        let oper = sel.select();
        match oper.index() {
            i if i == completion_idx => {
                // Treat disconnect as shutdown; discard the inner
                // per-event result: a compound batch continues past
                // non-fatal failures (´dec:construction:compound-batch´).
                let _lifecycle_result = oper.recv(&completion_rx).map_err(|_| LifecycleError::ModelOwnerShutdown)?;
                Ok(())
            }
            i if i == alive_idx => {
                // `owner_alive_rx` will *only* become ready via a
                // `Disconnected` error (we never send on it), but
                // we must consume the operation to satisfy `Select`.
                let _ = oper.recv(&self.owner_alive_rx);
                Err(LifecycleError::ModelOwnerShutdown)
            }
            _ => unreachable!("Select returned an unknown index"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Mudlark Config Construction
// ═══════════════════════════════════════════════════════════════════════════════

/// Constructs a `torrust_mudlark::Config` from an `IdentityBudget`.
///
/// Every graph sizing field comes directly from the registration budget. The
/// shipped budget keeps the historical two-level create/evict buffer and the
/// same split threshold, while a host may override either gate or threshold
/// without a second derivation here. A budget observation never exceeds the
/// configured maximum because it evicts before returning
/// (´[MUDLARK-claim:observe:a-budgeted-graph-never-exceeds-its-budget-because-observation-evicts-before-returning]´).
const fn mudlark_config_from_budget(budget: &crate::types::IdentityBudget) -> torrust_mudlark::Config<u64> {
    torrust_mudlark::Config {
        split_threshold: budget.split_threshold,
        depth_create: budget.depth_create,
        depth_evict: budget.depth_evict,
        budget: Some(budget.max_cells),
        alpha_relax: 0.75,
        bounded_eviction: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::IdentityBudget;

    /// Every graph threshold in the Mudlark configuration is copied from the
    /// public identity budget, while the shipped budget constructor reproduces
    /// the former derived values for the registration's competitive cutoff.
    ///
    /// ´claim:identity:registration-copies-host-graph-thresholds-without-deriving-them-downstream´
    /// ´test:unit:mudlark-config-uses-registered-identity-thresholds´
    #[test]
    fn mudlark_config_uses_registered_identity_thresholds() {
        let shipped = IdentityBudget::for_depth_cutoff(24);
        assert_eq!(shipped.split_threshold, 10);
        assert_eq!(shipped.depth_create, 24);
        assert_eq!(shipped.depth_evict, 26);
        assert!((shipped.spatial_decay_rate - 0.998).abs() < f64::EPSILON);

        let supplied = IdentityBudget {
            max_cells: 4096,
            observation_capacity: 512,
            split_threshold: 37,
            depth_create: 7,
            depth_evict: 11,
            spatial_decay_rate: 0.97,
        };
        let config = mudlark_config_from_budget(&supplied);

        assert_eq!(config.split_threshold, supplied.split_threshold);
        assert_eq!(config.depth_create, supplied.depth_create);
        assert_eq!(config.depth_evict, supplied.depth_evict);
        assert_eq!(config.budget, Some(supplied.max_cells));
    }
}
