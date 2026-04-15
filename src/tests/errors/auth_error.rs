//! `AuthError` status-code, display, and `From` impl tests (§T-006 §1–3).

use hyper::StatusCode;

use crate::databases::database;
use crate::errors::AuthError;

// ── §1 Status-code mapping ──────────────────────────────────────────

#[test]
fn wrong_password_or_username_returns_forbidden() {
    assert_eq!(AuthError::WrongPasswordOrUsername.status_code(), StatusCode::FORBIDDEN);
}

#[test]
fn invalid_password_returns_forbidden() {
    assert_eq!(AuthError::InvalidPassword.status_code(), StatusCode::FORBIDDEN);
}

#[test]
fn username_not_found_returns_not_found() {
    assert_eq!(AuthError::UsernameNotFound.status_code(), StatusCode::NOT_FOUND);
}

#[test]
fn token_not_found_returns_unauthorized() {
    assert_eq!(AuthError::TokenNotFound.status_code(), StatusCode::UNAUTHORIZED);
}

#[test]
fn token_expired_returns_unauthorized() {
    assert_eq!(AuthError::TokenExpired.status_code(), StatusCode::UNAUTHORIZED);
}

#[test]
fn token_invalid_returns_unauthorized() {
    assert_eq!(AuthError::TokenInvalid.status_code(), StatusCode::UNAUTHORIZED);
}

#[test]
fn unauthorized_action_returns_forbidden() {
    assert_eq!(AuthError::UnauthorizedAction.status_code(), StatusCode::FORBIDDEN);
}

#[test]
fn unauthorized_action_for_guests_returns_unauthorized() {
    assert_eq!(AuthError::UnauthorizedActionForGuests.status_code(), StatusCode::UNAUTHORIZED);
}

#[test]
fn logged_in_user_not_found_returns_unauthorized() {
    assert_eq!(AuthError::LoggedInUserNotFound.status_code(), StatusCode::UNAUTHORIZED);
}

#[test]
fn email_not_verified_returns_forbidden() {
    assert_eq!(AuthError::EmailNotVerified.status_code(), StatusCode::FORBIDDEN);
}

#[test]
fn internal_server_error_returns_500() {
    assert_eq!(
        AuthError::InternalServerError.status_code(),
        StatusCode::INTERNAL_SERVER_ERROR
    );
}

#[test]
fn database_error_returns_500() {
    assert_eq!(AuthError::DatabaseError.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[test]
fn user_not_found_returns_not_found() {
    assert_eq!(AuthError::UserNotFound.status_code(), StatusCode::NOT_FOUND);
}

// ── §2 Display messages ─────────────────────────────────────────────

#[test]
fn display_wrong_password_or_username() {
    assert_eq!(
        AuthError::WrongPasswordOrUsername.to_string(),
        "Invalid username/email or password"
    );
}

#[test]
fn display_invalid_password() {
    assert_eq!(AuthError::InvalidPassword.to_string(), "Invalid password");
}

#[test]
fn display_username_not_found() {
    assert_eq!(AuthError::UsernameNotFound.to_string(), "Username not found");
}

#[test]
fn display_token_not_found() {
    assert_eq!(AuthError::TokenNotFound.to_string(), "Token not found. Please sign in.");
}

#[test]
fn display_token_expired() {
    assert_eq!(AuthError::TokenExpired.to_string(), "Token expired. Please sign in again.");
}

#[test]
fn display_token_invalid() {
    assert_eq!(AuthError::TokenInvalid.to_string(), "Token invalid.");
}

#[test]
fn display_unauthorized_action() {
    assert_eq!(AuthError::UnauthorizedAction.to_string(), "Unauthorized action.");
}

#[test]
fn display_unauthorized_action_for_guests() {
    assert_eq!(
        AuthError::UnauthorizedActionForGuests.to_string(),
        "Unauthorized actions for guest users. Try logging in to check if you have permission to perform the action"
    );
}

#[test]
fn display_logged_in_user_not_found() {
    assert_eq!(
        AuthError::LoggedInUserNotFound.to_string(),
        "Authentication error, please sign in"
    );
}

#[test]
fn display_email_not_verified() {
    assert_eq!(
        AuthError::EmailNotVerified.to_string(),
        "Please verify your email before logging in"
    );
}

#[test]
fn display_internal_server_error() {
    assert_eq!(AuthError::InternalServerError.to_string(), "internal server error");
}

#[test]
fn display_database_error() {
    assert_eq!(AuthError::DatabaseError.to_string(), "Database error.");
}

#[test]
fn display_user_not_found() {
    assert_eq!(AuthError::UserNotFound.to_string(), "User not found");
}

// ── §3 From impl coverage ──────────────────────────────────────────

#[test]
fn from_database_user_not_found() {
    let err: AuthError = database::Error::UserNotFound.into();
    assert_eq!(err, AuthError::UserNotFound);
}

#[test]
fn from_database_fallback() {
    let err: AuthError = database::Error::CategoryNotFound.into();
    assert_eq!(err, AuthError::DatabaseError);
}

#[test]
fn from_argon2_error() {
    let err: AuthError = argon2::password_hash::Error::Password.into();
    assert_eq!(err, AuthError::InternalServerError);
}
