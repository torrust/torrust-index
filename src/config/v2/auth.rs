use std::path::Path;

use serde::{Deserialize, Serialize};

/// Default session-token lifetime: 2 weeks (1 209 600 s).
const DEFAULT_SESSION_TOKEN_LIFETIME_SECS: u64 = 1_209_600;

/// Default email-verification-token lifetime: ~10 years (315 569 260 s).
const DEFAULT_EMAIL_VERIFICATION_TOKEN_LIFETIME_SECS: u64 = 315_569_260;

/// Authentication options.
///
/// ## JWT signing (ADR-T-007)
///
/// JWT signing uses RS256 (RSA + SHA-256) with a public/private key
/// pair. Session and email-verification tokens share the same key
/// pair; purpose separation is via the `aud` claim. A per-user
/// `token_generation` counter enables near-instant revocation on
/// password change, role change, or ban.
///
/// Configuration supports two mechanisms (in priority order):
///
/// 1. **Inline PEM** — `private_key_pem` / `public_key_pem`.
///    Primarily for passing keys via environment variables
///    (e.g., `TORRUST_INDEX_CONFIG_OVERRIDE_AUTH__PRIVATE_KEY_PEM`).
/// 2. **File paths** — `private_key_path` / `public_key_path`.
///    Point to PEM files on disk.
///
/// If neither is provided, an ephemeral RSA-2048 key pair is
/// auto-generated in memory at startup. Sessions will not survive
/// server restarts. To persist sessions, generate your own key pair
/// and configure the paths or environment variables.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Auth {
    /// Inline RSA private key in PEM format (overrides `private_key_path`).
    ///
    /// Use this when passing the key via environment variable.
    #[serde(default)]
    pub private_key_pem: Option<String>,

    /// Inline RSA public key in PEM format (overrides `public_key_path`).
    ///
    /// Use this when passing the key via environment variable.
    #[serde(default)]
    pub public_key_pem: Option<String>,

    /// Path to the RSA private key PEM file for JWT signing.
    #[serde(default = "Auth::default_private_key_path")]
    pub private_key_path: Option<String>,

    /// Path to the RSA public key PEM file for JWT verification.
    #[serde(default = "Auth::default_public_key_path")]
    pub public_key_path: Option<String>,

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
            private_key_pem: None,
            public_key_pem: None,
            private_key_path: None,
            public_key_path: None,
            session_token_lifetime_secs: Self::default_session_token_lifetime_secs(),
            email_verification_token_lifetime_secs: Self::default_email_verification_token_lifetime_secs(),
            password_constraints: Self::default_password_constraints(),
        }
    }
}

impl Auth {
    const fn default_private_key_path() -> Option<String> {
        None
    }

    const fn default_public_key_path() -> Option<String> {
        None
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

    /// Resolve the RSA private key PEM bytes, if configured.
    ///
    /// Resolution order:
    /// 1. Inline PEM (`private_key_pem`)
    /// 2. File path (`private_key_path`)
    ///
    /// Returns `None` if no key is configured or the configured path
    /// does not exist. The caller (`JsonWebToken::new`) uses this to
    /// decide whether to auto-generate an ephemeral key pair.
    ///
    /// # Panics
    ///
    /// Panics if a configured path exists but cannot be read.
    #[must_use]
    pub fn resolve_private_key_pem(&self) -> Option<Vec<u8>> {
        if let Some(ref pem) = self.private_key_pem {
            return Some(pem.as_bytes().to_vec());
        }

        if let Some(ref path) = self.private_key_path {
            if Path::new(path).exists() {
                return Some(std::fs::read(path).unwrap_or_else(|e| panic!("Failed to read RSA private key from `{path}`: {e}")));
            }
        }

        None
    }

    /// Resolve the RSA public key PEM bytes, if configured.
    ///
    /// Resolution order:
    /// 1. Inline PEM (`public_key_pem`)
    /// 2. File path (`public_key_path`)
    ///
    /// Returns `None` if no key is configured or the configured path
    /// does not exist. The caller (`JsonWebToken::new`) uses this to
    /// decide whether to auto-generate an ephemeral key pair.
    ///
    /// # Panics
    ///
    /// Panics if a configured path exists but cannot be read.
    #[must_use]
    pub fn resolve_public_key_pem(&self) -> Option<Vec<u8>> {
        if let Some(ref pem) = self.public_key_pem {
            return Some(pem.as_bytes().to_vec());
        }

        if let Some(ref path) = self.public_key_path {
            if Path::new(path).exists() {
                return Some(std::fs::read(path).unwrap_or_else(|e| panic!("Failed to read RSA public key from `{path}`: {e}")));
            }
        }

        None
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
