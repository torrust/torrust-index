use std::fmt;

use serde::{Deserialize, Serialize};

/// Authentication options.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Auth {
    /// Whether or not to require an email on signup.
    pub email_on_signup: EmailOnSignup,
    /// The minimum password length.
    pub min_password_length: usize,
    /// The maximum password length.
    pub max_password_length: usize,
    /// The secret key used to sign JWT tokens.
    pub secret_key: SecretKey,
}

impl Default for Auth {
    fn default() -> Self {
        Self {
            email_on_signup: EmailOnSignup::default(),
            min_password_length: 6,
            max_password_length: 64,
            secret_key: SecretKey::new("MaxVerstappenWC2021"),
        }
    }
}

impl Auth {
    pub fn override_secret_key(&mut self, secret_key: &str) {
        self.secret_key = SecretKey::new(secret_key);
    }
}

/// Whether the email is required on signup or not.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EmailOnSignup {
    /// The email is required on signup.
    Required,
    /// The email is optional on signup.
    Optional,
    /// The email is not allowed on signup. It will only be ignored if provided.
    None, // code-review: rename to `Ignored`?
}

impl Default for EmailOnSignup {
    fn default() -> Self {
        Self::Optional
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

#[cfg(test)]
mod tests {
    use super::SecretKey;

    #[test]
    #[should_panic(expected = "secret key cannot be empty")]
    fn secret_key_can_not_be_empty() {
        drop(SecretKey::new(""));
    }
}
