// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Custom serialisation for `faer` types.
//!
//! `faer` provides no `serde` support. This module bridges the gap
//! with `Serialize`/`Deserialize` implementations for `Col<f64>`,
//! `Mat<f64>`, and `SymmetricMatrix`, built on top of the conversion
//! helpers in [`super::convert`].
//!
//! # Cross-References
//!
//! - (´dec:substrate:hand-serialisation´) — the serialisation strategy, written
//!   by hand rather than derived from the backend's own types
//! - (´dec:durability:checkpoint-journal´) — the persisted state this encoding
//!   is written into
//!
//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`matrix_dimension_overflow_is_a_deserialisation_error`] | persistence | Matrix dimension multiplication reports malformed persisted input on overflow, in every build profile, rather than panicking before the payload length can be rejected. |
//! | [`symmetric_dimension_overflow_is_a_deserialisation_error`] | persistence | Squaring a symmetric matrix dimension reports malformed persisted input on overflow rather than wrapping to the empty payload's length. |

use faer::{Col, Mat};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::convert::{col_to_vec, mat_to_vec, vec_to_col, vec_to_mat};
use super::symmetric::SymmetricMatrix;

// ═══════════════════════════════════════════════════════════════════════════════
// Col<f64> serde
// ═══════════════════════════════════════════════════════════════════════════════

/// Serialisable wrapper for `faer::Col<f64>`.
#[derive(Serialize, Deserialize)]
struct ColData {
    /// The column vector elements.
    data: Vec<f64>,
}

/// Serialise a `faer::Col<f64>`.
pub fn serialize_col<S: Serializer>(col: &Col<f64>, serializer: S) -> Result<S::Ok, S::Error> {
    let wrapper = ColData { data: col_to_vec(col) };
    wrapper.serialize(serializer)
}

/// Deserialise a `faer::Col<f64>`.
pub fn deserialize_col<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Col<f64>, D::Error> {
    let wrapper = ColData::deserialize(deserializer)?;
    Ok(vec_to_col(&wrapper.data))
}

// ═══════════════════════════════════════════════════════════════════════════════
// Mat<f64> serde
// ═══════════════════════════════════════════════════════════════════════════════

/// Serialisable wrapper for `faer::Mat<f64>`.
#[derive(Serialize, Deserialize)]
struct MatData {
    /// Number of rows.
    nrows: usize,
    /// Number of columns.
    ncols: usize,
    /// Column-major flat data.
    data: Vec<f64>,
}

/// Serialise a `faer::Mat<f64>`.
pub fn serialize_mat<S: Serializer>(mat: &Mat<f64>, serializer: S) -> Result<S::Ok, S::Error> {
    let wrapper = MatData {
        nrows: mat.nrows(),
        ncols: mat.ncols(),
        data: mat_to_vec(mat),
    };
    wrapper.serialize(serializer)
}

/// Deserialise a `faer::Mat<f64>`.
pub fn deserialize_mat<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Mat<f64>, D::Error> {
    let wrapper = MatData::deserialize(deserializer)?;
    let expected_len = wrapper
        .nrows
        .checked_mul(wrapper.ncols)
        .ok_or_else(|| serde::de::Error::custom("matrix dimensions overflow usize"))?;
    if wrapper.data.len() != expected_len {
        return Err(serde::de::Error::custom(format!(
            "data length {} does not match dimensions {}×{}",
            wrapper.data.len(),
            wrapper.nrows,
            wrapper.ncols,
        )));
    }
    Ok(vec_to_mat(&wrapper.data, wrapper.nrows, wrapper.ncols))
}

// ═══════════════════════════════════════════════════════════════════════════════
// SymmetricMatrix serde
// ═══════════════════════════════════════════════════════════════════════════════

/// Serialisable wrapper for `SymmetricMatrix`.
#[derive(Serialize, Deserialize)]
struct SymmetricMatrixData {
    /// Dimension p of the p×p matrix.
    p: usize,
    /// Column-major flat data.
    data: Vec<f64>,
}

impl Serialize for SymmetricMatrix {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let wrapper = SymmetricMatrixData {
            p: self.dim(),
            data: mat_to_vec(self.as_inner()),
        };
        wrapper.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SymmetricMatrix {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wrapper = SymmetricMatrixData::deserialize(deserializer)?;
        let expected_len = wrapper
            .p
            .checked_mul(wrapper.p)
            .ok_or_else(|| serde::de::Error::custom("symmetric matrix dimension overflows usize when squared"))?;
        if wrapper.data.len() != expected_len {
            return Err(serde::de::Error::custom(format!(
                "data length {} does not match dimension {}×{}",
                wrapper.data.len(),
                wrapper.p,
                wrapper.p,
            )));
        }
        let mat = vec_to_mat(&wrapper.data, wrapper.p, wrapper.p);
        Self::from_storage(mat, 1e-12).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Matrix dimension multiplication reports malformed persisted input on
    /// overflow, in every build profile, rather than panicking before the
    /// payload length can be rejected.
    ///
    /// ´claim:persistence:matrix-dimension-overflow-is-a-deserialisation-error´
    /// ´test:unit:matrix-dimension-overflow-is-a-deserialisation-error´
    #[test]
    fn matrix_dimension_overflow_is_a_deserialisation_error() {
        #[derive(Deserialize)]
        struct Wrapper {
            #[serde(deserialize_with = "deserialize_mat")]
            _matrix: Mat<f64>,
        }

        let overflowing_rows = 1_usize << (usize::BITS - 1);
        let input = serde_json::json!({
            "_matrix": {
                "nrows": overflowing_rows,
                "ncols": 2,
                "data": [],
            },
        });
        let result = std::panic::catch_unwind(|| serde_json::from_value::<Wrapper>(input));

        assert!(
            result.is_ok(),
            "matrix dimension overflow must return a deserialisation error, not panic",
        );
        let Ok(deserialised) = result else {
            unreachable!("the assertion above rejects a panic");
        };
        let Err(error) = deserialised else {
            panic!("overflowing matrix dimensions must not deserialize");
        };
        assert!(
            error.to_string().contains("overflow usize"),
            "matrix overflow error should name the arithmetic failure: {error}",
        );
    }

    /// Squaring a symmetric matrix dimension reports malformed persisted input
    /// on overflow rather than wrapping to the empty payload's length.
    ///
    /// ´claim:persistence:symmetric-matrix-dimension-overflow-is-a-deserialisation-error´
    /// ´test:unit:symmetric-dimension-overflow-is-a-deserialisation-error´
    #[test]
    fn symmetric_dimension_overflow_is_a_deserialisation_error() {
        let overflowing_dimension = 1_usize << usize::BITS.div_ceil(2);
        let input = serde_json::json!({
            "p": overflowing_dimension,
            "data": [],
        });
        let result = std::panic::catch_unwind(|| serde_json::from_value::<SymmetricMatrix>(input));

        assert!(
            result.is_ok(),
            "symmetric dimension overflow must return a deserialisation error, not panic",
        );
        let Ok(deserialised) = result else {
            unreachable!("the assertion above rejects a panic");
        };
        let Err(error) = deserialised else {
            panic!("an overflowing symmetric dimension must not deserialize");
        };
        assert!(
            error.to_string().contains("overflows usize"),
            "symmetric overflow error should name the arithmetic failure: {error}",
        );
    }
}
