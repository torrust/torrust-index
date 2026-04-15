//! Crate tests for the centralised JWT module (`src/jwt.rs`).
//!
//! # Test index
//!
//! ## Session tokens
//!
//! - [`sign_and_verify_session_token`] — round-trip sign → verify.
//! - [`session_token_contains_expected_claims`] — `sub`, `iss`, `aud`,
//!   `role`, `username`, `gen`.
//! - [`session_token_admin_role`] — admin users get `role: "admin"`.
//! - [`session_token_has_kid_header`] — `kid` is present in the JWT
//!   header.
//! - [`session_token_rejected_with_email_verification_audience`] —
//!   audience mismatch → `TokenInvalid`.
//!
//! ## Email-verification tokens
//!
//! - [`sign_and_verify_email_verification_token`] — round-trip.
//! - [`email_token_contains_expected_claims`] — `sub`, `iss`, `aud`.
//! - [`email_token_rejected_with_session_audience`] — audience
//!   mismatch → `TokenInvalid`.
//!
//! ## Error paths
//!
//! - [`verify_rejects_garbage_token`] — random string → `TokenInvalid`.
//! - [`verify_rejects_tampered_token`] — flipped character →
//!   `TokenInvalid`.

use std::sync::Arc;

use crate::config::Configuration;
use crate::errors::AuthError;
use crate::jwt::JsonWebToken;
use crate::models::user::UserCompact;

/// Build a `JsonWebToken` service using ephemeral auto-generated keys
/// (the default when no key paths are configured).
async fn jwt_service() -> JsonWebToken {
    let cfg = Arc::new(Configuration::default());
    JsonWebToken::new(cfg).await
}

fn test_user(admin: bool) -> UserCompact {
    UserCompact {
        user_id: 42,
        username: "testuser".to_string(),
        administrator: admin,
    }
}

// ── Session tokens ───────────────────────────────────────────────────

#[tokio::test]
async fn sign_and_verify_session_token() {
    let jwt = jwt_service().await;
    let token = jwt.sign(test_user(false), 0).await.unwrap();
    let claims = jwt.verify(&token).unwrap();

    assert_eq!(claims.sub, 42);
    assert_eq!(claims.username, "testuser");
}

#[tokio::test]
async fn session_token_contains_expected_claims() {
    let jwt = jwt_service().await;
    let token = jwt.sign(test_user(false), 7).await.unwrap();
    let claims = jwt.verify(&token).unwrap();

    assert_eq!(claims.iss, "torrust-index");
    assert_eq!(claims.aud, "session");
    assert_eq!(claims.role, "user");
    assert_eq!(claims.token_gen, 7);
    assert!(claims.iat > 0);
    assert!(claims.exp > claims.iat);
}

#[tokio::test]
async fn session_token_admin_role() {
    let jwt = jwt_service().await;
    let token = jwt.sign(test_user(true), 0).await.unwrap();
    let claims = jwt.verify(&token).unwrap();

    assert_eq!(claims.role, "admin");
}

#[tokio::test]
async fn session_token_has_kid_header() {
    let jwt = jwt_service().await;
    let token = jwt.sign(test_user(false), 0).await.unwrap();

    // Decode the header (first segment) without verification.
    let header = jsonwebtoken::decode_header(&token).unwrap();
    assert!(header.kid.is_some(), "JWT header should contain a `kid`");
    assert_eq!(header.alg, jsonwebtoken::Algorithm::RS256);
}

#[tokio::test]
async fn session_token_rejected_with_email_verification_audience() {
    let jwt = jwt_service().await;
    let token = jwt.sign(test_user(false), 0).await.unwrap();

    // Trying to verify a session token as an email-verification token
    // should fail because the audience does not match.
    let err = jwt.verify_email_token(&token).unwrap_err();
    assert_eq!(err, AuthError::TokenInvalid);
}

// ── Email-verification tokens ────────────────────────────────────────

#[tokio::test]
async fn sign_and_verify_email_verification_token() {
    let jwt = jwt_service().await;
    let token = jwt.sign_email_verification(99).await.unwrap();
    let claims = jwt.verify_email_token(&token).unwrap();

    assert_eq!(claims.sub, 99);
}

#[tokio::test]
async fn email_token_contains_expected_claims() {
    let jwt = jwt_service().await;
    let token = jwt.sign_email_verification(99).await.unwrap();
    let claims = jwt.verify_email_token(&token).unwrap();

    assert_eq!(claims.iss, "torrust-index");
    assert_eq!(claims.aud, "email-verification");
    assert!(claims.iat > 0);
    assert!(claims.exp > claims.iat);
}

#[tokio::test]
async fn email_token_rejected_with_session_audience() {
    let jwt = jwt_service().await;
    let token = jwt.sign_email_verification(99).await.unwrap();

    // Trying to verify an email-verification token as a session token
    // should fail because the audience does not match.
    let err = jwt.verify(&token).unwrap_err();
    assert_eq!(err, AuthError::TokenInvalid);
}

// ── Error paths ──────────────────────────────────────────────────────

#[tokio::test]
async fn verify_rejects_garbage_token() {
    let jwt = jwt_service().await;
    let err = jwt.verify("not-a-jwt").unwrap_err();
    assert_eq!(err, AuthError::TokenInvalid);
}

#[tokio::test]
async fn verify_rejects_tampered_token() {
    let jwt = jwt_service().await;
    let token = jwt.sign(test_user(false), 0).await.unwrap();

    // Flip a character in the signature (last segment).
    let mut chars: Vec<char> = token.chars().collect();
    let last = chars.len() - 1;
    chars[last] = if chars[last] == 'A' { 'B' } else { 'A' };
    let tampered: String = chars.into_iter().collect();

    let err = jwt.verify(&tampered).unwrap_err();
    assert_eq!(err, AuthError::TokenInvalid);
}
