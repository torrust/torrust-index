use std::path::Path;

use serde::{Deserialize, Serialize};
use tracing::warn;

/// Default session-token lifetime: 2 weeks (1 209 600 s).
const DEFAULT_SESSION_TOKEN_LIFETIME_SECS: u64 = 1_209_600;

/// Default email-verification-token lifetime: ~10 years (315 569 260 s).
const DEFAULT_EMAIL_VERIFICATION_TOKEN_LIFETIME_SECS: u64 = 315_569_260;

/// Default paths for the development RSA key pair shipped with the repo.
const DEFAULT_PRIVATE_KEY_PATH: &str = "./share/default/jwt/private.pem";
const DEFAULT_PUBLIC_KEY_PATH: &str = "./share/default/jwt/public.pem";

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
/// If neither is provided, the development key pair shipped at
/// `share/default/jwt/` is used with a loud warning.
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
            private_key_path: Self::default_private_key_path(),
            public_key_path: Self::default_public_key_path(),
            session_token_lifetime_secs: Self::default_session_token_lifetime_secs(),
            email_verification_token_lifetime_secs: Self::default_email_verification_token_lifetime_secs(),
            password_constraints: Self::default_password_constraints(),
        }
    }
}

impl Auth {
    #[allow(clippy::unnecessary_wraps)] // serde default must match the field type
    fn default_private_key_path() -> Option<String> {
        Some(DEFAULT_PRIVATE_KEY_PATH.to_owned())
    }

    #[allow(clippy::unnecessary_wraps)] // serde default must match the field type
    fn default_public_key_path() -> Option<String> {
        Some(DEFAULT_PUBLIC_KEY_PATH.to_owned())
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

    /// Resolve the RSA private key PEM bytes.
    ///
    /// Resolution order:
    /// 1. Inline PEM (`private_key_pem`)
    /// 2. File path (`private_key_path`)
    /// 3. Fallback to default dev key path (with warning)
    ///
    /// # Panics
    ///
    /// Panics if no valid private key PEM can be resolved.
    #[must_use]
    pub fn resolve_private_key_pem(&self) -> Vec<u8> {
        if let Some(ref pem) = self.private_key_pem {
            return pem.as_bytes().to_vec();
        }

        if let Some(ref path) = self.private_key_path {
            if Path::new(path).exists() {
                if path == DEFAULT_PRIVATE_KEY_PATH {
                    warn!(
                        "Using the DEVELOPMENT RSA private key at `{path}`. \
                         This key is PUBLIC and must NOT be used in production! \
                         Generate your own key pair: \
                         `openssl genrsa -out private.pem 2048 && openssl rsa -in private.pem -pubout -out public.pem`"
                    );
                }
                return std::fs::read(path).unwrap_or_else(|e| panic!("Failed to read RSA private key from `{path}`: {e}"));
            }
        }

        panic!(
            "No RSA private key configured. Set `auth.private_key_path` or `auth.private_key_pem` in the configuration, \
             or generate a key pair: `openssl genrsa -out private.pem 2048`"
        );
    }

    /// Resolve the RSA public key PEM bytes.
    ///
    /// Resolution order:
    /// 1. Inline PEM (`public_key_pem`)
    /// 2. File path (`public_key_path`)
    /// 3. Fallback to default dev key path (with warning)
    ///
    /// # Panics
    ///
    /// Panics if no valid public key PEM can be resolved.
    #[must_use]
    pub fn resolve_public_key_pem(&self) -> Vec<u8> {
        if let Some(ref pem) = self.public_key_pem {
            return pem.as_bytes().to_vec();
        }

        if let Some(ref path) = self.public_key_path {
            if Path::new(path).exists() {
                if path == DEFAULT_PUBLIC_KEY_PATH {
                    warn!(
                        "Using the DEVELOPMENT RSA public key at `{path}`. \
                         This key is PUBLIC and must NOT be used in production!"
                    );
                }
                return std::fs::read(path).unwrap_or_else(|e| panic!("Failed to read RSA public key from `{path}`: {e}"));
            }
        }

        panic!(
            "No RSA public key configured. Set `auth.public_key_path` or `auth.public_key_pem` in the configuration, \
             or generate a key pair: `openssl rsa -in private.pem -pubout -out public.pem`"
        );
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
