// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Signal cache for entity-persistent features.
//!
//! This module provides `SignalCache`, an LRU cache that stores per-entity
//! feature vectors. The cache supports:
//!
//! - **Entity persistence** — Features from entity-persistent signals are
//!   retained across requests.
//! - **Merge semantics** — New values overwrite cached values; request-scoped
//!   signals overlay without caching.
//! - **Health tracking** — Hit/miss/eviction counters for monitoring.
//!
//! # Cross-References
//!
//! - (´dec:retention:cache-preencoded´) — why entity-persistent signals are
//!   cached already encoded
//! - (´dec:retention:cache-losable´) — why losing the cache is allowed
//! - (´dec:health:concrete-trackers´) — why this cache carries its own
//!   counters

// Items in this module are re-exported via crate::signal and used by tests.

use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crossbeam_channel::{Receiver, Sender};
use lru::LruCache;

use super::schema::SignalSchemaIndex;
use super::value::{SignalValue, encode_signal};
use crate::types::EntityKey;

// ═══════════════════════════════════════════════════════════════════════════════
// SignalCache
// ═══════════════════════════════════════════════════════════════════════════════

/// An LRU cache for entity-persistent signal features.
///
/// The cache stores per-entity feature vectors. On each access:
///
/// 1. Load the cached vector (or zeros if absent), without touching
///    the recency order.
/// 2. Overwrite entity-persistent positions from provided signals.
/// 3. Defer the write: the update is enqueued, not applied
///    (´inv:runtime:enumerated-writes´).
/// 4. Overlay request-scoped signals without caching.
/// 5. Return the merged vector.
///
/// The deferred writes are applied by [`Self::drain_deferred_writes`],
/// which the identity maintenance thread calls on its cycle — the same
/// drainage the deferred identity observation uses
/// (´alg:publication:identity-draining´). The assessment path therefore
/// enqueues and never mutates the cache: its returned vector always
/// carries the values it was handed, and the cached copy catches up at
/// the drain.
///
/// # Thread Safety
///
/// The cache uses a `Mutex` for interior mutability. The assessment
/// path takes it only to read; mutation happens on the drain.
///
/// # Example
///
/// ```
/// use torrust_assayer::signal_export::{SignalCache, SignalSchemaIndex};
/// use torrust_assayer::{
///     SignalDeclaration, SignalShape, Persistence, SignalValue,
/// };
/// use torrust_assayer::types::EntityKey;
/// use std::sync::Arc;
/// use std::collections::HashMap;
///
/// // Build schema
/// let decls = vec![
///     SignalDeclaration::new("auth", SignalShape::Scalar { clip: (0.0, 1.0) }, Persistence::Entity),
///     SignalDeclaration::new("ip_rep", SignalShape::Scalar { clip: (0.0, 1.0) }, Persistence::Request),
/// ];
/// let schema = Arc::new(SignalSchemaIndex::from_declarations(&decls).unwrap());
///
/// // Create cache
/// let cache = SignalCache::new(1000, schema);
///
/// // First request for entity A
/// let entity_a = EntityKey::new(b"user-123".to_vec());
/// let mut signals = HashMap::new();
/// signals.insert("auth".to_string(), SignalValue::Numeric(0.9));
/// signals.insert("ip_rep".to_string(), SignalValue::Numeric(0.5));
///
/// let features = cache.get_and_merge(&entity_a, &signals);
/// assert_eq!(features, vec![0.9, 0.5]);
///
/// // The write was deferred; the drain applies it to the cache.
/// cache.drain_deferred_writes();
///
/// // Second request — auth is cached, ip_rep is new
/// let mut signals2 = HashMap::new();
/// signals2.insert("ip_rep".to_string(), SignalValue::Numeric(0.8));
///
/// let features2 = cache.get_and_merge(&entity_a, &signals2);
/// assert_eq!(features2, vec![0.9, 0.8]); // auth retained from cache
/// ```
pub struct SignalCache {
    /// The LRU cache, keyed by `EntityKey`.
    cache: Mutex<LruCache<EntityKey, Vec<f64>>>,

    /// The schema defining feature layout and persistence.
    schema: Arc<SignalSchemaIndex>,

    /// Entity-persistent mask (cached from schema for fast access).
    entity_mask: Vec<bool>,

    /// The deferred-write queue's sender: the assessment path enqueues
    /// here and never takes the cache lock for writing
    /// (´inv:runtime:enumerated-writes´). Bounded at the cache's own
    /// capacity, so the queue carries no figure of its own.
    deferred_tx: Sender<DeferredCacheWrite>,

    /// The deferred-write queue's receiver, drained by
    /// [`Self::drain_deferred_writes`] on the maintenance thread.
    deferred_rx: Receiver<DeferredCacheWrite>,

    /// Deferred writes dropped on a full queue (degradation, not error:
    /// a lost write costs a stale cached vector until the next one).
    deferred_dropped: AtomicU64,

    /// Cache hit counter.
    hits: AtomicU64,

    /// Cache miss counter.
    misses: AtomicU64,

    /// Eviction counter.
    evictions: AtomicU64,
}

/// A cache mutation deferred off the assessment path.
enum DeferredCacheWrite {
    /// Apply encoded entity-persistent updates to the entity's vector.
    Put {
        /// The entity whose vector is updated.
        entity: EntityKey,
        /// Encoded `(offset, values)` runs, entity-persistent positions only.
        updates: Vec<(usize, Vec<f64>)>,
    },
    /// Refresh the entity's recency without changing its vector.
    Touch(EntityKey),
}

impl SignalCache {
    /// Creates a new signal cache.
    ///
    /// # Arguments
    ///
    /// * `capacity` — Maximum number of entities to cache. Must be > 0.
    /// * `schema` — The signal schema defining feature layout.
    ///
    /// # Panics
    ///
    /// Panics if `capacity` is 0.
    #[must_use]
    pub fn new(capacity: usize, schema: Arc<SignalSchemaIndex>) -> Self {
        let cap = NonZeroUsize::new(capacity).expect("SignalCache capacity must be > 0");
        let entity_mask = schema.entity_persistent_mask();
        // The deferred-write queue is bounded at the cache's own
        // capacity: it holds no more entities than the cache can.
        let (deferred_tx, deferred_rx) = crossbeam_channel::bounded(capacity);

        Self {
            cache: Mutex::new(LruCache::new(cap)),
            schema,
            entity_mask,
            deferred_tx,
            deferred_rx,
            deferred_dropped: AtomicU64::new(0),
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            evictions: AtomicU64::new(0),
        }
    }

    /// Retrieves and merges signal features for an entity.
    ///
    /// # Process
    ///
    /// 1. Encode all provided signals (outside the lock).
    /// 2. Lock the cache and **read** the entity's vector (or zeros on a
    ///    miss), without touching the recency order.
    /// 3. Overwrite entity-persistent positions from encoded signals.
    /// 4. **Defer the write**: enqueue the entity-persistent updates (or
    ///    a recency touch on a plain hit) for the maintenance thread's
    ///    drain — the assessment path enqueues and never mutates
    ///    (´inv:runtime:enumerated-writes´).
    /// 5. Overlay request-scoped positions from encoded signals.
    /// 6. Return the merged vector.
    ///
    /// A full deferred-write queue drops the write and counts it: a lost
    /// write costs a stale cached vector until the entity's next one.
    ///
    /// # NaN Handling
    ///
    /// NaN values in provided signals are sanitised to 0.0 during encoding.
    ///
    #[must_use]
    pub fn get_and_merge(&self, entity: &EntityKey, provided: &HashMap<String, SignalValue>) -> Vec<f64> {
        let width = self.schema.total_width();
        if width == 0 {
            return Vec::new();
        }

        // Pre-encode signals outside the lock
        let encoded = self.encode_provided(provided);

        // Determine which entity-persistent signals were provided
        let has_entity_updates = self.has_entity_persistent_updates(provided);

        // Lock the cache to read. `peek` leaves the recency order alone —
        // the recency refresh travels with the deferred write.
        // Poison recovery — core assessment does not fail
        // (´dec:degradation:infallible-core´).
        let (mut features, was_hit) = {
            let cache = self.cache.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            cache
                .peek(entity)
                .map_or_else(|| (vec![0.0; width], false), |cached| (cached.clone(), true))
        };
        if was_hit {
            self.hits.fetch_add(1, Ordering::Relaxed);
        } else {
            self.misses.fetch_add(1, Ordering::Relaxed);
        }

        // Merge entity-persistent positions into the returned vector.
        if has_entity_updates {
            self.apply_entity_persistent(&mut features, &encoded);
        }

        // Defer the cache mutation.
        let deferred = if has_entity_updates {
            let updates: Vec<(usize, Vec<f64>)> = encoded
                .iter()
                .filter(|(offset, values)| (*offset..offset + values.len()).any(|i| self.entity_mask.get(i) == Some(&true)))
                .cloned()
                .collect();
            Some(DeferredCacheWrite::Put {
                entity: entity.clone(),
                updates,
            })
        } else if was_hit {
            Some(DeferredCacheWrite::Touch(entity.clone()))
        } else {
            None
        };
        if let Some(write) = deferred
            && self.deferred_tx.try_send(write).is_err()
        {
            self.deferred_dropped.fetch_add(1, Ordering::Relaxed);
        }

        // Apply request-scoped positions (outside the lock)
        self.apply_request_scoped(&mut features, &encoded);

        features
    }

    /// Applies the deferred writes to the cache.
    ///
    /// Called on the identity maintenance thread's cycle — the drainage
    /// the deferred identity observation already uses
    /// (´alg:publication:identity-draining´) — and by tests as a
    /// synchronisation point. Takes the cache lock once for the batch.
    pub fn drain_deferred_writes(&self) {
        let mut cache_guard = None;
        while let Ok(write) = self.deferred_rx.try_recv() {
            let cache = cache_guard.get_or_insert_with(|| self.cache.lock().unwrap_or_else(std::sync::PoisonError::into_inner));
            match write {
                DeferredCacheWrite::Put { entity, updates } => {
                    let existing = cache.peek(&entity).cloned();
                    let inserting_new = existing.is_none();
                    let at_capacity = cache.len() == cache.cap().get();
                    let mut features = existing.unwrap_or_else(|| vec![0.0; self.schema.total_width()]);
                    self.apply_entity_persistent(&mut features, &updates);
                    cache.put(entity, features);
                    // A new key inserted at capacity evicted the oldest.
                    if inserting_new && at_capacity {
                        self.evictions.fetch_add(1, Ordering::Relaxed);
                    }
                }
                DeferredCacheWrite::Touch(entity) => {
                    // `get` refreshes recency; the value is untouched.
                    let _ = cache.get(&entity);
                }
            }
        }
        drop(cache_guard);
    }

    /// Counts host-provided signals whose `SignalValue` variant does not
    /// match the declared `SignalShape`, which the host supplies raw
    /// (´dec:surface:internal-encoding´).
    ///
    /// Signals not present in the schema are ignored by this counter;
    /// use [`Self::count_unknown_signals`] for that diagnostic.
    /// Returns the count saturated at `u32::MAX`.
    #[must_use]
    pub fn count_shape_mismatches(&self, provided: &HashMap<String, SignalValue>) -> u32 {
        let mut count: u32 = 0;
        for (name, value) in provided {
            if let Some((_offset, _width, shape)) = self.schema.get_with_shape(name)
                && !super::value::shape_matches(value, shape)
            {
                count = count.saturating_add(1);
            }
        }
        count
    }

    /// Counts host-provided signal names not declared in the schema.
    ///
    /// Unknown signals are skipped during encoding rather than raising: this
    /// is a data-quality problem, not a structural one
    /// (´dec:degradation:error-partition´). Returning
    /// a count lets the assessment path surface likely host/schema drift
    /// without making `assess()` fallible.
    #[must_use]
    pub fn count_unknown_signals(&self, provided: &HashMap<String, SignalValue>) -> u32 {
        let mut count: u32 = 0;
        for name in provided.keys() {
            if self.schema.get(name).is_none() {
                count = count.saturating_add(1);
            }
        }
        count
    }

    /// Returns cache health statistics.
    ///
    /// Acquires the cache lock briefly to read size and capacity.
    ///
    #[must_use]
    pub fn health(&self) -> SignalCacheHealth {
        let (size, capacity) = {
            let cache = self.cache.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            (cache.len(), cache.cap().get())
        };

        SignalCacheHealth {
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            evictions: self.evictions.load(Ordering::Relaxed),
            capacity,
            size,
        }
    }

    /// Pre-encodes all provided signals according to the schema.
    ///
    /// Encodes each signal directly via its shape
    /// (´dec:retention:cache-preencoded´), avoiding full-vector allocation
    /// per signal.
    fn encode_provided(&self, provided: &HashMap<String, SignalValue>) -> Vec<(usize, Vec<f64>)> {
        let mut result = Vec::with_capacity(provided.len());

        for (name, value) in provided {
            if let Some((offset, _width, shape)) = self.schema.get_with_shape(name) {
                let encoded = encode_signal(value, shape);
                if !encoded.is_empty() {
                    result.push((offset, encoded));
                }
            }
        }

        result
    }

    /// Checks if any entity-persistent signal was provided.
    fn has_entity_persistent_updates(&self, provided: &HashMap<String, SignalValue>) -> bool {
        for name in provided.keys() {
            if let Some((offset, width)) = self.schema.get(name) {
                // Check if any position in this signal's range is entity-persistent
                for i in offset..offset + width {
                    if i < self.entity_mask.len() && self.entity_mask[i] {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Applies entity-persistent encoded values to features.
    fn apply_entity_persistent(&self, features: &mut [f64], encoded: &[(usize, Vec<f64>)]) {
        for (offset, values) in encoded {
            for (i, &v) in values.iter().enumerate() {
                let pos = offset + i;
                if pos < features.len() && pos < self.entity_mask.len() && self.entity_mask[pos] {
                    features[pos] = v;
                }
            }
        }
    }

    /// Applies request-scoped encoded values to features.
    fn apply_request_scoped(&self, features: &mut [f64], encoded: &[(usize, Vec<f64>)]) {
        for (offset, values) in encoded {
            for (i, &v) in values.iter().enumerate() {
                let pos = offset + i;
                if pos < features.len() && pos < self.entity_mask.len() && !self.entity_mask[pos] {
                    features[pos] = v;
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SignalCacheHealth
// ═══════════════════════════════════════════════════════════════════════════════

/// Health statistics for a `SignalCache`.
///
/// This cache carries its own concrete tracker
/// (´dec:health:concrete-trackers´).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignalCacheHealth {
    /// Number of cache hits.
    pub hits: u64,

    /// Number of cache misses.
    pub misses: u64,

    /// Number of evictions due to capacity limits.
    pub evictions: u64,

    /// Maximum cache capacity (number of entities).
    pub capacity: usize,

    /// Current cache size (number of entities).
    pub size: usize,
    // TODO ´todo:code:add-schema-width-and-estimated-bytes´: add schema-width and estimated-bytes fields so
    // health endpoints can surface cache footprint scaling by p_entity.
}

impl SignalCacheHealth {
    /// Returns the hit rate as a fraction in `[0, 1]`.
    ///
    /// Returns `0.0` if no accesses have occurred.
    #[must_use]
    #[allow(clippy::cast_precision_loss)] // Precision loss acceptable for rates
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 { 0.0 } else { self.hits as f64 / total as f64 }
    }

    /// Returns the utilisation as a fraction in `[0, 1]`.
    #[must_use]
    #[allow(clippy::cast_precision_loss)] // Precision loss acceptable for utilisation
    pub fn utilisation(&self) -> f64 {
        if self.capacity == 0 {
            0.0
        } else {
            self.size as f64 / self.capacity as f64
        }
    }
}
