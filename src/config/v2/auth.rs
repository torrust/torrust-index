use std::fmt;

use serde::{Deserialize, Serialize};

/// Default session-token lifetime: 2 weeks (1 209 600 s).
const DEFAULT_SESSION_TOKEN_LIFETIME_SECS: u64 = 1_209_600;

/// Default email-verification-token lifetime: ~10 years (315 569 260 s).
const DEFAULT_EMAIL_VERIFICATION_TOKEN_LIFETIME_SECS: u64 = 315_569_260;

/// Minimum allowed length for signing secrets (ADR-T-007 Phase 2).
const MIN_SECRET_LENGTH: usize = 32;

/// Authentication options.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Auth {
    /// The HMAC secret used to sign session JWT tokens.
    ///
    /// Phase 2 (ADR-T-007): renamed from `jwt_signing_secret` to
    /// `session_signing_key` to reflect per-purpose key separation.
    #[serde(
        default = "Auth::default_session_signing_key",
        alias = "jwt_signing_secret",
        alias = "user_claim_token_pepper"
    )]
    pub session_signing_key: JwtSigningSecret,

    /// The HMAC secret used to sign email-verification JWT tokens.
    ///
    /// If absent, falls back to `session_signing_key` for backward
    /// compatibility, but deployers should provide a separate value.
    #[serde(default = "Auth::default_email_verification_signing_key")]
    pub email_verification_signing_key: JwtSigningSecret,

    /// Session-token lifetime in seconds (default: 2 weeks).
    #[serde(default = "Auth::default_session_token_lifetime_secs")]
    pub session_token_lifetime_secs: u64,

    /// Email-verification-token lifetime in seconds (default: ~10 years).
    #[serde(default = "Auth::default_email_verification_token_lifetime_secs")]
    pub email_verification_token_lifetime_secs: u64,

    /// The password constraints.
    #[serde(default = "Auth::default_password_constraints")]
    pub password_constraints: PasswordConstraints,
}

impl Default for Auth {
    fn default() -> Self {
        Self {
            session_signing_key: Self::default_session_signing_key(),
            email_verification_signing_key: Self::default_email_verification_signing_key(),
            session_token_lifetime_secs: Self::default_session_token_lifetime_secs(),
            email_verification_token_lifetime_secs: Self::default_email_verification_token_lifetime_secs(),
            password_constraints: Self::default_password_constraints(),
        }
    }
}

impl Auth {
    pub fn override_session_signing_key(&mut self, secret: &str) {
        self.session_signing_key = JwtSigningSecret::new(secret);
    }

    pub fn override_email_verification_signing_key(&mut self, secret: &str) {
        self.email_verification_signing_key = JwtSigningSecret::new(secret);
    }

    fn default_session_signing_key() -> JwtSigningSecret {
        JwtSigningSecret::new("MaxVerstappenWC2021-session-key!")
    }

    fn default_email_verification_signing_key() -> JwtSigningSecret {
        JwtSigningSecret::new("MaxVerstappenWC2021-emailverify!")
    }

    const fn default_session_token_lifetime_secs() -> u64 {
        DEFAULT_SESSION_TOKEN_LIFETIME_SECS
    }

    const fn default_email_verification_token_lifetime_secs() -> u64 {
        DEFAULT_EMAIL_VERIFICATION_TOKEN_LIFETIME_SECS
    }

    fn default_password_constraints() -> PasswordConstraints {
        PasswordConstraints::default()
    }
}

/// The HMAC signing secret for JWT tokens.
///
/// Renamed from `ClaimTokenPepper` (see ADR-T-007) — the old name
/// incorrectly suggested a password-hashing "pepper" rather than a
/// signing key.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JwtSigningSecret(String);

impl JwtSigningSecret {
    /// Creates a new signing secret.
    ///
    /// # Panics
    ///
    /// Will panic if the key is empty or shorter than 32 bytes.
    #[must_use]
    pub fn new(key: &str) -> Self {
        assert!(!key.is_empty(), "secret key cannot be empty");
        assert!(
            key.len() >= MIN_SECRET_LENGTH,
            "secret key must be at least {MIN_SECRET_LENGTH} bytes, got {}",
            key.len()
        );

        Self(key.to_owned())
    }

    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl fmt::Display for JwtSigningSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PasswordConstraints {
    /// The maximum password length.
    #[serde(default = "PasswordConstraints::default_max_password_length")]
    pub max_password_length: usize,
    /// The minimum password length.
    #[serde(default = "PasswordConstraints::default_min_password_length")]
    pub min_password_length: usize,
}

impl Default for PasswordConstraints {
    fn default() -> Self {
        Self {
            max_password_length: Self::default_max_password_length(),
            min_password_length: Self::default_min_password_length(),
        }
    }
}

impl PasswordConstraints {
    const fn default_min_password_length() -> usize {
        6
    }

    const fn default_max_password_length() -> usize {
        64
    }
}
