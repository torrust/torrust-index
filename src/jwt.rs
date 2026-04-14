//! Centralised JWT (JSON Web Token) module.
//!
//! All `jsonwebtoken` usage is confined to this module: key loading,
//! signing, verification, and algorithm configuration.
//!
//! See ADR-T-007 for the rationale behind centralising JWT handling.

use std::sync::Arc;

use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use crate::config::Configuration;
use crate::errors::AuthError;
use crate::models::user::UserCompact;
use crate::utils::clock;

// ── Claim types ──────────────────────────────────────────────────────

/// Claims embedded in session (login) JWTs.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserClaims {
    pub user: UserCompact,
    pub exp: u64,
}

/// Claims embedded in email-verification JWTs.
#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyClaims {
    pub iss: String,
    pub sub: i64,
    pub exp: u64,
}

// ── Service ──────────────────────────────────────────────────────────

/// Centralised JWT signing and verification service.
///
/// Holds a reference to [`Configuration`] so it can read the signing
/// secret and token lifetimes at runtime.
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
        let key = settings.auth.jwt_signing_secret.as_bytes();
        let exp_date = clock::now() + settings.auth.session_token_lifetime_secs;
        let claims = UserClaims { user, exp: exp_date };
        let result =
            encode(&Header::default(), &claims, &EncodingKey::from_secret(key)).map_err(|_| AuthError::InternalServerError);
        drop(settings);
        result
    }

    /// Verify a session JWT and return its claims.
    ///
    /// Expiration is validated by the `jsonwebtoken` library; there is
    /// no redundant manual check.
    ///
    /// # Errors
    ///
    /// * `AuthError::TokenExpired` — the token's `exp` is in the past.
    /// * `AuthError::TokenInvalid` — signature mismatch or malformed token.
    pub async fn verify(&self, token: &str) -> Result<UserClaims, AuthError> {
        let settings = self.cfg.settings.read().await;

        let result = decode::<UserClaims>(
            token,
            &DecodingKey::from_secret(settings.auth.jwt_signing_secret.as_bytes()),
            &Validation::new(Algorithm::HS256),
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
    /// # Errors
    ///
    /// Returns `AuthError::InternalServerError` if encoding fails.
    pub async fn sign_email_verification(&self, user_id: i64) -> Result<String, AuthError> {
        let settings = self.cfg.settings.read().await;
        let key = settings.auth.jwt_signing_secret.as_bytes();

        let claims = VerifyClaims {
            iss: String::from("email-verification"),
            sub: user_id,
            exp: clock::now() + settings.auth.email_verification_token_lifetime_secs,
        };

        let result =
            encode(&Header::default(), &claims, &EncodingKey::from_secret(key)).map_err(|_| AuthError::InternalServerError);
        drop(settings);
        result
    }

    /// Verify an email-verification JWT and return its claims.
    ///
    /// Rejects tokens whose `iss` claim is not `"email-verification"`.
    ///
    /// # Errors
    ///
    /// * `AuthError::TokenExpired` — the token's `exp` is in the past.
    /// * `AuthError::TokenInvalid` — bad signature, wrong issuer, or malformed.
    pub async fn verify_email_token(&self, token: &str) -> Result<VerifyClaims, AuthError> {
        let settings = self.cfg.settings.read().await;

        let token_data = decode::<VerifyClaims>(
            token,
            &DecodingKey::from_secret(settings.auth.jwt_signing_secret.as_bytes()),
            &Validation::new(Algorithm::HS256),
        )
        .map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
            _ => AuthError::TokenInvalid,
        })?;

        drop(settings);

        if token_data.claims.iss != "email-verification" {
            return Err(AuthError::TokenInvalid);
        }

        Ok(token_data.claims)
    }
}
