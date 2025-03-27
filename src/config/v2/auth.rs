use std::fmt;

use serde::{Deserialize, Serialize};

/// Authentication options.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Auth {
    /// The secret key used to sign JWT tokens.
    #[serde(default = "Auth::default_user_claim_token_pepper")]
    pub user_claim_token_pepper: ClaimTokenPepper,

    /// The password constraints
    #[serde(default = "Auth::default_password_constraints")]
    pub password_constraints: PasswordConstraints,

    /// The password reset rate-limiting policy.
    #[serde(default = "Auth::default_password_reset_policy")]
    pub password_reset_policy: ThrottlePolicy,

    /// The email-verification resend rate-limiting policy.
    #[serde(default = "Auth::default_email_verification_policy")]
    pub email_verification_policy: ThrottlePolicy,
}

impl Default for Auth {
    fn default() -> Self {
        Self {
            password_constraints: Self::default_password_constraints(),
            user_claim_token_pepper: Self::default_user_claim_token_pepper(),
            password_reset_policy: Self::default_password_reset_policy(),
            email_verification_policy: Self::default_email_verification_policy(),
        }
    }
}

impl Auth {
    pub fn override_user_claim_token_pepper(&mut self, user_claim_token_pepper: &str) {
        self.user_claim_token_pepper = ClaimTokenPepper::new(user_claim_token_pepper);
    }

    fn default_user_claim_token_pepper() -> ClaimTokenPepper {
        ClaimTokenPepper::new("MaxVerstappenWC2021")
    }

    fn default_password_constraints() -> PasswordConstraints {
        PasswordConstraints::default()
    }

    fn default_password_reset_policy() -> ThrottlePolicy {
        ThrottlePolicy::default()
    }

    fn default_email_verification_policy() -> ThrottlePolicy {
        ThrottlePolicy::default()
    }
}

/// Exponential-backoff rate-limiting policy.
///
/// The backoff between successive attempts grows as
/// `min(base_backoff_secs * 2^(attempt - 1), max_backoff_secs)`.
///
/// After `max_attempts` are exhausted, the action is hard-locked until
/// the underlying condition is cleared (e.g. a successful password change
/// or email verification) or an admin intervenes.
///
/// Used by both password-reset and email-verification flows.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ThrottlePolicy {
    /// Base backoff interval in seconds.
    #[serde(default = "ThrottlePolicy::default_base_backoff_secs")]
    pub base_backoff_secs: u64,

    /// Maximum backoff ceiling in seconds.
    #[serde(default = "ThrottlePolicy::default_max_backoff_secs")]
    pub max_backoff_secs: u64,

    /// Hard cap on the number of attempts before the action is locked.
    #[serde(default = "ThrottlePolicy::default_max_attempts")]
    pub max_attempts: u32,
}

impl Default for ThrottlePolicy {
    fn default() -> Self {
        Self {
            base_backoff_secs: Self::default_base_backoff_secs(),
            max_backoff_secs: Self::default_max_backoff_secs(),
            max_attempts: Self::default_max_attempts(),
        }
    }
}

impl ThrottlePolicy {
    const fn default_base_backoff_secs() -> u64 {
        600 // 10 minutes
    }

    const fn default_max_backoff_secs() -> u64 {
        86_400 // 1 day
    }

    const fn default_max_attempts() -> u32 {
        5
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClaimTokenPepper(String);

impl ClaimTokenPepper {
    /// # Panics
    ///
    /// Will panic if the key if empty.
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

impl fmt::Display for ClaimTokenPepper {
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

#[cfg(test)]
mod tests {
    use super::ClaimTokenPepper;

    #[test]
    #[should_panic(expected = "secret key cannot be empty")]
    fn secret_key_can_not_be_empty() {
        drop(ClaimTokenPepper::new(""));
    }
}
