use lettre::message::Mailbox;
use serde::{Deserialize, Serialize};

/// SMTP configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Mail {
    /// Whether or not to enable email verification on signup.
    #[serde(default = "Mail::default_email_verification_enabled")]
    pub email_verification_enabled: bool,
    /// The email address to send emails from.
    #[serde(default = "Mail::default_from")]
    pub from: Mailbox,
    /// The email address to reply to.
    #[serde(default = "Mail::default_reply_to")]
    pub reply_to: Mailbox,
    /// The username to use for SMTP authentication.
    #[serde(default = "Mail::default_username")]
    pub username: String,
    /// The password to use for SMTP authentication.
    #[serde(default = "Mail::default_password")]
    pub password: String,
    /// The SMTP server to use.
    #[serde(default = "Mail::default_server")]
    pub server: String,
    /// The SMTP port to use.
    #[serde(default = "Mail::default_port")]
    pub port: u16,
}

impl Default for Mail {
    fn default() -> Self {
        Self {
            email_verification_enabled: Self::default_email_verification_enabled(),
            from: Self::default_from(),
            reply_to: Self::default_reply_to(),
            username: Self::default_username(),
            password: Self::default_password(),
            server: Self::default_server(),
            port: Self::default_port(),
        }
    }
}

impl Mail {
    fn default_email_verification_enabled() -> bool {
        false
    }

    fn default_from() -> Mailbox {
        "example@email.com".parse().expect("valid mailbox")
    }

    fn default_reply_to() -> Mailbox {
        "noreply@email.com".parse().expect("valid mailbox")
    }

    fn default_username() -> String {
        String::default()
    }

    fn default_password() -> String {
        String::default()
    }

    fn default_server() -> String {
        String::default()
    }

    fn default_port() -> u16 {
        25
    }
}
