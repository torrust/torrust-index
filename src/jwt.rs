//! Centralised JWT (JSON Web Token) module.
//!
//! All `jsonwebtoken` usage is confined to this module: key loading,
//! signing, verification, and algorithm configuration.
//!
//! See ADR-T-007 for the rationale behind centralising JWT handling.
//!
//! ## Phase 2 changes (ADR-T-007)
//!
//! - `UserClaims` → [`SessionClaims`] with RFC 7519 registered claims.
//! - `VerifyClaims` redesigned with `aud: "email-verification"`.
//! - Per-purpose signing keys (`session_signing_key` and
//!   `email_verification_signing_key`).
//! - The `role` and `username` in [`SessionClaims`] are **advisory only**;
//!   the authoritative role is re-validated from the database on every
//!   authenticated request.

use std::sync::Arc;

use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use crate::config::Configuration;
use crate::errors::AuthError;
use crate::models::user::{UserCompact, UserId};
use crate::utils::clock;

// ── Issuer constant ──────────────────────────────────────────────────

const ISSUER: &str = "torrust-index";

// ── Claim types ──────────────────────────────────────────────────────

/// Claims embedded in session (login) JWTs.
///
/// Follows RFC 7519 registered claim names:
/// - `sub` — subject (user ID)
/// - `iss` — issuer (`"torrust-index"`)
/// - `aud` — audience (`"session"`)
/// - `iat` — issued-at (epoch seconds)
/// - `exp` — expiration (epoch seconds)
///
/// `role` and `username` are **advisory only**. The authoritative role
/// is always re-checked from the database on each authenticated request.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionClaims {
    /// Subject — the user ID.
    pub sub: UserId,
    /// Issuer.
    pub iss: String,
    /// Audience.
    pub aud: String,
    /// Issued-at (epoch seconds).
    pub iat: u64,
    /// Expiration (epoch seconds).
    pub exp: u64,
    /// Advisory role: `"admin"` or `"user"`. Non-authoritative.
    pub role: String,
    /// Advisory username. Non-authoritative.
    pub username: String,
}

/// Backward-compatible type alias.
///
/// Existing code that imports `UserClaims` will keep compiling.
/// New code should prefer [`SessionClaims`].
pub type UserClaims = SessionClaims;

/// Claims embedded in email-verification JWTs.
///
/// Phase 2: now includes `aud: "email-verification"` for purpose
/// separation, and `iss: "torrust-index"`.
#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyClaims {
    pub iss: String,
    pub aud: String,
    pub sub: i64,
    pub iat: u64,
    pub exp: u64,
}

// ── Service ──────────────────────────────────────────────────────────

/// Centralised JWT signing and verification service.
///
/// Holds a reference to [`Configuration`] so it can read the signing
/// secrets and token lifetimes at runtime.
pub struct JsonWebToken {
    cfg: Arc<Configuration>,
}

impl JsonWebToken {
    pub const fn new(cfg: Arc<Configuration>) -> Self {
        Self { cfg }
    }

    /// Sign a session JWT for the given user.
    ///
    /// # Errors
    ///
    /// Returns `AuthError::InternalServerError` if the token cannot be
    /// encoded (e.g. the encoding key is invalid).
    pub async fn sign(&self, user: UserCompact) -> Result<String, AuthError> {
        let settings = self.cfg.settings.read().await;
        let key = settings.auth.session_signing_key.as_bytes();
        let now = clock::now();
        let exp_date = now + settings.auth.session_token_lifetime_secs;
        let claims = SessionClaims {
            sub: user.user_id,
            iss: ISSUER.to_owned(),
            aud: "session".to_owned(),
            iat: now,
            exp: exp_date,
            role: if user.administrator {
                "admin".to_owned()
            } else {
                "user".to_owned()
            },
            username: user.username,
        };
        let result =
            encode(&Header::default(), &claims, &EncodingKey::from_secret(key)).map_err(|_| AuthError::InternalServerError);
        drop(settings);
        result
    }

    /// Verify a session JWT and return its claims.
    ///
    /// Expiration is validated by the `jsonwebtoken` library; there is
    /// no redundant manual check. The `aud` claim is validated as
    /// `"session"`.
    ///
    /// # Errors
    ///
    /// * `AuthError::TokenExpired` — the token's `exp` is in the past.
    /// * `AuthError::TokenInvalid` — signature mismatch or malformed token.
    pub async fn verify(&self, token: &str) -> Result<SessionClaims, AuthError> {
        let settings = self.cfg.settings.read().await;

        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_audience(&["session"]);
        validation.set_issuer(&[ISSUER]);

        let result = decode::<SessionClaims>(
            token,
            &DecodingKey::from_secret(settings.auth.session_signing_key.as_bytes()),
            &validation,
        )
        .map(|token_data| token_data.claims)
        .map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
            _ => AuthError::TokenInvalid,
        });

        drop(settings);
        result
    }

    /// Sign an email-verification JWT for the given user ID.
    ///
    /// Uses the dedicated `email_verification_signing_key`.
    ///
    /// # Errors
    ///
    /// Returns `AuthError::InternalServerError` if encoding fails.
    pub async fn sign_email_verification(&self, user_id: i64) -> Result<String, AuthError> {
        let settings = self.cfg.settings.read().await;
        let key = settings.auth.email_verification_signing_key.as_bytes();
        let now = clock::now();

        let claims = VerifyClaims {
            iss: ISSUER.to_owned(),
            aud: "email-verification".to_owned(),
            sub: user_id,
            iat: now,
            exp: now + settings.auth.email_verification_token_lifetime_secs,
        };

        let result =
            encode(&Header::default(), &claims, &EncodingKey::from_secret(key)).map_err(|_| AuthError::InternalServerError);
        drop(settings);
        result
    }

    /// Verify an email-verification JWT and return its claims.
    ///
    /// Validates `iss` and `aud` claims.
    ///
    /// # Errors
    ///
    /// * `AuthError::TokenExpired` — the token's `exp` is in the past.
    /// * `AuthError::TokenInvalid` — bad signature, wrong issuer/audience, or malformed.
    pub async fn verify_email_token(&self, token: &str) -> Result<VerifyClaims, AuthError> {
        let settings = self.cfg.settings.read().await;

        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_audience(&["email-verification"]);
        validation.set_issuer(&[ISSUER]);

        let result = decode::<VerifyClaims>(
            token,
            &DecodingKey::from_secret(settings.auth.email_verification_signing_key.as_bytes()),
            &validation,
        )
        .map(|token_data| token_data.claims)
        .map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
            _ => AuthError::TokenInvalid,
        });

        drop(settings);
        result
    }
}
