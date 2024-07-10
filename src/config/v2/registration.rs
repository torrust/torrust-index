use serde::{Deserialize, Serialize};

/// SMTP configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Registration {
    /// Whether or not to enable email verification on signup.
    #[serde(default = "Registration::default_email")]
    pub email: Option<Email>,
}

impl Default for Registration {
    fn default() -> Self {
        Self {
            email: Self::default_email(),
        }
    }
}

impl Registration {
    fn default_email() -> Option<Email> {
        None
    }
}

/// SMTP configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Email {
    /// Whether or not email is required on signup.
    #[serde(default = "Email::default_required")]
    pub required: bool,

    /// Whether or not email is verified.
    #[serde(default = "Email::default_verified")]
    pub verified: bool,
}

impl Default for Email {
    fn default() -> Self {
        Self {
            required: Self::default_required(),
            verified: Self::default_verified(),
        }
    }
}

impl Email {
    fn default_required() -> bool {
        false
    }

    fn default_verified() -> bool {
        false
    }
}
