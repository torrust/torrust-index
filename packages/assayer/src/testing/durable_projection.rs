// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! The complete durable schema projected into owned, field-addressable values. Precision belongs to these crate-level witnesses and never enters the public published-state probe surface (´dec:harness:probe-contract´).

use std::collections::BTreeSet;

use serde::Serialize;
use serde_json::{Map, Value};

use super::World;
use super::durable_value::DurableValue;
use crate::persistence::checkpoint::CheckpointPayload;

#[derive(Debug)]
pub struct DurableProjection(Value);

impl DurableProjection {
    pub(crate) fn capture(world: &World, restore_fields: &BTreeSet<String>) -> Result<Self, Box<dyn std::error::Error>> {
        Self::from_payload(&super::fork::checkpoint(world)?, restore_fields)
    }

    pub(crate) fn from_payload(
        payload: &CheckpointPayload,
        restore_fields: &BTreeSet<String>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        macro_rules! fields {
            ($($field:ident),+ $(,)?) => {{
                // Exhaustive destructuring makes any schema addition a compile-time refusal until it has an explicit projection field.
                let CheckpointPayload { $($field),+ } = payload;
                let mut projected = Map::new();
                $(projected.insert(stringify!($field).to_owned(), $field.serialize(DurableValue)?);)+
                projected
            }};
        }
        let mut projected = fields!(
            operational,
            sister,
            anchor,
            outcome_models,
            kappa_sister,
            kappa_anchor,
            p_positive_global,
            p_positive_eligible,
            kappa_v,
            last_processed_label_seq,
            checkpoint_timestamp,
            ledger_state,
            identity_state,
            dimension_map,
            signal_schema,
            interaction_templates,
            concordance_state,
            marginalisation_errors,
            hibernation,
            drift_state,
            feature_means,
            feature_variances,
            feature_classes,
            cold_ramp,
            owner_state,
        );
        let names: BTreeSet<_> = projected.keys().cloned().collect();
        let serialized = payload.serialize(DurableValue)?;
        let schema: BTreeSet<_> = serialized
            .as_object()
            .ok_or("checkpoint schema is not a record")?
            .keys()
            .cloned()
            .collect();
        if names != schema || !restore_fields.is_subset(&names) {
            return Err(format!(
                "durable projection schema mismatch: projected={names:?}, schema={schema:?}, restore={restore_fields:?}"
            )
            .into());
        }
        // Capture instants identify the act of checkpointing, not the durable learned state at the common suffix boundary.
        projected.insert("checkpoint_timestamp".to_owned(), Value::Null);
        // These sequence-shaped fields are keyed associations assembled from hash maps. Sort by their identity, preserving every value and all genuinely ordered histories.
        for field in ["ledger_state", "identity_state", "drift_state", "outcome_models"] {
            sort_association(projected.get_mut(field).ok_or("projected association is absent")?)?;
        }
        let owner = projected.get_mut("owner_state").ok_or("owner state is absent")?;
        sort_association(&mut owner["identity_trackers"])?;
        // Identity snapshots publish sets and keyed cell outcomes; neither vector's iteration order is part of the persistence contract.
        if let Some(identities) = projected.get_mut("identity_state").and_then(Value::as_array_mut) {
            for identity in identities {
                sort_association(&mut identity[1]["cell_outcome_state"])?;
                if let Some(cells) = identity[1]["competitive_cells"].as_array_mut() {
                    cells.sort_by_cached_key(Value::to_string);
                }
            }
        }
        // Sequence marks remain exact: both arms start at the same durable mark and apply the same journalled suffix. Resetting a mark would hide lost or duplicated labels.
        Ok(Self(Value::Object(projected)))
    }

    pub(crate) fn compare(&self, restored: &Self) -> Result<(), String> {
        first_difference("", &self.0, &restored.0).map_or(Ok(()), Err)
    }
}

fn sort_association(value: &mut Value) -> Result<(), Box<dyn std::error::Error>> {
    let rows = value.as_array_mut().ok_or("durable association is not a sequence")?;
    rows.sort_by_cached_key(|row| row[0].to_string());
    Ok(())
}

fn first_difference(path: &str, left: &Value, right: &Value) -> Option<String> {
    if left == right {
        return None;
    }
    match (left, right) {
        (Value::Object(left), Value::Object(right)) if left.keys().eq(right.keys()) => left.iter().find_map(|(field, value)| {
            let path = if path.is_empty() {
                field.clone()
            } else {
                format!("{path}.{field}")
            };
            first_difference(&path, value, &right[field])
        }),
        (Value::Array(left), Value::Array(right)) if left.len() == right.len() => left
            .iter()
            .zip(right)
            .enumerate()
            .find_map(|(index, (left, right))| first_difference(&format!("{path}[{index}]"), left, right)),
        (Value::Number(left), Value::Number(right)) if left.is_f64() && right.is_f64() => {
            let left = left.as_f64()?;
            let right = right.as_f64()?;
            // Restore applies elapsed decay before the label; the uninterrupted arm multiplies the factors together. Permit only rounding from those arithmetic orderings, never a second decay factor.
            let tolerance = 64.0 * f64::EPSILON * left.abs().max(right.abs()).max(1.0);
            if (left - right).abs() <= tolerance {
                None
            } else {
                Some(format!("{path}: uninterrupted={left:?}, restored={right:?}"))
            }
        }
        _ => Some(format!("{path}: uninterrupted={left}, restored={right}")),
    }
}
