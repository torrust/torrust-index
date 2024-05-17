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
    pub secret_key: String,
}

impl Default for Auth {
    fn default() -> Self {
        Self {
            email_on_signup: EmailOnSignup::default(),
            min_password_length: 6,
            max_password_length: 64,
            secret_key: "MaxVerstappenWC2021".to_string(),
        }
    }
}

impl Auth {
    pub fn override_secret_key(&mut self, secret_key: &str) {
        self.secret_key = secret_key.to_string();
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
