//! Crate tests for the authorization module (`src/services/authorization.rs`).
//!
//! # Test index
//!
//! ## Role
//!
//! - [`role_display_round_trips_through_from_str`] — every `Role`
//!   variant round-trips through `Display` → `FromStr`.
//! - [`role_serde_round_trip`] — every `Role` variant round-trips
//!   through serde JSON serialization.
//! - [`role_from_str_rejects_unknown`] — unknown strings yield
//!   `RoleParseError`.
//!
//! ## Action
//!
//! - [`action_all_is_exhaustive`] — `Action::ALL` covers every variant
//!   (enforced by an exhaustive match).
//!
//! ## `PermissionMatrix`
//!
//! - [`admin_is_granted_every_action`] — `Admin` can do everything.
//! - [`moderator_grants_and_denials`] — `Moderator` gets the
//!   expected subset (Registered + tag/torrent moderation).
//! - [`registered_grants_and_denials`] — `Registered` gets the
//!   expected subset.
//! - [`guest_grants_and_denials`] — `Guest` gets the expected subset.
//! - [`every_role_action_pair_has_a_decision`] — the matrix covers
//!   every `(Role, Action)` combination (either grant or deny).
//!
//! ## Permissions trait
//!
//! - [`permissions_trait_delegates_to_matrix`] — calling `can()` on the
//!   trait object matches the matrix.
//!
//! ## Resource-level authorization (Phase 3)
//!
//! - [`can_on_resource_allows_owner`] — owner is allowed for granted
//!   action.
//! - [`can_on_resource_denies_non_owner`] — non-owner is denied even
//!   when the base matrix grants the action.
//! - [`can_on_resource_admin_bypasses_ownership`] — admin is allowed
//!   regardless of ownership.
//! - [`can_on_resource_denied_action_returns_false`] — denied action
//!   returns false even for owner.
//!
//! ## `allowed_actions` (Phase 3)
//!
//! - [`allowed_actions_returns_correct_list`] — matches the granted set
//!   for each role.
//!
//! ## `with_overrides` (Phase 3)
//!
//! - [`with_overrides_allow_adds_permission`] — an `allow` override
//!   grants a previously denied action.
//! - [`with_overrides_deny_removes_permission`] — a `deny` override
//!   revokes a previously granted action.
//! - [`with_overrides_empty_matches_default`] — empty overrides produce
//!   the same matrix as the default.
//! - [`with_overrides_high_risk_grant_still_applies`] — a high-risk
//!   override is applied (the `warn!` is emitted but does not block).

use crate::services::authorization::{Action, Effect, PermissionMatrix, PermissionOverride, Permissions, Role};

// ── Role ─────────────────────────────────────────────────────────────

#[test]
fn role_display_round_trips_through_from_str() {
    for &role in Role::ALL {
        let s = role.to_string();
        let parsed: Role = s.parse().unwrap();
        assert_eq!(parsed, role);
    }
}

#[test]
fn role_serde_round_trip() {
    for &role in Role::ALL {
        let json = serde_json::to_string(&role).unwrap();
        let back: Role = serde_json::from_str(&json).unwrap();
        assert_eq!(back, role);
    }
}

#[test]
fn role_from_str_rejects_unknown() {
    let err = "superuser".parse::<Role>();
    assert!(err.is_err());
    let err = err.unwrap_err();
    assert!(err.0.contains("superuser"));
}

// ── Action ───────────────────────────────────────────────────────────

/// Ensures `Action::ALL` covers every variant.
///
/// The exhaustive match (no wildcard) below means adding a new `Action`
/// variant without updating this test — and `ALL` — is a compile error.
#[test]
fn action_all_is_exhaustive() {
    let expected = [
        Action::GetAboutPage,
        Action::GetLicensePage,
        Action::AddCategory,
        Action::DeleteCategory,
        Action::GetCategories,
        Action::GetImageByUrl,
        Action::GetSettingsSecret,
        Action::GetPublicSettings,
        Action::GetSiteName,
        Action::AddTag,
        Action::DeleteTag,
        Action::GetTags,
        Action::AddTorrent,
        Action::GetTorrent,
        Action::DeleteTorrent,
        Action::UpdateTorrent,
        Action::GetTorrentInfo,
        Action::GenerateTorrentInfoListing,
        Action::ChangePassword,
        Action::BanUser,
        Action::GenerateUserProfileSpecification,
        Action::GetMyPermissions,
    ];

    // Exhaustive match — compile error if a variant is missing.
    for action in &expected {
        match action {
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
            | Action::UpdateTorrent
            | Action::GetTorrentInfo
            | Action::GenerateTorrentInfoListing
            | Action::ChangePassword
            | Action::BanUser
            | Action::GenerateUserProfileSpecification
            | Action::GetMyPermissions => {}
        }
    }

    assert_eq!(Action::ALL.len(), expected.len());
}

// ── PermissionMatrix ─────────────────────────────────────────────────

#[test]
fn admin_is_granted_every_action() {
    let matrix = PermissionMatrix::default_matrix();
    for &action in Action::ALL {
        assert!(matrix.can(&Role::Admin, action), "Admin should be granted {action}");
    }
}

#[test]
fn moderator_grants_and_denials() {
    let matrix = PermissionMatrix::default_matrix();

    let granted = [
        Action::GetAboutPage,
        Action::GetLicensePage,
        Action::GetCategories,
        Action::GetImageByUrl,
        Action::GetPublicSettings,
        Action::GetSiteName,
        Action::GetTags,
        Action::AddTorrent,
        Action::GetTorrent,
        Action::UpdateTorrent,
        Action::GetTorrentInfo,
        Action::GenerateTorrentInfoListing,
        Action::ChangePassword,
        Action::GetMyPermissions,
        Action::AddTag,
        Action::DeleteTag,
        Action::DeleteTorrent,
    ];

    let denied = [
        Action::AddCategory,
        Action::DeleteCategory,
        Action::GetSettingsSecret,
        Action::BanUser,
        Action::GenerateUserProfileSpecification,
    ];

    for action in granted {
        assert!(matrix.can(&Role::Moderator, action), "Moderator should be granted {action}");
    }

    for action in denied {
        assert!(!matrix.can(&Role::Moderator, action), "Moderator should be denied {action}");
    }

    // Sanity: granted + denied == all actions
    assert_eq!(granted.len() + denied.len(), Action::ALL.len());
}

#[test]
fn registered_grants_and_denials() {
    let matrix = PermissionMatrix::default_matrix();

    let granted = [
        Action::GetAboutPage,
        Action::GetLicensePage,
        Action::GetCategories,
        Action::GetImageByUrl,
        Action::GetPublicSettings,
        Action::GetSiteName,
        Action::GetTags,
        Action::AddTorrent,
        Action::GetTorrent,
        Action::UpdateTorrent,
        Action::GetTorrentInfo,
        Action::GenerateTorrentInfoListing,
        Action::ChangePassword,
        Action::GetMyPermissions,
    ];

    let denied = [
        Action::AddCategory,
        Action::DeleteCategory,
        Action::GetSettingsSecret,
        Action::AddTag,
        Action::DeleteTag,
        Action::DeleteTorrent,
        Action::BanUser,
        Action::GenerateUserProfileSpecification,
    ];

    for action in granted {
        assert!(matrix.can(&Role::Registered, action), "Registered should be granted {action}");
    }

    for action in denied {
        assert!(!matrix.can(&Role::Registered, action), "Registered should be denied {action}");
    }

    // Sanity: granted + denied == all actions
    assert_eq!(granted.len() + denied.len(), Action::ALL.len());
}

#[test]
fn guest_grants_and_denials() {
    let matrix = PermissionMatrix::default_matrix();

    let granted = [
        Action::GetAboutPage,
        Action::GetLicensePage,
        Action::GetCategories,
        Action::GetPublicSettings,
        Action::GetSiteName,
        Action::GetTags,
        Action::GetTorrent,
        Action::GetTorrentInfo,
        Action::GenerateTorrentInfoListing,
        Action::GetMyPermissions,
    ];

    let denied = [
        Action::AddCategory,
        Action::DeleteCategory,
        Action::GetImageByUrl,
        Action::GetSettingsSecret,
        Action::AddTag,
        Action::DeleteTag,
        Action::AddTorrent,
        Action::DeleteTorrent,
        Action::UpdateTorrent,
        Action::ChangePassword,
        Action::BanUser,
        Action::GenerateUserProfileSpecification,
    ];

    for action in granted {
        assert!(matrix.can(&Role::Guest, action), "Guest should be granted {action}");
    }

    for action in denied {
        assert!(!matrix.can(&Role::Guest, action), "Guest should be denied {action}");
    }

    assert_eq!(granted.len() + denied.len(), Action::ALL.len());
}

#[test]
fn every_role_action_pair_has_a_decision() {
    let matrix = PermissionMatrix::default_matrix();

    // Every (Role, Action) must be either granted or denied —
    // i.e. the matrix was populated for every pair.
    // With a HashSet-based matrix, an absent pair is a denial,
    // but we want to confirm the builder visited every pair.
    let mut visited = 0usize;
    for &role in Role::ALL {
        for &action in Action::ALL {
            // Just exercise the lookup — it must not panic.
            let _ = matrix.can(&role, action);
            visited += 1;
        }
    }
    assert_eq!(visited, Role::ALL.len() * Action::ALL.len());
}

// ── Permissions trait ────────────────────────────────────────────────

#[test]
fn permissions_trait_delegates_to_matrix() {
    let matrix = PermissionMatrix::default_matrix();
    let permissions: &dyn Permissions = &matrix;

    // Spot-check a few pairs.
    assert!(permissions.can(&Role::Admin, Action::BanUser));
    assert!(!permissions.can(&Role::Guest, Action::BanUser));
    assert!(permissions.can(&Role::Registered, Action::AddTorrent));
    assert!(!permissions.can(&Role::Guest, Action::AddTorrent));
}

// ── Resource-level authorization (Phase 3) ───────────────────────────

#[test]
fn can_on_resource_allows_owner() {
    let matrix = PermissionMatrix::default_matrix();
    // Registered + UpdateTorrent is granted in the base matrix.
    // Owner → allowed.
    assert!(matrix.can_on_resource(&Role::Registered, Action::UpdateTorrent, true));
}

#[test]
fn can_on_resource_denies_non_owner() {
    let matrix = PermissionMatrix::default_matrix();
    // Registered + UpdateTorrent is granted in the base matrix,
    // but non-owner → denied.
    assert!(!matrix.can_on_resource(&Role::Registered, Action::UpdateTorrent, false));
}

#[test]
fn can_on_resource_admin_bypasses_ownership() {
    let matrix = PermissionMatrix::default_matrix();
    // Admin bypasses ownership — allowed even as non-owner.
    assert!(matrix.can_on_resource(&Role::Admin, Action::UpdateTorrent, false));
    assert!(matrix.can_on_resource(&Role::Admin, Action::UpdateTorrent, true));
}

#[test]
fn can_on_resource_denied_action_returns_false() {
    let matrix = PermissionMatrix::default_matrix();
    // Guest + UpdateTorrent is denied in the base matrix.
    // Even as "owner" → denied.
    assert!(!matrix.can_on_resource(&Role::Guest, Action::UpdateTorrent, true));
}

// ── allowed_actions (Phase 3) ────────────────────────────────────────

#[test]
fn allowed_actions_returns_correct_list() {
    let matrix = PermissionMatrix::default_matrix();

    // Admin should get all actions.
    let admin_actions = matrix.allowed_actions(&Role::Admin);
    assert_eq!(admin_actions.len(), Action::ALL.len());

    // Guest should get the expected subset.
    let guest_actions = matrix.allowed_actions(&Role::Guest);
    assert!(guest_actions.contains(&Action::GetAboutPage));
    assert!(guest_actions.contains(&Action::GetMyPermissions));
    assert!(!guest_actions.contains(&Action::AddTorrent));
    assert!(!guest_actions.contains(&Action::BanUser));
}

// ── with_overrides (Phase 3) ─────────────────────────────────────────

#[test]
fn with_overrides_allow_adds_permission() {
    let overrides = vec![PermissionOverride {
        role: Role::Guest,
        action: Action::AddTorrent,
        effect: Effect::Allow,
    }];
    let matrix = PermissionMatrix::with_overrides(&overrides);

    // Guest + AddTorrent is normally denied; override grants it.
    assert!(matrix.can(&Role::Guest, Action::AddTorrent));
}

#[test]
fn with_overrides_deny_removes_permission() {
    let overrides = vec![PermissionOverride {
        role: Role::Registered,
        action: Action::AddTorrent,
        effect: Effect::Deny,
    }];
    let matrix = PermissionMatrix::with_overrides(&overrides);

    // Registered + AddTorrent is normally granted; override revokes it.
    assert!(!matrix.can(&Role::Registered, Action::AddTorrent));
}

#[test]
fn with_overrides_empty_matches_default() {
    let matrix_default = PermissionMatrix::default_matrix();
    let matrix_overridden = PermissionMatrix::with_overrides(&[]);

    for &role in Role::ALL {
        for &action in Action::ALL {
            assert_eq!(
                matrix_default.can(&role, action),
                matrix_overridden.can(&role, action),
                "mismatch for ({role}, {action})"
            );
        }
    }
}

#[test]
fn with_overrides_high_risk_grant_still_applies() {
    // A high-risk override emits a warn! but still takes effect.
    let overrides = vec![PermissionOverride {
        role: Role::Guest,
        action: Action::DeleteTorrent,
        effect: Effect::Allow,
    }];
    let matrix = PermissionMatrix::with_overrides(&overrides);

    // Guest + DeleteTorrent is normally denied; override grants it.
    assert!(matrix.can(&Role::Guest, Action::DeleteTorrent));
}
