// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Signal schema index for feature-space layout.
//!
//! This module provides `SignalSchemaIndex`, which maps signal declarations
//! to their offsets in the feature vector. It handles:
//!
//! - **Layout computation** — Sequential offsets based on signal widths.
//! - **Bulk encoding** — Encoding all provided signals into a feature vector.
//! - **Persistence tracking** — Identifying which features are entity-persistent.
//!
//! # Cross-References
//!
//! - (´req:signal:schema-fixed´) — why the schema cannot change after
//!   construction
//! - (´rem:signal:shape-widths´) — the widths the sequential offsets are built
//!   from
//! - (´dec:vector:sole-resolver´) — the one resolver every feature index goes
//!   through

// Items in this module are re-exported via crate::signal and used by tests.
#![allow(dead_code)]

use std::collections::HashMap;

use indexmap::IndexMap;

use super::value::{Persistence, SignalDeclaration, SignalShape, SignalValue, encode_signal};
use crate::error::BuildError;
use crate::feature::standardisation::FeatureClass;

// ═══════════════════════════════════════════════════════════════════════════════
// Declaration validation
// ═══════════════════════════════════════════════════════════════════════════════

/// Refuses a declaration whose shape parameters the encoding cannot
/// honour (´dec:degradation:error-partition´): a clipping interval with
/// reversed or non-finite bounds panics in the clamping helper on the
/// assessment path, a divisor, maximum or period of zero yields a
/// quotient that is not a number, and a zero hashed or vector width
/// declares a signal with nowhere to encode to.
fn validate_shape(decl: &SignalDeclaration) -> Result<(), BuildError> {
    let refuse = |reason: &'static str| {
        Err(BuildError::InvalidSignalShape {
            name: decl.name.clone(),
            reason,
        })
    };

    match &decl.shape {
        SignalShape::Scalar { clip: (lo, hi) } | SignalShape::Vector { clip: (lo, hi), .. }
            if !lo.is_finite() || !hi.is_finite() =>
        {
            refuse("clipping bounds must be finite")
        }
        SignalShape::Scalar { clip: (lo, hi) } | SignalShape::Vector { clip: (lo, hi), .. } if lo > hi => {
            refuse("clipping bounds must not be reversed")
        }
        SignalShape::Vector { len: 0, .. } => refuse("vector width must be at least 1"),
        SignalShape::LogScaled { divisor } if *divisor == 0.0 => refuse("divisor must not be zero"),
        SignalShape::Ordinal { max } if *max == 0.0 => refuse("maximum must not be zero"),
        SignalShape::Cyclic { period } if *period == 0.0 => refuse("period must not be zero"),
        SignalShape::HashedCategorical { width: 0 } => refuse("hashed width must be at least 1"),
        _ => Ok(()),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SignalSchemaIndex
// ═══════════════════════════════════════════════════════════════════════════════

/// Maps signal declarations to their positions in the feature vector.
///
/// The schema index provides:
/// - Total feature width (`p_sig`) for vector allocation.
/// - Per-signal offsets for encoding.
/// - Persistence masks for cache merge operations.
///
/// # Example
///
/// ```
/// use torrust_assayer::{SignalDeclaration, SignalShape, Persistence, SignalValue};
/// use torrust_assayer::signal_export::SignalSchemaIndex;
/// use std::collections::HashMap;
///
/// let decls = vec![
///     SignalDeclaration::new("score", SignalShape::Scalar { clip: (0.0, 1.0) }, Persistence::Entity),
///     SignalDeclaration::new("flag", SignalShape::Binary, Persistence::Request),
/// ];
///
/// let schema = SignalSchemaIndex::from_declarations(&decls).unwrap();
/// assert_eq!(schema.total_width(), 2);
///
/// let mut signals = HashMap::new();
/// signals.insert("score".to_string(), SignalValue::Numeric(0.5));
/// signals.insert("flag".to_string(), SignalValue::Numeric(1.0));
///
/// let features = schema.encode_all(&signals);
/// assert_eq!(features, vec![0.5, 1.0]);
/// ```
#[derive(Clone, Debug)]
pub struct SignalSchemaIndex {
    /// Signal entries in declaration order.
    entries: IndexMap<String, SignalSchemaEntry>,
    /// Total feature width (sum of all signal widths).
    total_width: usize,
}

// TODO ´todo:code:expose-declaration-order-signalshape-iteration-for´: expose declaration-order SignalShape iteration for
// expand_signal_classes() without leaking mutable schema internals —
// declaration order is what the canonical block order is built from
// (´dec:vector:block-order´).

/// Internal entry for a single signal in the schema.
#[derive(Clone, Debug)]
struct SignalSchemaEntry {
    /// Offset into the feature vector.
    offset: usize,
    /// Number of features this signal produces.
    width: usize,
    /// Encoding shape.
    shape: SignalShape,
    /// Persistence mode.
    persistence: Persistence,
}

impl SignalSchemaIndex {
    /// Creates a schema index from signal declarations.
    ///
    /// Computes sequential offsets based on each signal's feature width.
    ///
    /// # Errors
    ///
    /// Returns `BuildError::DuplicateSignalName` if any two declarations
    /// have the same name, and `BuildError::InvalidSignalShape` for shape
    /// parameters the encoding cannot honour — a clipping interval with
    /// reversed or non-finite bounds, a divisor, maximum or period of
    /// zero, or a hashed or vector width of zero. A structural violation
    /// is refused here, on the fallible surface, rather than admitted to
    /// reach the infallible assessment call
    /// (´dec:degradation:error-partition´), (´dec:degradation:infallible-core´).
    ///
    /// # Example
    ///
    /// ```
    /// use torrust_assayer::{SignalDeclaration, SignalShape, Persistence};
    /// use torrust_assayer::signal_export::SignalSchemaIndex;
    ///
    /// let decls = vec![
    ///     SignalDeclaration::new("a", SignalShape::Scalar { clip: (0.0, 1.0) }, Persistence::Entity),
    ///     SignalDeclaration::new("b", SignalShape::Cyclic { period: 24.0 }, Persistence::Request),
    /// ];
    ///
    /// let schema = SignalSchemaIndex::from_declarations(&decls).unwrap();
    /// assert_eq!(schema.total_width(), 3); // 1 + 2
    /// ```
    pub fn from_declarations(decls: &[SignalDeclaration]) -> Result<Self, BuildError> {
        let mut entries = IndexMap::with_capacity(decls.len());
        let mut offset = 0;

        for decl in decls {
            if entries.contains_key(&decl.name) {
                return Err(BuildError::DuplicateSignalName { name: decl.name.clone() });
            }
            validate_shape(decl)?;

            let width = decl.shape.feature_width();
            entries.insert(
                decl.name.clone(),
                SignalSchemaEntry {
                    offset,
                    width,
                    shape: decl.shape.clone(),
                    persistence: decl.persistence,
                },
            );
            offset += width;
        }

        Ok(Self {
            entries,
            total_width: offset,
        })
    }

    /// Returns the total feature width (`p_sig`).
    #[must_use]
    pub const fn total_width(&self) -> usize {
        self.total_width
    }

    /// Returns the number of declared signals.
    #[must_use]
    pub fn signal_count(&self) -> usize {
        self.entries.len()
    }

    /// Encodes all provided signals into a feature vector.
    ///
    /// Returns a `total_width()`-length vector with:
    /// - Provided signals encoded at their correct offsets.
    /// - Unprovided signals as zeros.
    ///
    /// # Example
    ///
    /// ```
    /// use torrust_assayer::{SignalDeclaration, SignalShape, Persistence, SignalValue};
    /// use torrust_assayer::signal_export::SignalSchemaIndex;
    /// use std::collections::HashMap;
    ///
    /// let decls = vec![
    ///     SignalDeclaration::new("a", SignalShape::Scalar { clip: (0.0, 1.0) }, Persistence::Entity),
    ///     SignalDeclaration::new("b", SignalShape::Scalar { clip: (0.0, 1.0) }, Persistence::Entity),
    /// ];
    /// let schema = SignalSchemaIndex::from_declarations(&decls).unwrap();
    ///
    /// // Only provide "a"
    /// let mut signals = HashMap::new();
    /// signals.insert("a".to_string(), SignalValue::Numeric(0.75));
    ///
    /// let features = schema.encode_all(&signals);
    /// assert_eq!(features, vec![0.75, 0.0]);
    /// ```
    #[must_use]
    pub fn encode_all(&self, provided: &HashMap<String, SignalValue>) -> Vec<f64> {
        let mut result = vec![0.0; self.total_width];

        for (name, entry) in &self.entries {
            if let Some(value) = provided.get(name) {
                let encoded = encode_signal(value, &entry.shape);
                // Copy encoded values to the correct offset
                for (i, &v) in encoded.iter().enumerate() {
                    if entry.offset + i < result.len() {
                        result[entry.offset + i] = v;
                    }
                }
            }
        }

        result
    }

    /// Returns a mask indicating which positions are entity-persistent.
    ///
    /// `true` at position `i` means that feature is from an entity-persistent
    /// signal and should be cached.
    ///
    /// # Example
    ///
    /// ```
    /// use torrust_assayer::{SignalDeclaration, SignalShape, Persistence};
    /// use torrust_assayer::signal_export::SignalSchemaIndex;
    ///
    /// let decls = vec![
    ///     SignalDeclaration::new("entity_sig", SignalShape::Scalar { clip: (0.0, 1.0) }, Persistence::Entity),
    ///     SignalDeclaration::new("request_sig", SignalShape::Scalar { clip: (0.0, 1.0) }, Persistence::Request),
    /// ];
    /// let schema = SignalSchemaIndex::from_declarations(&decls).unwrap();
    ///
    /// let mask = schema.entity_persistent_mask();
    /// assert_eq!(mask, vec![true, false]);
    /// ```
    #[must_use]
    pub fn entity_persistent_mask(&self) -> Vec<bool> {
        let mut mask = vec![false; self.total_width];

        for entry in self.entries.values() {
            if entry.persistence == Persistence::Entity {
                for i in 0..entry.width {
                    mask[entry.offset + i] = true;
                }
            }
        }

        mask
    }

    /// Looks up a signal entry by name.
    ///
    /// Returns the offset and width for the named signal, if it exists.
    #[must_use]
    pub(crate) fn get(&self, name: &str) -> Option<(usize, usize)> {
        self.entries.get(name).map(|e| (e.offset, e.width))
    }

    /// Looks up a signal entry by name, including its shape.
    ///
    /// Returns `(offset, width, shape)` for direct per-signal encoding
    /// without allocating a full feature vector.
    #[must_use]
    pub(crate) fn get_with_shape(&self, name: &str) -> Option<(usize, usize, &SignalShape)> {
        self.entries.get(name).map(|e| (e.offset, e.width, &e.shape))
    }

    /// Returns an iterator over signal names in declaration order.
    pub fn signal_names(&self) -> impl Iterator<Item = &str> {
        self.entries.keys().map(String::as_str)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Signal class expansion
// ═══════════════════════════════════════════════════════════════════════════════

/// Expands declared signal shapes into one standardisation class per position.
///
/// The class assignment takes three authorities and the signal schema is the
/// one for the signal block (´req:standardisation:class-assignment´): a
/// declaration's shape is what says how its positions should be standardised,
/// and a shape occupying more than one position gives the same class to each
/// of them. The result is `p_sig` long, so it indexes the signal block
/// directly.
///
/// A scalar and a vector component are z-scored; a log-scaled value carries
/// its own class; a binary indicator and an ordinal are bounded rather than
/// unbounded, the ordinal being normalised into the unit interval; cyclic and
/// hashed-categorical encodings carry the classes named for them
/// (´tab:standardisation:class-priors´).
#[must_use]
pub fn expand_signal_classes(declarations: &[SignalDeclaration]) -> Vec<FeatureClass> {
    let mut classes = Vec::new();
    for declaration in declarations {
        let class = match declaration.shape {
            SignalShape::Scalar { .. } | SignalShape::Vector { .. } => FeatureClass::ZScore,
            SignalShape::LogScaled { .. } => FeatureClass::LogScaled,
            SignalShape::Binary => FeatureClass::Binary,
            SignalShape::Ordinal { .. } => FeatureClass::RateOrFraction,
            SignalShape::Cyclic { .. } => FeatureClass::Cyclic,
            SignalShape::HashedCategorical { .. } => FeatureClass::HashedCategorical,
        };
        classes.extend(std::iter::repeat_n(class, declaration.shape.feature_width()));
    }
    classes
}
