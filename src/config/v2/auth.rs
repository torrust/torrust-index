use std::fmt;

use serde::{Deserialize, Serialize};

/// Authentication options.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Auth {
    /// The secret key used to sign JWT tokens.
    #[serde(default = "Auth::default_secret_key")]
    pub secret_key: SecretKey,

    /// The password constraints
    #[serde(default = "Auth::default_password_constraints")]
    pub password_constraints: PasswordConstraints,
}

impl Default for Auth {
    fn default() -> Self {
        Self {
            password_constraints: Self::default_password_constraints(),
            secret_key: Self::default_secret_key(),
        }
    }
}

impl Auth {
    pub fn override_secret_key(&mut self, secret_key: &str) {
        self.secret_key = SecretKey::new(secret_key);
    }

    fn default_secret_key() -> SecretKey {
        SecretKey::new("MaxVerstappenWC2021")
    }

    fn default_password_constraints() -> PasswordConstraints {
        PasswordConstraints::default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SecretKey(String);

impl SecretKey {
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

impl fmt::Display for SecretKey {
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
    use super::SecretKey;

    #[test]
    #[should_panic(expected = "secret key cannot be empty")]
    fn secret_key_can_not_be_empty() {
        drop(SecretKey::new(""));
    }
}
