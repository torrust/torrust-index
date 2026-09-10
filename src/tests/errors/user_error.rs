//! `UserError` status-code, display, and `From` impl tests (§T-006 §1–3).

use hyper::StatusCode;

use crate::databases::database;
use crate::errors::{AuthError, UserError};

// ── §1 Status-code mapping ──────────────────────────────────────────

#[test]
fn closed_for_registration_returns_forbidden() {
    assert_eq!(UserError::ClosedForRegistration.status_code(), StatusCode::FORBIDDEN);
}

#[test]
fn email_missing_returns_not_found() {
    assert_eq!(UserError::EmailMissing.status_code(), StatusCode::NOT_FOUND);
}

#[test]
fn email_invalid_returns_bad_request() {
    assert_eq!(UserError::EmailInvalid.status_code(), StatusCode::BAD_REQUEST);
}

#[test]
fn email_taken_returns_bad_request() {
    assert_eq!(UserError::EmailTaken.status_code(), StatusCode::BAD_REQUEST);
}

#[test]
fn email_not_verified_returns_forbidden() {
    assert_eq!(UserError::EmailNotVerified.status_code(), StatusCode::FORBIDDEN);
}

#[test]
fn failed_to_send_verification_email_returns_500() {
    assert_eq!(
        UserError::FailedToSendVerificationEmail.status_code(),
        StatusCode::INTERNAL_SERVER_ERROR
    );
}

#[test]
fn user_not_found_returns_not_found() {
    assert_eq!(UserError::UserNotFound.status_code(), StatusCode::NOT_FOUND);
}

#[test]
fn account_not_found_returns_not_found() {
    assert_eq!(UserError::AccountNotFound.status_code(), StatusCode::NOT_FOUND);
}

#[test]
fn username_taken_returns_bad_request() {
    assert_eq!(UserError::UsernameTaken.status_code(), StatusCode::BAD_REQUEST);
}

#[test]
fn username_invalid_returns_bad_request() {
    assert_eq!(UserError::UsernameInvalid.status_code(), StatusCode::BAD_REQUEST);
}

#[test]
fn profanity_error_returns_bad_request() {
    assert_eq!(UserError::ProfanityError.status_code(), StatusCode::BAD_REQUEST);
}

#[test]
fn blacklist_error_returns_bad_request() {
    assert_eq!(UserError::BlacklistError.status_code(), StatusCode::BAD_REQUEST);
}

#[test]
fn username_case_mapped_error_returns_bad_request() {
    assert_eq!(UserError::UsernameCaseMappedError.status_code(), StatusCode::BAD_REQUEST);
}

#[test]
fn password_too_short_returns_bad_request() {
    assert_eq!(UserError::PasswordTooShort.status_code(), StatusCode::BAD_REQUEST);
}

#[test]
fn password_too_long_returns_bad_request() {
    assert_eq!(UserError::PasswordTooLong.status_code(), StatusCode::BAD_REQUEST);
}

#[test]
fn passwords_dont_match_returns_bad_request() {
    assert_eq!(UserError::PasswordsDontMatch.status_code(), StatusCode::BAD_REQUEST);
}

#[test]
fn invalid_user_listing_returns_bad_request() {
    assert_eq!(UserError::InvalidUserListing.status_code(), StatusCode::BAD_REQUEST);
}

#[test]
fn invalid_password_returns_forbidden() {
    assert_eq!(UserError::InvalidPassword.status_code(), StatusCode::FORBIDDEN);
}

#[test]
fn unauthorized_action_returns_forbidden() {
    assert_eq!(UserError::UnauthorizedAction.status_code(), StatusCode::FORBIDDEN);
}

#[test]
fn unauthorized_action_for_guests_returns_unauthorized() {
    assert_eq!(UserError::UnauthorizedActionForGuests.status_code(), StatusCode::UNAUTHORIZED);
}

#[test]
fn database_error_returns_500() {
    assert_eq!(UserError::DatabaseError.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[test]
fn internal_server_error_returns_500() {
    assert_eq!(
        UserError::InternalServerError.status_code(),
        StatusCode::INTERNAL_SERVER_ERROR
    );
}

// ── §2 Display messages ─────────────────────────────────────────────

#[test]
fn display_closed_for_registration() {
    assert_eq!(
        UserError::ClosedForRegistration.to_string(),
        "This server is closed for registration. Contact admin if this is unexpected"
    );
}

#[test]
fn display_email_missing() {
    assert_eq!(UserError::EmailMissing.to_string(), "Email is required");
}

#[test]
fn display_email_invalid() {
    assert_eq!(UserError::EmailInvalid.to_string(), "Please enter a valid email address");
}

#[test]
fn display_email_taken() {
    assert_eq!(UserError::EmailTaken.to_string(), "Email not available");
}

#[test]
fn display_email_not_verified() {
    assert_eq!(
        UserError::EmailNotVerified.to_string(),
        "Please verify your email before logging in"
    );
}

#[test]
fn display_failed_to_send_verification_email() {
    assert_eq!(
        UserError::FailedToSendVerificationEmail.to_string(),
        "Failed to send verification email."
    );
}

#[test]
fn display_user_not_found() {
    assert_eq!(UserError::UserNotFound.to_string(), "User not found");
}

#[test]
fn display_account_not_found() {
    assert_eq!(UserError::AccountNotFound.to_string(), "Account not found");
}

#[test]
fn display_username_taken() {
    assert_eq!(UserError::UsernameTaken.to_string(), "Username not available");
}

#[test]
fn display_username_invalid() {
    assert_eq!(
        UserError::UsernameInvalid.to_string(),
        "Invalid username. Usernames must consist of 1-20 alphanumeric characters, dashes, or underscore"
    );
}

#[test]
fn display_profanity_error() {
    assert_eq!(UserError::ProfanityError.to_string(), "Can't allow profanity in usernames");
}

#[test]
fn display_blacklist_error() {
    assert_eq!(UserError::BlacklistError.to_string(), "Username contains blacklisted words");
}

#[test]
fn display_username_case_mapped_error() {
    assert_eq!(
        UserError::UsernameCaseMappedError.to_string(),
        "username_case_mapped violation"
    );
}

#[test]
fn display_password_too_short() {
    assert_eq!(UserError::PasswordTooShort.to_string(), "Password too short");
}

#[test]
fn display_password_too_long() {
    assert_eq!(UserError::PasswordTooLong.to_string(), "Password too long");
}

#[test]
fn display_passwords_dont_match() {
    assert_eq!(UserError::PasswordsDontMatch.to_string(), "Passwords don't match");
}

#[test]
fn display_invalid_user_listing() {
    assert_eq!(
        UserError::InvalidUserListing.to_string(),
        "Invalid user listing fields in the URL params."
    );
}

#[test]
fn display_invalid_password() {
    assert_eq!(UserError::InvalidPassword.to_string(), "Invalid password");
}

#[test]
fn display_unauthorized_action() {
    assert_eq!(UserError::UnauthorizedAction.to_string(), "Unauthorized action.");
}

#[test]
fn display_unauthorized_action_for_guests() {
    assert_eq!(
        UserError::UnauthorizedActionForGuests.to_string(),
        "Unauthorized actions for guest users. Try logging in to check if you have permission to perform the action"
    );
}

#[test]
fn display_database_error() {
    assert_eq!(UserError::DatabaseError.to_string(), "Database error.");
}

#[test]
fn display_internal_server_error() {
    assert_eq!(UserError::InternalServerError.to_string(), "internal server error");
}

// ── §3 From impl coverage ──────────────────────────────────────────

#[test]
fn from_auth_unauthorized_action() {
    let err: UserError = AuthError::UnauthorizedAction.into();
    assert_eq!(err, UserError::UnauthorizedAction);
}

#[test]
fn from_auth_unauthorized_action_for_guests() {
    let err: UserError = AuthError::UnauthorizedActionForGuests.into();
    assert_eq!(err, UserError::UnauthorizedActionForGuests);
}

#[test]
fn from_auth_invalid_password() {
    let err: UserError = AuthError::InvalidPassword.into();
    assert_eq!(err, UserError::InvalidPassword);
}

#[test]
fn from_auth_fallback() {
    let err: UserError = AuthError::TokenExpired.into();
    assert_eq!(err, UserError::InternalServerError);
}

#[test]
fn from_database_username_taken() {
    let err: UserError = database::Error::UsernameTaken.into();
    assert_eq!(err, UserError::UsernameTaken);
}

#[test]
fn from_database_email_taken() {
    let err: UserError = database::Error::EmailTaken.into();
    assert_eq!(err, UserError::EmailTaken);
}

#[test]
fn from_database_user_not_found() {
    let err: UserError = database::Error::UserNotFound.into();
    assert_eq!(err, UserError::UserNotFound);
}

#[test]
fn from_database_fallback() {
    let err: UserError = database::Error::CategoryNotFound.into();
    assert_eq!(err, UserError::DatabaseError);
}

#[test]
fn from_argon2_error() {
    let err: UserError = argon2::password_hash::Error::PasswordInvalid.into();
    assert_eq!(err, UserError::InternalServerError);
}

#[test]
fn from_serde_json_error() {
    let source: serde_json::Error = serde_json::from_str::<String>("not json").unwrap_err();
    let err: UserError = source.into();
    assert_eq!(err, UserError::InternalServerError);
}
