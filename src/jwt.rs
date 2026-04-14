//! Centralised JWT (JSON Web Token) module.
//!
//! All `jsonwebtoken` usage is confined to this module: key loading,
//! signing, verification, and algorithm configuration.
//!
//! See ADR-T-007 for the rationale behind centralising JWT handling.
//!
//! ## Phase 3 changes (ADR-T-007)
//!
//! - Switched from HMAC-HS256 to RS256 (RSA + SHA-256) asymmetric signing.
//! - Single RSA key pair for all token purposes (session and
//!   email-verification); purpose separation is via the `aud` claim.
//! - `EncodingKey` (private) is used only for signing; `DecodingKey`
//!   (public) is used only for verification.
//! - A `kid` (Key ID) is included in every JWT header to support
//!   future key rotation.
//! - Keys are resolved once at construction time from PEM files or
//!   inline PEM config, not on every request.

use std::sync::Arc;

use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

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
/// Includes `aud: "email-verification"` for purpose separation,
/// and `iss: "torrust-index"`.
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
/// Holds pre-loaded RSA keys (`EncodingKey` from the private key,
/// `DecodingKey` from the public key) and a reference to
/// [`Configuration`] for token lifetimes at runtime.
pub struct JsonWebToken {
    cfg: Arc<Configuration>,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    kid: String,
}

impl JsonWebToken {
    /// Create a new `JsonWebToken` service, resolving the RSA key pair
    /// from the configuration.
    ///
    /// # Panics
    ///
    /// Panics if the RSA key PEM cannot be resolved or is invalid.
    pub async fn new(cfg: Arc<Configuration>) -> Self {
        let settings = cfg.settings.read().await;
        let private_pem = settings.auth.resolve_private_key_pem();
        let public_pem = settings.auth.resolve_public_key_pem();
        drop(settings);

        let encoding_key = EncodingKey::from_rsa_pem(&private_pem)
            .expect("Invalid RSA private key PEM — check auth.private_key_path or auth.private_key_pem");
        let decoding_key = DecodingKey::from_rsa_pem(&public_pem)
            .expect("Invalid RSA public key PEM — check auth.public_key_path or auth.public_key_pem");

        let kid = compute_kid(&public_pem);

        Self {
            cfg,
            encoding_key,
            decoding_key,
            kid,
        }
    }

    /// Sign a session JWT for the given user.
    ///
    /// # Errors
    ///
    /// Returns `AuthError::InternalServerError` if the token cannot be
    /// encoded.
    pub async fn sign(&self, user: UserCompact) -> Result<String, AuthError> {
        let settings = self.cfg.settings.read().await;
        let now = clock::now();
        let exp_date = now + settings.auth.session_token_lifetime_secs;
        drop(settings);

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

        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(self.kid.clone());

        encode(&header, &claims, &self.encoding_key).map_err(|_| AuthError::InternalServerError)
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
    pub fn verify(&self, token: &str) -> Result<SessionClaims, AuthError> {
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&["session"]);
        validation.set_issuer(&[ISSUER]);

        decode::<SessionClaims>(token, &self.decoding_key, &validation)
            .map(|token_data| token_data.claims)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
                _ => AuthError::TokenInvalid,
            })
    }

    /// Sign an email-verification JWT for the given user ID.
    ///
    /// Uses the same RSA key pair as session tokens; purpose
    /// separation is via the `aud` claim.
    ///
    /// # Errors
    ///
    /// Returns `AuthError::InternalServerError` if encoding fails.
    pub async fn sign_email_verification(&self, user_id: i64) -> Result<String, AuthError> {
        let settings = self.cfg.settings.read().await;
        let now = clock::now();
        let exp = now + settings.auth.email_verification_token_lifetime_secs;
        drop(settings);

        let claims = VerifyClaims {
            iss: ISSUER.to_owned(),
            aud: "email-verification".to_owned(),
            sub: user_id,
            iat: now,
            exp,
        };

        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(self.kid.clone());

        encode(&header, &claims, &self.encoding_key).map_err(|_| AuthError::InternalServerError)
    }

    /// Verify an email-verification JWT and return its claims.
    ///
    /// Validates `iss` and `aud` claims.
    ///
    /// # Errors
    ///
    /// * `AuthError::TokenExpired` — the token's `exp` is in the past.
    /// * `AuthError::TokenInvalid` — bad signature, wrong issuer/audience, or malformed.
    pub fn verify_email_token(&self, token: &str) -> Result<VerifyClaims, AuthError> {
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&["email-verification"]);
        validation.set_issuer(&[ISSUER]);

        decode::<VerifyClaims>(token, &self.decoding_key, &validation)
            .map(|token_data| token_data.claims)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
                _ => AuthError::TokenInvalid,
            })
    }
}

/// Compute a deterministic Key ID from the public key PEM.
///
/// Uses the first 16 hex characters (8 bytes) of the SHA-256 hash of
/// the PEM content. This is stable across restarts for the same key
/// and supports future key rotation via the JWT `kid` header.
fn compute_kid(public_key_pem: &[u8]) -> String {
    let hash = Sha256::digest(public_key_pem);
    hex::encode(&hash[..8])
}
