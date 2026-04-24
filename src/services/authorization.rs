//! Authorization module.
//!
//! Implements ADR-T-008: native Rust permission system.
//!
//! # Architecture
//!
//! - [`Role`] — the user's privilege level (`Guest`, `Registered`,
//!   `Moderator`, `Admin`).
//! - [`Action`] — an operation the user wants to perform.
//! - [`PermissionMatrix`] — the default-deny policy table.
//! - [`Permissions`] trait — abstraction consumed by the
//!   `RequirePermission` extractor (see `extractors::require_permission`).
//!
//! # Crate layout (ADR-T-009 phase 3)
//!
//! The *value* types ([`Role`], [`Action`], [`Effect`],
//! [`PermissionOverride`], [`RoleParseError`]) live in the
//! `torrust-index-config` crate so the configuration schema can
//! reference them without depending on the service layer. They are
//! re-exported here for backwards compatibility with existing call
//! sites.
//!
//! # Compile-time guarantees
//!
//! - Adding a `Role` variant without updating `default_grant` is a
//!   compile error (exhaustive top-level `match`).
//! - Adding an `Action` variant without updating `default_grant` is a
//!   compile error (exhaustive inner `match` for every role).
//! - The `action_markers!` macro in `extractors::require_permission`
//!   emits a `const` assertion that its marker count equals
//!   `Action::ALL.len()`, catching marker/enum drift at compile time.
use std::collections::HashSet;

pub use torrust_index_config::permissions::{Action, Effect, PermissionOverride, Role, RoleParseError};
use tracing::warn;

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
            Role::Admin => match action {
                Action::GetAboutPage
                | Action::GetLicensePage
                | Action::AddCategory
                | Action::DeleteCategory
                | Action::GetCategories
                | Action::GetImageByUrl
                | Action::GetSettingsSecret
                | Action::GetPublicSettings
                | Action::GetSiteName
                | Action::AddTag
                | Action::DeleteTag
                | Action::GetTags
                | Action::AddTorrent
                | Action::GetTorrent
                | Action::DeleteTorrent
                | Action::GetTorrentInfo
                | Action::GenerateTorrentInfoListing
                | Action::ChangePassword
                | Action::BanUser
                | Action::GenerateUserProfileSpecification
                | Action::UpdateTorrent
                | Action::GetMyPermissions => true,
            },

            // Moderator: everything Registered has, plus tag and
            // torrent moderation actions.
            Role::Moderator => match action {
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
                | Action::GetMyPermissions
                | Action::AddTag
                | Action::DeleteTag
                | Action::DeleteTorrent => true,

                Action::AddCategory
                | Action::DeleteCategory
                | Action::GetSettingsSecret
                | Action::BanUser
                | Action::GenerateUserProfileSpecification => false,
            },

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

    /// Actions considered high-risk when granted to `Guest` or
    /// `Registered` via a TOML override.  A `warn!` is emitted at
    /// startup for each such grant so operators are alerted.
    const HIGH_RISK_ACTIONS: &[Action] = &[
        Action::DeleteTorrent,
        Action::DeleteCategory,
        Action::DeleteTag,
        Action::BanUser,
        Action::GetSettingsSecret,
        Action::GenerateUserProfileSpecification,
    ];

    /// Build a matrix from the defaults, then apply TOML overrides.
    ///
    /// Each override inserts (allow) or removes (deny) a `(Role, Action)` pair.
    /// A `warn!` is emitted for overrides that grant high-risk actions to
    /// non-admin roles.
    #[must_use]
    pub fn with_overrides(overrides: &[PermissionOverride]) -> Self {
        let mut matrix = Self::default_matrix();
        for ov in overrides {
            match ov.effect {
                Effect::Allow => {
                    if !matches!(ov.role, Role::Admin) && Self::HIGH_RISK_ACTIONS.contains(&ov.action) {
                        warn!(
                            role = %ov.role,
                            action = %ov.action,
                            "high-risk permission override: granting a destructive action to a non-admin role",
                        );
                    }
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
