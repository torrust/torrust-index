// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! A quiescent checkpoint fork with one scenario clock and independently owned storage (´cor:durability:harness-fork´).

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use super::{Clock, World};
use crate::config::types::PersistenceConfig;
use crate::error::LifecycleError;

/// A root created exclusively beneath the same temporary directory used by persistence fixtures.
pub(super) struct PersistenceRoot(PathBuf);

impl PersistenceRoot {
    fn create() -> Result<Self, std::io::Error> {
        static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);
        let serial = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("assayer-fork-{}-{serial}", std::process::id()));
        // Exclusive creation refuses an existing root; no arm can adopt another instance's files.
        fs::create_dir(&path)?;
        Ok(Self(path))
    }
}

impl Drop for PersistenceRoot {
    fn drop(&mut self) {
        if let Err(error) = fs::remove_dir_all(&self.0) {
            tracing::warn!(path = ?self.0, %error, "could not remove persistence fork storage");
        }
    }
}

/// Uninterrupted and restored Worlds whose durable files have separate lifetimes.
///
/// Storage is owned by the restored World itself, so moving the arms out of the fork retains the root until that engine's threads have shut down. Names and host-owned policies are copied, while models, Ledgers and owner state are rebuilt exclusively from the durable copy.
pub struct PersistenceFork {
    uninterrupted: World,
    restored: World,
    storage: [PersistenceConfig; 2],
    wall: Duration,
}

impl PersistenceFork {
    pub(super) fn new(
        uninterrupted: World,
        elapsed: Duration,
        rebind: impl FnOnce(&World) -> Result<(), LifecycleError>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let original = uninterrupted
            .assayer()
            .config()
            .persistence
            .clone()
            .ok_or("fork requires persistence")?;
        uninterrupted.quiesce_persistence()?;
        let started = Instant::now();
        let root = PersistenceRoot::create()?;
        let mut copied = original.clone();
        copied.checkpoint_dir = root.0.join("checkpoint");
        copied.journal_dir = root.0.join("journal");
        fs::create_dir(&copied.checkpoint_dir)?;
        fs::create_dir(&copied.journal_dir)?;
        {
            // The acknowledged barriers fix the prefix; parking excludes later scheduler checkpoints during the copy.
            let parked = uninterrupted.block_model_owner_for_test();
            fs::copy(original.checkpoint_path(), copied.checkpoint_path())?;
            fs::copy(original.journal_path(), copied.journal_path())?;
            parked.release();
        }
        // Move the shared clock once, after preserving the checkpoint timestamp and before restore reads it.
        uninterrupted.advance(elapsed)?;
        let restored = uninterrupted.rebuild_persistence(copied.clone(), root)?;
        let restore_time = restored.clock().now();
        rebind(&restored)?;
        if restored.clock().now() != restore_time {
            return Err("identity rebinding advanced the fork clock".into());
        }
        let identities = |world: &World| -> BTreeSet<_> {
            world
                .assayer()
                .identity_dimensions
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .keys()
                .map(|id| id.0)
                .collect()
        };
        if identities(&uninterrupted) != identities(&restored) {
            return Err("fork requires every process-local identity registration to be rebound".into());
        }
        restored.flush_identity_maintenance()?;
        let wall = started.elapsed();
        Ok(Self {
            uninterrupted,
            restored,
            storage: [original, copied],
            wall,
        })
    }

    /// Borrow both arms without changing their shared scenario time.
    #[must_use]
    pub const fn arms(&self) -> (&World, &World) {
        (&self.uninterrupted, &self.restored)
    }

    /// Retain both engines and their storage ownership outside the fork wrapper.
    #[must_use]
    pub fn into_arms(self) -> (World, World) {
        (self.uninterrupted, self.restored)
    }

    /// Checkpoint and journal roots for each arm, in uninterrupted/restored order.
    #[must_use]
    pub fn roots(&self) -> [(&Path, &Path); 2] {
        self.storage
            .each_ref()
            .map(|storage| (storage.checkpoint_dir.as_path(), storage.journal_dir.as_path()))
    }

    /// Copy and rebuild wall, including storage creation and the declared downtime barriers, excluding the common suffix.
    #[must_use]
    pub const fn wall(&self) -> Duration {
        self.wall
    }

    /// Advance both arms' one clock once, then settle each arm's time-activated work.
    ///
    /// # Errors
    /// Returns the failing queue barrier's diagnostic.
    pub fn advance(&self, elapsed: Duration) -> Result<(), super::WorldFixtureError> {
        self.uninterrupted.advance(elapsed)?;
        self.restored.flush_identity_maintenance()?;
        self.restored.flush_ledger_gc()?;
        Ok(())
    }

    /// Freeze owned journal bytes while both model owners are parked, for the storage-isolation witness.
    ///
    /// # Errors
    /// Returns the filesystem error if either journal cannot be read.
    ///
    /// # Panics
    /// Panics only if a model owner cannot enter its bounded parked state, which live quiescent arms cannot arrange.
    pub fn journal_bytes(&self) -> Result<[Vec<u8>; 2], std::io::Error> {
        let left = self.uninterrupted.block_model_owner_for_test();
        let right = self.restored.block_model_owner_for_test();
        let bytes = [
            fs::read(self.storage[0].journal_path())?,
            fs::read(self.storage[1].journal_path())?,
        ];
        right.release();
        left.release();
        Ok(bytes)
    }
}

/// Read one frozen checkpoint; precision remains confined to crate-level witnesses.
#[cfg(test)]
pub fn checkpoint(world: &World) -> Result<crate::persistence::checkpoint::CheckpointPayload, Box<dyn std::error::Error>> {
    world.quiesce_persistence()?;
    let parked = world.block_model_owner_for_test();
    let persistence = world
        .assayer()
        .config()
        .persistence
        .as_ref()
        .ok_or("projection requires persistence")?;
    let payload = crate::persistence::checkpoint::read_checkpoint(&persistence.checkpoint_path())?;
    parked.release();
    Ok(payload)
}
