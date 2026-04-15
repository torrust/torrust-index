//! Authorization module.
//!
//! Implements ADR-T-008: native Rust permission system.
//!
//! # Architecture
//!
//! - [`Role`] — the user's privilege level (`Guest`, `Registered`, `Admin`).
//! - [`Action`] — an operation the user wants to perform.
//! - [`PermissionMatrix`] — the default-deny policy table.
//! - [`Permissions`] trait — abstraction consumed by the
//!   `RequirePermission` extractor (see `extractors::require_permission`).

use std::collections::HashSet;
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
    Admin,
}

impl Role {
    /// All variants (compile-time safe — see tests).
    pub const ALL: &[Self] = &[Self::Guest, Self::Registered, Self::Admin];
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Guest => "guest",
            Self::Registered => "registered",
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
/// Adding a variant without updating `PermissionMatrix::default_grant`
/// is a compile error (the exhaustive `match` has no wildcard).
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

// ── Permissions trait ────────────────────────────────────────────────

/// Abstraction over any permission-checking backend.
pub trait Permissions: Send + Sync {
    /// Returns `true` if `role` is allowed to perform `action`.
    fn can(&self, role: &Role, action: Action) -> bool;

    /// Returns `true` if `role` is allowed to perform `action` on a
    /// resource, taking ownership into account.
    ///
    /// - `Admin` bypasses the ownership requirement (if the base
    ///   matrix grants the action).
    /// - Other roles must be the resource owner.
    fn can_on_resource(&self, role: &Role, action: Action, is_owner: bool) -> bool {
        if !self.can(role, action) {
            return false;
        }
        matches!(role, Role::Admin) || is_owner
    }

    /// Returns the list of actions allowed for `role`.
    fn allowed_actions(&self, role: &Role) -> Vec<Action> {
        Action::ALL.iter().copied().filter(|&a| self.can(role, a)).collect()
    }
}

// ── PermissionMatrix ─────────────────────────────────────────────────

/// A set-based permission matrix: `(Role, Action)` pairs that are allowed.
///
/// Default-deny: any pair not in the set is denied.
pub struct PermissionMatrix {
    allowed: HashSet<(Role, Action)>,
}

impl PermissionMatrix {
    /// Build the default permission matrix.
    ///
    /// Iterates every `(Role, Action)` pair and delegates to
    /// `default_grant`, which uses exhaustive matches (no
    /// wildcards for `Registered` and `Guest`) — adding a new `Action`
    /// variant without deciding its permission is a compile error.
    #[must_use]
    pub fn default_matrix() -> Self {
        let mut allowed = HashSet::new();

        for &role in Role::ALL {
            for &action in Action::ALL {
                if Self::default_grant(role, action) {
                    allowed.insert((role, action));
                }
            }
        }

        Self { allowed }
    }

    /// Default grant decision for a single `(role, action)` pair.
    ///
    /// # Compile-time guarantees
    ///
    /// - Adding a `Role` variant → the top-level `match role` becomes
    ///   non-exhaustive → compile error.
    /// - Adding an `Action` variant → the `match action` arms for
    ///   `Registered` and `Guest` become non-exhaustive → compile error.
    /// - `Admin` intentionally grants all actions by default.
    const fn default_grant(role: Role, action: Action) -> bool {
        match role {
            // Admin is granted every action.
            Role::Admin => true,

            Role::Registered => match action {
                Action::GetAboutPage
                | Action::GetLicensePage
                | Action::GetCategories
                | Action::GetImageByUrl
                | Action::GetPublicSettings
                | Action::GetSiteName
                | Action::GetTags
                | Action::AddTorrent
                | Action::GetTorrent
                | Action::GetTorrentInfo
                | Action::GenerateTorrentInfoListing
                | Action::ChangePassword
                | Action::UpdateTorrent
                | Action::GetMyPermissions => true,

                Action::AddCategory
                | Action::DeleteCategory
                | Action::GetSettingsSecret
                | Action::AddTag
                | Action::DeleteTag
                | Action::DeleteTorrent
                | Action::BanUser
                | Action::GenerateUserProfileSpecification => false,
            },

            Role::Guest => match action {
                Action::GetAboutPage
                | Action::GetLicensePage
                | Action::GetCategories
                | Action::GetPublicSettings
                | Action::GetSiteName
                | Action::GetTags
                | Action::GetTorrent
                | Action::GetTorrentInfo
                | Action::GenerateTorrentInfoListing
                | Action::GetMyPermissions => true,

                Action::AddCategory
                | Action::DeleteCategory
                | Action::GetImageByUrl
                | Action::GetSettingsSecret
                | Action::AddTag
                | Action::DeleteTag
                | Action::AddTorrent
                | Action::DeleteTorrent
                | Action::ChangePassword
                | Action::BanUser
                | Action::GenerateUserProfileSpecification
                | Action::UpdateTorrent => false,
            },
        }
    }

    /// Build a matrix from the defaults, then apply TOML overrides.
    ///
    /// Each override inserts (allow) or removes (deny) a `(Role, Action)` pair.
    #[must_use]
    pub fn with_overrides(overrides: &[PermissionOverride]) -> Self {
        let mut matrix = Self::default_matrix();
        for ov in overrides {
            match ov.effect {
                Effect::Allow => {
                    matrix.allowed.insert((ov.role, ov.action));
                }
                Effect::Deny => {
                    matrix.allowed.remove(&(ov.role, ov.action));
                }
            }
        }
        matrix
    }
}

impl Permissions for PermissionMatrix {
    fn can(&self, role: &Role, action: Action) -> bool {
        self.allowed.contains(&(*role, action))
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
