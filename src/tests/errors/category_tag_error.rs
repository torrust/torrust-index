//! `CategoryTagError` status-code, display, and `From` impl tests (§T-006 §1–3).

use hyper::StatusCode;

use crate::errors::{AuthError, CategoryTagError};

// ── §1 Status-code mapping ──────────────────────────────────────────

#[test]
fn invalid_category_returns_bad_request() {
    assert_eq!(CategoryTagError::InvalidCategory.status_code(), StatusCode::BAD_REQUEST);
}

#[test]
fn category_already_exists_returns_bad_request() {
    assert_eq!(CategoryTagError::CategoryAlreadyExists.status_code(), StatusCode::BAD_REQUEST);
}

#[test]
fn category_name_empty_returns_bad_request() {
    assert_eq!(CategoryTagError::CategoryNameEmpty.status_code(), StatusCode::BAD_REQUEST);
}

#[test]
fn category_not_found_returns_not_found() {
    assert_eq!(CategoryTagError::CategoryNotFound.status_code(), StatusCode::NOT_FOUND);
}

#[test]
fn invalid_tag_returns_bad_request() {
    assert_eq!(CategoryTagError::InvalidTag.status_code(), StatusCode::BAD_REQUEST);
}

#[test]
fn tag_already_exists_returns_bad_request() {
    assert_eq!(CategoryTagError::TagAlreadyExists.status_code(), StatusCode::BAD_REQUEST);
}

#[test]
fn tag_name_empty_returns_bad_request() {
    assert_eq!(CategoryTagError::TagNameEmpty.status_code(), StatusCode::BAD_REQUEST);
}

#[test]
fn tag_not_found_returns_not_found() {
    assert_eq!(CategoryTagError::TagNotFound.status_code(), StatusCode::NOT_FOUND);
}

#[test]
fn unauthorized_action_returns_forbidden() {
    assert_eq!(CategoryTagError::UnauthorizedAction.status_code(), StatusCode::FORBIDDEN);
}

#[test]
fn unauthorized_action_for_guests_returns_unauthorized() {
    assert_eq!(
        CategoryTagError::UnauthorizedActionForGuests.status_code(),
        StatusCode::UNAUTHORIZED
    );
}

#[test]
fn database_error_returns_500() {
    assert_eq!(
        CategoryTagError::DatabaseError.status_code(),
        StatusCode::INTERNAL_SERVER_ERROR
    );
}

// ── §2 Display messages ─────────────────────────────────────────────

#[test]
fn display_invalid_category() {
    assert_eq!(
        CategoryTagError::InvalidCategory.to_string(),
        "Selected category does not exist."
    );
}

#[test]
fn display_category_already_exists() {
    assert_eq!(
        CategoryTagError::CategoryAlreadyExists.to_string(),
        "Category already exists."
    );
}

#[test]
fn display_category_name_empty() {
    assert_eq!(
        CategoryTagError::CategoryNameEmpty.to_string(),
        "Category name cannot be empty."
    );
}

#[test]
fn display_category_not_found() {
    assert_eq!(CategoryTagError::CategoryNotFound.to_string(), "Category not found.");
}

#[test]
fn display_invalid_tag() {
    assert_eq!(CategoryTagError::InvalidTag.to_string(), "Selected tag does not exist.");
}

#[test]
fn display_tag_already_exists() {
    assert_eq!(CategoryTagError::TagAlreadyExists.to_string(), "Tag already exists.");
}

#[test]
fn display_tag_name_empty() {
    assert_eq!(CategoryTagError::TagNameEmpty.to_string(), "Tag name cannot be empty.");
}

#[test]
fn display_tag_not_found() {
    assert_eq!(CategoryTagError::TagNotFound.to_string(), "Tag not found.");
}

#[test]
fn display_unauthorized_action() {
    assert_eq!(CategoryTagError::UnauthorizedAction.to_string(), "Unauthorized action.");
}

#[test]
fn display_unauthorized_action_for_guests() {
    assert_eq!(
        CategoryTagError::UnauthorizedActionForGuests.to_string(),
        "Unauthorized actions for guest users. Try logging in to check if you have permission to perform the action"
    );
}

#[test]
fn display_database_error() {
    assert_eq!(CategoryTagError::DatabaseError.to_string(), "Database error.");
}

// ── §3 From impl coverage ──────────────────────────────────────────

#[test]
fn from_auth_unauthorized_action() {
    let err: CategoryTagError = AuthError::UnauthorizedAction.into();
    assert_eq!(err, CategoryTagError::UnauthorizedAction);
}

#[test]
fn from_auth_unauthorized_action_for_guests() {
    let err: CategoryTagError = AuthError::UnauthorizedActionForGuests.into();
    assert_eq!(err, CategoryTagError::UnauthorizedActionForGuests);
}

#[test]
fn from_auth_fallback() {
    let err: CategoryTagError = AuthError::TokenExpired.into();
    assert_eq!(err, CategoryTagError::DatabaseError);
}
