use std::fmt;

use serde::{Deserialize, Serialize};

/// Authentication options.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Auth {
    /// The secret key used to sign JWT tokens.
    #[serde(default = "Auth::default_user_claim_token_pepper")]
    pub user_claim_token_pepper: ClaimTokenPepper,

    /// The password constraints
    #[serde(default = "Auth::default_password_constraints")]
    pub password_constraints: PasswordConstraints,
}

impl Default for Auth {
    fn default() -> Self {
        Self {
            password_constraints: Self::default_password_constraints(),
            user_claim_token_pepper: Self::default_user_claim_token_pepper(),
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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    fn default_min_password_length() -> usize {
        6
    }

    fn default_max_password_length() -> usize {
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
