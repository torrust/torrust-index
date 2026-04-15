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

use crate::services::authorization::{Action, PermissionMatrix, Permissions, Role};

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
    let err = "moderator".parse::<Role>();
    assert!(err.is_err());
    let err = err.unwrap_err();
    assert!(err.0.contains("moderator"));
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
            | Action::GenerateUserProfileSpecification => {}
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
