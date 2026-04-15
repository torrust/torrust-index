//! `ApiError` delegation tests (§T-006 §4).
//!
//! Verifies that `ApiError` transparently delegates `status_code()` and
//! `Display` to the wrapped domain error.

use hyper::StatusCode;

use crate::errors::{ApiError, AuthError, CategoryTagError, TorrentError, UserError};

// ── status_code delegation ──────────────────────────────────────────

#[test]
fn delegates_status_code_to_auth() {
    let inner = AuthError::TokenNotFound;
    let api = ApiError::from(inner);
    assert_eq!(api.status_code(), StatusCode::UNAUTHORIZED);
}

#[test]
fn delegates_status_code_to_user() {
    let inner = UserError::PasswordTooShort;
    let api = ApiError::from(inner);
    assert_eq!(api.status_code(), StatusCode::BAD_REQUEST);
}

#[test]
fn delegates_status_code_to_torrent() {
    let inner = TorrentError::TorrentNotFound;
    let api = ApiError::from(inner);
    assert_eq!(api.status_code(), StatusCode::NOT_FOUND);
}

#[test]
fn delegates_status_code_to_category_tag() {
    let inner = CategoryTagError::CategoryNotFound;
    let api = ApiError::from(inner);
    assert_eq!(api.status_code(), StatusCode::NOT_FOUND);
}

// ── Display delegation ──────────────────────────────────────────────

#[test]
fn delegates_display_to_auth() {
    let inner = AuthError::TokenExpired;
    let api = ApiError::from(inner);
    assert_eq!(api.to_string(), "Token expired. Please sign in again.");
}

#[test]
fn delegates_display_to_user() {
    let inner = UserError::EmailMissing;
    let api = ApiError::from(inner);
    assert_eq!(api.to_string(), "Email is required");
}

#[test]
fn delegates_display_to_torrent() {
    let inner = TorrentError::InvalidFileType;
    let api = ApiError::from(inner);
    assert_eq!(api.to_string(), "Only .torrent files can be uploaded.");
}

#[test]
fn delegates_display_to_category_tag() {
    let inner = CategoryTagError::TagNameEmpty;
    let api = ApiError::from(inner);
    assert_eq!(api.to_string(), "Tag name cannot be empty.");
}
