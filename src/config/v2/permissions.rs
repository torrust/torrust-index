use serde::{Deserialize, Serialize};

use crate::services::authorization::PermissionOverride;

/// Operator-supplied permission overrides.
///
/// Loaded from the `[permissions]` section of the TOML config.
/// Each entry in `overrides` patches the default permission matrix
/// at startup.
///
/// ## Example
///
/// ```toml
/// [[permissions.overrides]]
/// role = "registered"
/// action = "DeleteTorrent"
/// effect = "allow"
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Permissions {
    /// List of `(role, action, effect)` tuples that override the
    /// built-in permission matrix.
    #[serde(default)]
    pub overrides: Vec<PermissionOverride>,
}
