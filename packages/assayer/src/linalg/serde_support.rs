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
    if wrapper.data.len() != wrapper.nrows * wrapper.ncols {
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
        if wrapper.data.len() != wrapper.p * wrapper.p {
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
