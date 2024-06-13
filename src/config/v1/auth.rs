use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Authentication options.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Auth {
    /// Whether or not to require an email on signup.
    #[serde(default = "Auth::default_email_on_signup")]
    pub email_on_signup: EmailOnSignup,

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
            email_on_signup: EmailOnSignup::default(),
            password_constraints: Self::default_password_constraints(),
            secret_key: Self::default_secret_key(),
        }
    }
}

impl Auth {
    pub fn override_secret_key(&mut self, secret_key: &str) {
        self.secret_key = SecretKey::new(secret_key);
    }

    fn default_email_on_signup() -> EmailOnSignup {
        EmailOnSignup::default()
    }

    fn default_secret_key() -> SecretKey {
        SecretKey::new("MaxVerstappenWC2021")
    }

    fn default_password_constraints() -> PasswordConstraints {
        PasswordConstraints::default()
    }
}

/// Whether the email is required on signup or not.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum EmailOnSignup {
    /// The email is required on signup.
    Required,
    /// The email is optional on signup.
    Optional,
    /// The email is not allowed on signup. It will only be ignored if provided.
    Ignored,
}

impl Default for EmailOnSignup {
    fn default() -> Self {
        Self::Optional
    }
}

impl fmt::Display for EmailOnSignup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display_str = match self {
            EmailOnSignup::Required => "required",
            EmailOnSignup::Optional => "optional",
            EmailOnSignup::Ignored => "ignored",
        };
        write!(f, "{display_str}")
    }
}

impl FromStr for EmailOnSignup {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "required" => Ok(EmailOnSignup::Required),
            "optional" => Ok(EmailOnSignup::Optional),
            "none" => Ok(EmailOnSignup::Ignored),
            _ => Err(format!(
                "Unknown config 'email_on_signup' option (required, optional, ignored): {s}"
            )),
        }
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
