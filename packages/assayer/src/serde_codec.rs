// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Shared postcard serde adapter helpers.

use serde::Serialize;
use serde::de::DeserializeOwned;

/// Serializes a serde value into the crate's postcard encoding.
///
/// # Errors
///
/// Returns the postcard encode error as a string when serialisation fails.
pub fn serialize<T: Serialize>(value: &T) -> Result<Vec<u8>, String> {
    postcard::to_allocvec(value).map_err(|e| e.to_string())
}

/// Deserializes a serde value and rejects unused trailing bytes.
///
/// # Errors
///
/// Returns the postcard decode error as a string when deserialisation fails, or
/// a trailing-bytes error when the input contains unused data after the value.
pub fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, String> {
    let (value, rest) = postcard::take_from_bytes::<T>(bytes).map_err(|e| e.to_string())?;

    if !rest.is_empty() {
        return Err(format!("postcard deserialisation left {} trailing bytes", rest.len()));
    }

    Ok(value)
}
