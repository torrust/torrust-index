//! Centralised JWT (JSON Web Token) module.
//!
//! All `jsonwebtoken` usage is confined to this module: key loading,
//! signing, verification, and algorithm configuration.
//!
//! See ADR-T-007 for the rationale behind centralising JWT handling.
//!
//! # Architecture (ADR-T-007 Phases 1–7)
//!
//! **Phase 1 — Structural cleanup.** Consolidated all `jsonwebtoken`
//! usage into this single module with `Result`-based error propagation.
//!
//! **Phase 2 — Claim redesign.** `SessionClaims` follows RFC 7519
//! registered claim names (`sub`, `iss`, `aud`, `iat`, `exp`) with
//! advisory `role`/`username` fields. Purpose separation between
//! session and email-verification tokens is via the `aud` claim.
//!
//! **Phase 3 — RS256 asymmetric signing.** Switched from HMAC-HS256
//! to RS256 (RSA + SHA-256). A single RSA key pair is used for all
//! token purposes. `EncodingKey` (private) signs; `DecodingKey`
//! (public) verifies. A `kid` (Key ID) is included in every JWT
//! header for future key rotation. Keys are resolved once at
//! construction time from PEM files or inline PEM config.
//!
//! **Phase 4 — Token revocation.** `SessionClaims` includes a `gen`
//! (token generation) counter. Password changes, role changes, and
//! bans increment the counter in the database; tokens carrying an
//! older `gen` are rejected at verification time.
//!
//! **Phase 5 — Ephemeral auto-generated keys.** When no key paths or
//! inline PEM values are configured, an RSA-2048 key pair is
//! auto-generated in memory at startup. The keys are never written to
//! disk; sessions do not survive server restarts. Deployers who want
//! persistent sessions supply their own key pair via config.
//!
//! **Phase 6 — `torrust-index-auth-keypair` CLI.** A standalone binary
//! (`torrust-index-auth-keypair`) generates an RSA-2048 key pair
//! and writes a JSON object with both PEM blocks to stdout. The container
//! entry script uses it to auto-generate persistent keys on first boot.
//! See `packages/index-auth-keypair/src/bin/torrust-index-auth-keypair.rs`
//! for the binary and ADR-T-007 Phase 6 / ADR-T-009 Phase 2 / ADR-T-010
//! for full context.
//!
//! **Phase 7 — Consolidated session validation.** `validate_session`
//! is the sole entry point for session-token validation: it verifies
//! the JWT, checks the token-generation counter, and rejects banned
//! users. All callers delegate here instead of re-implementing the
//! sequence. See ADR-T-007 Phase 7.

use std::sync::Arc;

use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use rsa::RsaPrivateKey;
use rsa::pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::info;

use crate::config::Configuration;
use crate::databases::database::Database;
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
    /// Token generation counter. Validated by
    /// [`JsonWebToken::validate_session`] — tokens whose `gen` does
    /// not match the current database value are rejected.
    /// See ADR-T-007 Phases 4 & 7.
    #[serde(rename = "gen")]
    pub token_gen: u64,
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
    /// Create a new `JsonWebToken` service.
    ///
    /// Key resolution:
    /// 1. Inline PEM or file path from configuration → host-supplied keys.
    /// 2. Neither configured → auto-generate an ephemeral RSA-2048 key
    ///    pair in memory (keys are never written to disk).
    ///
    /// # Panics
    ///
    /// Panics if a configured key path exists but contains invalid PEM.
    pub async fn new(cfg: Arc<Configuration>) -> Self {
        let settings = cfg.settings.read().await;
        let private_pem_opt = settings.auth.resolve_private_key_pem();
        let public_pem_opt = settings.auth.resolve_public_key_pem();
        drop(settings);

        let (private_pem, public_pem) = match (private_pem_opt, public_pem_opt) {
            (Some(priv_pem), Some(pub_pem)) => (priv_pem, pub_pem),
            (None, None) => {
                info!(
                    "Using ephemeral auto-generated RSA key pair. \
                     Sessions will not survive server restarts. \
                     To persist sessions, configure auth.private_key_path / auth.public_key_path."
                );
                generate_ephemeral_key_pair().await
            }
            (Some(_), None) => {
                panic!(
                    "RSA private key is configured but public key is missing. \
                     Set both auth.private_key_path / auth.private_key_pem and \
                     auth.public_key_path / auth.public_key_pem, or remove both \
                     to use ephemeral auto-generated keys."
                );
            }
            (None, Some(_)) => {
                panic!(
                    "RSA public key is configured but private key is missing. \
                     Set both auth.private_key_path / auth.private_key_pem and \
                     auth.public_key_path / auth.public_key_pem, or remove both \
                     to use ephemeral auto-generated keys."
                );
            }
        };

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
    pub async fn sign(&self, user: UserCompact, token_generation: u64) -> Result<String, AuthError> {
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
            role: user.role.clone(),
            username: user.username,
            token_gen: token_generation,
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

    /// Verify a session JWT and validate it against the database.
    ///
    /// This is the **sole entry point** for session-token validation.
    /// It verifies the JWT signature and expiry, checks the token
    /// generation counter, and rejects banned users.
    ///
    /// # Errors
    ///
    /// * `AuthError::TokenExpired` — the token's `exp` is in the past.
    /// * `AuthError::TokenInvalid` — signature mismatch or malformed token.
    /// * `AuthError::TokenRevoked` — generation mismatch or user is banned.
    pub async fn validate_session(&self, db: &dyn Database, token: &str) -> Result<SessionClaims, AuthError> {
        let claims = self.verify(token)?;

        let current_gen = db.get_token_generation(claims.sub).await?;

        if claims.token_gen != current_gen {
            return Err(AuthError::TokenRevoked);
        }

        if db.is_user_banned(claims.sub).await? {
            return Err(AuthError::TokenRevoked);
        }

        Ok(claims)
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

/// Generate an ephemeral RSA-2048 key pair in memory.
///
/// The generation is CPU-intensive (~100-300 ms) and is offloaded to
/// a blocking thread via `tokio::task::spawn_blocking`.
///
/// Returns `(private_pem_bytes, public_pem_bytes)`.
///
/// # Panics
///
/// Panics if RSA key generation or PEM export fails (indicates a bug
/// in the `rsa` crate or a system RNG failure).
async fn generate_ephemeral_key_pair() -> (Vec<u8>, Vec<u8>) {
    tokio::task::spawn_blocking(|| {
        let mut rng = rsa::rand_core::OsRng;
        let private_key = RsaPrivateKey::new(&mut rng, 2048).expect("RSA-2048 key generation failed");

        let private_pem = private_key
            .to_pkcs8_pem(LineEnding::LF)
            .expect("RSA private key PEM export failed");

        let public_pem = private_key
            .to_public_key()
            .to_public_key_pem(LineEnding::LF)
            .expect("RSA public key PEM export failed");

        (private_pem.as_bytes().to_vec(), public_pem.into_bytes())
    })
    .await
    .expect("ephemeral key generation task panicked")
}
