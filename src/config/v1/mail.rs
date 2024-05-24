use lettre::message::Mailbox;
use serde::{Deserialize, Serialize};

/// SMTP configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Mail {
    /// Whether or not to enable email verification on signup.
    pub email_verification_enabled: bool,
    /// The email address to send emails from.
    pub from: Mailbox,
    /// The email address to reply to.
    pub reply_to: Mailbox,
    /// The username to use for SMTP authentication.
    pub username: String,
    /// The password to use for SMTP authentication.
    pub password: String,
    /// The SMTP server to use.
    pub server: String,
    /// The SMTP port to use.
    pub port: u16,
}

impl Default for Mail {
    fn default() -> Self {
        Self {
            email_verification_enabled: false,
            from: "example@email.com".parse().expect("valid mailbox"),
            reply_to: "noreply@email.com".parse().expect("valid mailbox"),
            username: String::default(),
            password: String::default(),
            server: String::default(),
            port: 25,
        }
    }
}
