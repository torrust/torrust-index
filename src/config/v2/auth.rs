use std::fmt;

use serde::{Deserialize, Serialize};

/// Default session-token lifetime: 2 weeks (1 209 600 s).
const DEFAULT_SESSION_TOKEN_LIFETIME_SECS: u64 = 1_209_600;

/// Default email-verification-token lifetime: ~10 years (315 569 260 s).
const DEFAULT_EMAIL_VERIFICATION_TOKEN_LIFETIME_SECS: u64 = 315_569_260;

/// Authentication options.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Auth {
    /// The HMAC secret used to sign JWT tokens.
    #[serde(default = "Auth::default_jwt_signing_secret")]
    pub jwt_signing_secret: JwtSigningSecret,

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
            jwt_signing_secret: Self::default_jwt_signing_secret(),
            session_token_lifetime_secs: Self::default_session_token_lifetime_secs(),
            email_verification_token_lifetime_secs: Self::default_email_verification_token_lifetime_secs(),
            password_constraints: Self::default_password_constraints(),
        }
    }
}

impl Auth {
    pub fn override_jwt_signing_secret(&mut self, secret: &str) {
        self.jwt_signing_secret = JwtSigningSecret::new(secret);
    }

    fn default_jwt_signing_secret() -> JwtSigningSecret {
        JwtSigningSecret::new("MaxVerstappenWC2021")
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
    /// # Panics
    ///
    /// Will panic if the key is empty.
    #[must_use]
    pub fn new(key: &str) -> Self {
        assert!(!key.is_empty(), "secret key cannot be empty");

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
