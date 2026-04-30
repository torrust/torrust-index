//! Permission-related value types shared between the
//! configuration schema and the authorization service.
//!
//! Per ADR-T-009 phase 3 these *value* types live in the config
//! crate so that both `[[permissions.overrides]]` deserialisation
//! and the runtime authorization matrix can reference them
//! without the config crate depending on the service layer.
//!
//! The `PermissionMatrix` and `Permissions` *trait* (the runtime
//! policy) remain in the root crate's `services::authorization`
//! module, which re-exports the types defined here for backwards
//! compatibility with existing call sites.
use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

// ── Role ─────────────────────────────────────────────────────────────

/// User privilege level.
///
/// Stored as a lowercase string in the `torrust_users.role` column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Guest,
    Registered,
    Moderator,
    Admin,
}

impl Role {
    /// All variants (compile-time safe — see tests).
    pub const ALL: &[Self] = &[Self::Guest, Self::Registered, Self::Moderator, Self::Admin];
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Guest => "guest",
            Self::Registered => "registered",
            Self::Moderator => "moderator",
            Self::Admin => "admin",
        };
        write!(f, "{s}")
    }
}

impl FromStr for Role {
    type Err = RoleParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "guest" => Ok(Self::Guest),
            "registered" => Ok(Self::Registered),
            "moderator" => Ok(Self::Moderator),
            "admin" => Ok(Self::Admin),
            _ => Err(RoleParseError(s.to_owned())),
        }
    }
}

/// Error returned when a string cannot be parsed into a [`Role`].
#[derive(Debug, Clone)]
pub struct RoleParseError(pub String);

impl fmt::Display for RoleParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown role: {:?}", self.0)
    }
}

impl std::error::Error for RoleParseError {}

// ── Action ───────────────────────────────────────────────────────────

/// An operation that may be authorized.
///
/// Adding a variant without updating the root-crate `PermissionMatrix`'s
/// `default_grant` method is a compile error (the exhaustive `match` has
/// no wildcard).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    GetAboutPage,
    GetLicensePage,
    AddCategory,
    DeleteCategory,
    GetCategories,
    GetImageByUrl,
    GetSettingsSecret,
    GetPublicSettings,
    GetSiteName,
    AddTag,
    DeleteTag,
    GetTags,
    AddTorrent,
    GetTorrent,
    DeleteTorrent,
    GetTorrentInfo,
    GenerateTorrentInfoListing,
    ChangePassword,
    BanUser,
    /// Render a user profile as a PNG image (admin-only).
    GenerateUserProfileSpecification,
    UpdateTorrent,
    GetMyPermissions,
}

impl Action {
    /// Every variant in declaration order.
    pub const ALL: &[Self] = &[
        Self::GetAboutPage,
        Self::GetLicensePage,
        Self::AddCategory,
        Self::DeleteCategory,
        Self::GetCategories,
        Self::GetImageByUrl,
        Self::GetSettingsSecret,
        Self::GetPublicSettings,
        Self::GetSiteName,
        Self::AddTag,
        Self::DeleteTag,
        Self::GetTags,
        Self::AddTorrent,
        Self::GetTorrent,
        Self::DeleteTorrent,
        Self::GetTorrentInfo,
        Self::GenerateTorrentInfoListing,
        Self::ChangePassword,
        Self::BanUser,
        Self::GenerateUserProfileSpecification,
        Self::UpdateTorrent,
        Self::GetMyPermissions,
    ];
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

// ── PermissionOverride ───────────────────────────────────────────────

/// Effect of a permission override.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Effect {
    Allow,
    Deny,
}

/// A single operator-supplied permission override.
///
/// Loaded from the `[[permissions.overrides]]` TOML array and applied
/// on top of the default matrix at startup.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermissionOverride {
    pub role: Role,
    pub action: Action,
    pub effect: Effect,
}
