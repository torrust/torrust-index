// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Signal shape and value definitions.
//!
//! This module provides the core types for signal encoding in the Assayer:
//!
//! - **`SignalShape`** — Describes how a signal should be encoded into features.
//! - **`SignalValue`** — A runtime signal value (numeric, categorical, or vector).
//! - **`Persistence`** — Whether a signal persists per-entity or per-request.
//! - **`SignalDeclaration`** — A named signal with its shape and persistence.
//!
//! # Cross-References
//!
//! - (´schema:signal:shapes´) — the declared shapes and the runtime values
//!   they accept
//! - (´rem:signal:shape-widths´) — the feature width each shape occupies
//! - (´dec:surface:sanitise-not-reject´) — what encoding does with an
//!   anomalous value

// Items in this module are re-exported via crate::signal and used by tests.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

// ═══════════════════════════════════════════════════════════════════════════════
// SignalShape
// ═══════════════════════════════════════════════════════════════════════════════

/// Describes how a signal value should be encoded into feature space.
///
/// Each variant specifies the encoding transformation and the resulting
/// feature width (´schema:signal:shapes´).
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SignalShape {
    /// A scalar value clamped to `[clip.0, clip.1]`.
    ///
    /// Feature width: 1.
    Scalar {
        /// `(min, max)` clipping bounds.
        clip: (f64, f64),
    },

    /// A log-scaled value: `ln(1 + |val|/divisor) * sign(val)`.
    ///
    /// Useful for values spanning many orders of magnitude.
    /// Feature width: 1.
    LogScaled {
        /// Divisor for scaling before log transform.
        divisor: f64,
    },

    /// A binary indicator (0.0 or 1.0, threshold at 0.5).
    ///
    /// Feature width: 1.
    Binary,

    /// An ordinal value normalised to `[0, 1]` by dividing by `max`.
    ///
    /// Feature width: 1.
    Ordinal {
        /// Maximum value for normalisation.
        max: f64,
    },

    /// A cyclic value encoded as `[sin(2π * val / period), cos(2π * val / period)]`.
    ///
    /// Useful for time-of-day, day-of-week, etc.
    /// Feature width: 2.
    Cyclic {
        /// Period of the cycle.
        period: f64,
    },

    /// A categorical value hashed to a one-hot vector.
    ///
    /// The category string is hashed, and the hash modulo `width` determines
    /// which position in the one-hot vector is set to 1.0.
    /// Feature width: `width`.
    HashedCategorical {
        /// Number of bins in the one-hot encoding.
        width: usize,
    },

    /// A fixed-length vector with per-element clamping.
    ///
    /// Feature width: `len`.
    Vector {
        /// Number of elements in the vector.
        len: usize,
        /// `(min, max)` clipping bounds applied to each element.
        clip: (f64, f64),
    },
}

impl SignalShape {
    /// Returns the number of features this shape produces.
    ///
    /// # Examples
    ///
    /// ```
    /// use torrust_assayer::SignalShape;
    ///
    /// assert_eq!(SignalShape::Scalar { clip: (0.0, 1.0) }.feature_width(), 1);
    /// assert_eq!(SignalShape::Cyclic { period: 24.0 }.feature_width(), 2);
    /// assert_eq!(SignalShape::HashedCategorical { width: 8 }.feature_width(), 8);
    /// assert_eq!(SignalShape::Vector { len: 5, clip: (0.0, 1.0) }.feature_width(), 5);
    /// ```
    #[must_use]
    pub const fn feature_width(&self) -> usize {
        match self {
            Self::Scalar { .. } | Self::LogScaled { .. } | Self::Binary | Self::Ordinal { .. } => 1,
            Self::Cyclic { .. } => 2,
            Self::HashedCategorical { width } => *width,
            Self::Vector { len, .. } => *len,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SignalValue
// ═══════════════════════════════════════════════════════════════════════════════

/// A runtime signal value.
///
/// Signals come in three flavours: numeric scalars, categorical strings, and
/// numeric vectors (´schema:signal:shapes´).
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SignalValue {
    /// A numeric scalar value.
    Numeric(f64),

    /// A categorical value represented as a string.
    Categorical(String),

    /// A vector of numeric values.
    Vector(Vec<f64>),
}

impl SignalValue {
    /// Returns `true` if all numeric components are finite (not NaN or Inf).
    ///
    /// Categorical values are always considered finite.
    #[must_use]
    pub fn is_finite(&self) -> bool {
        match self {
            Self::Numeric(v) => v.is_finite(),
            Self::Categorical(_) => true,
            Self::Vector(v) => v.iter().all(|x| x.is_finite()),
        }
    }
}

impl From<f64> for SignalValue {
    fn from(v: f64) -> Self {
        Self::Numeric(v)
    }
}

impl From<String> for SignalValue {
    fn from(s: String) -> Self {
        Self::Categorical(s)
    }
}

impl From<&str> for SignalValue {
    fn from(s: &str) -> Self {
        Self::Categorical(s.to_owned())
    }
}

impl From<Vec<f64>> for SignalValue {
    fn from(v: Vec<f64>) -> Self {
        Self::Vector(v)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Persistence
// ═══════════════════════════════════════════════════════════════════════════════

/// Signal persistence mode.
///
/// Determines whether a signal value is cached per-entity across requests,
/// or only applies to the current request.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Persistence {
    /// Signal persists per-entity across requests.
    ///
    /// Cached values are retained and merged with new values.
    Entity,

    /// Signal applies only to the current request.
    ///
    /// Not stored in the entity cache; overlaid at merge time.
    Request,
}

// ═══════════════════════════════════════════════════════════════════════════════
// SignalDeclaration
// ═══════════════════════════════════════════════════════════════════════════════

/// A named signal with its encoding shape and persistence mode.
///
/// Declarations are collected into a `SignalSchemaIndex` to define the
/// feature space layout.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SignalDeclaration {
    /// Unique name for this signal.
    pub name: String,

    /// How the signal value should be encoded.
    pub shape: SignalShape,

    /// Whether this signal persists per-entity or per-request.
    pub persistence: Persistence,
}

impl SignalDeclaration {
    /// Creates a new signal declaration.
    #[must_use]
    pub fn new(name: impl Into<String>, shape: SignalShape, persistence: Persistence) -> Self {
        Self {
            name: name.into(),
            shape,
            persistence,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Shape compatibility
// ═══════════════════════════════════════════════════════════════════════════════

/// Returns `true` when the runtime `value` is compatible with `shape`.
///
/// Compatibility rules, matching declared shapes against the raw values the
/// host supplies (´dec:surface:internal-encoding´):
///
/// - `Numeric` matches `Scalar`, `LogScaled`, `Binary`, `Ordinal`, `Cyclic`.
/// - `Categorical` matches `HashedCategorical`.
/// - `Vector` matches `Vector`.
///
/// All other combinations are mismatches that [`encode_signal`] handles by
/// producing a zero-filled vector. Callers in the assessment pipeline use
/// this predicate to count mismatches into
/// `DegradationContext::signals_shape_mismatched`.
#[must_use]
pub const fn shape_matches(value: &SignalValue, shape: &SignalShape) -> bool {
    matches!(
        (value, shape),
        (
            SignalValue::Numeric(_),
            SignalShape::Scalar { .. }
                | SignalShape::LogScaled { .. }
                | SignalShape::Binary
                | SignalShape::Ordinal { .. }
                | SignalShape::Cyclic { .. },
        ) | (SignalValue::Categorical(_), SignalShape::HashedCategorical { .. })
            | (SignalValue::Vector(_), SignalShape::Vector { .. })
    )
}

// ═══════════════════════════════════════════════════════════════════════════════
// Encoding
// ═══════════════════════════════════════════════════════════════════════════════

/// Encodes a signal value according to its shape.
///
/// # Sanitisation
///
/// NaN and infinite values are replaced with 0.0: anomalous values are
/// sanitised rather than rejected (´dec:surface:sanitise-not-reject´).
///
/// # Panics
///
/// Does not panic. Invalid shape/value combinations (e.g., `Categorical` with
/// `Scalar` shape) produce a vector of zeros.
///
/// # Examples
///
/// ```
/// use torrust_assayer::{SignalShape, SignalValue};
/// use torrust_assayer::signal_export::encode_signal;
///
/// // Scalar encoding
/// let encoded = encode_signal(&SignalValue::Numeric(3.14), &SignalShape::Scalar { clip: (0.0, 10.0) });
/// assert_eq!(encoded, vec![3.14]);
///
/// // Cyclic encoding (phase = 0.25 → sin(π/2), cos(π/2))
/// let encoded = encode_signal(&SignalValue::Numeric(0.25), &SignalShape::Cyclic { period: 1.0 });
/// assert!((encoded[0] - 1.0).abs() < 1e-10);  // sin(π/2) ≈ 1
/// assert!(encoded[1].abs() < 1e-10);          // cos(π/2) ≈ 0
/// ```
#[must_use]
pub fn encode_signal(value: &SignalValue, shape: &SignalShape) -> Vec<f64> {
    match (value, shape) {
        // ─── Scalar ──────────────────────────────────────────────────────────
        (SignalValue::Numeric(v), SignalShape::Scalar { clip: (lo, hi) }) => {
            vec![sanitise_and_clamp(*v, *lo, *hi)]
        }

        // ─── LogScaled ───────────────────────────────────────────────────────
        (SignalValue::Numeric(v), SignalShape::LogScaled { divisor }) => {
            let v = sanitise(*v);
            let scaled = (v.abs() / divisor).ln_1p() * v.signum();
            vec![sanitise(scaled)]
        }

        // ─── Binary ──────────────────────────────────────────────────────────
        (SignalValue::Numeric(v), SignalShape::Binary) => {
            let v = sanitise(*v);
            vec![if v >= 0.5 { 1.0 } else { 0.0 }]
        }

        // ─── Ordinal ─────────────────────────────────────────────────────────
        (SignalValue::Numeric(v), SignalShape::Ordinal { max }) => {
            let v = sanitise(*v);
            // The quotient is sanitised like the five sibling arms': the
            // module contract replaces non-finite values with zero, and a
            // clamp passes a quotient that is not a number through unchanged.
            vec![sanitise((v / max).clamp(0.0, 1.0))]
        }

        // ─── Cyclic ──────────────────────────────────────────────────────────
        (SignalValue::Numeric(v), SignalShape::Cyclic { period }) => {
            let v = sanitise(*v);
            let phase = 2.0 * std::f64::consts::PI * v / period;
            vec![sanitise(phase.sin()), sanitise(phase.cos())]
        }

        // ─── HashedCategorical ───────────────────────────────────────────────
        (SignalValue::Categorical(s), SignalShape::HashedCategorical { width }) => {
            let mut one_hot = vec![0.0; *width];
            if *width > 0 {
                let mut hasher = DefaultHasher::new();
                s.hash(&mut hasher);
                #[allow(clippy::cast_possible_truncation)] // Modulo ensures value fits
                let index = (hasher.finish() as usize) % width;
                one_hot[index] = 1.0;
            }
            one_hot
        }

        // ─── Vector ──────────────────────────────────────────────────────────
        (SignalValue::Vector(vec), SignalShape::Vector { len, clip: (lo, hi) }) => {
            let mut result = vec![0.0; *len];
            for (i, &v) in vec.iter().take(*len).enumerate() {
                result[i] = sanitise_and_clamp(v, *lo, *hi);
            }
            result
        }

        // ─── Mismatched shape/value ──────────────────────────────────────────
        // Return zeros for the expected width. This is a defensive fallback.
        _ => vec![0.0; shape.feature_width()],
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════════════════

/// Sanitises a float, replacing NaN and infinity with 0.0.
#[inline]
const fn sanitise(v: f64) -> f64 {
    if v.is_finite() { v } else { 0.0 }
}

/// Sanitises and clamps a float to `[lo, hi]`.
#[inline]
#[allow(clippy::missing_const_for_fn)] // clamp is not const
fn sanitise_and_clamp(v: f64, lo: f64, hi: f64) -> f64 {
    sanitise(v).clamp(lo, hi)
}
