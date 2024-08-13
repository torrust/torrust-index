use lettre::message::Mailbox;
use serde::{Deserialize, Serialize};

/// SMTP configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Mail {
    /// The email address to send emails from.
    #[serde(default = "Mail::default_from")]
    pub from: Mailbox,

    /// The email address to reply to.
    #[serde(default = "Mail::default_reply_to")]
    pub reply_to: Mailbox,

    /// The SMTP server configuration.
    #[serde(default = "Mail::default_smtp")]
    pub smtp: Smtp,
}

impl Default for Mail {
    fn default() -> Self {
        Self {
            from: Self::default_from(),
            reply_to: Self::default_reply_to(),
            smtp: Self::default_smtp(),
        }
    }
}

impl Mail {
    fn default_from() -> Mailbox {
        "example@email.com".parse().expect("valid mailbox")
    }

    fn default_reply_to() -> Mailbox {
        "noreply@email.com".parse().expect("valid mailbox")
    }

    fn default_smtp() -> Smtp {
        Smtp::default()
    }
}

/// SMTP configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Smtp {
    /// The SMTP port to use.
    #[serde(default = "Smtp::default_port")]
    pub port: u16,
    /// The SMTP server to use.
    #[serde(default = "Smtp::default_server")]
    pub server: String,
    /// The SMTP server credentials.
    #[serde(default = "Smtp::default_credentials")]
    pub credentials: Credentials,
}

impl Default for Smtp {
    fn default() -> Self {
        Self {
            server: Self::default_server(),
            port: Self::default_port(),
            credentials: Self::default_credentials(),
        }
    }
}

impl Smtp {
    fn default_server() -> String {
        String::default()
    }

    fn default_port() -> u16 {
        25
    }

    fn default_credentials() -> Credentials {
        Credentials::default()
    }
}

/// SMTP configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Credentials {
    /// The password to use for SMTP authentication.
    #[serde(default = "Credentials::default_password")]
    pub password: String,
    /// The username to use for SMTP authentication.
    #[serde(default = "Credentials::default_username")]
    pub username: String,
}

impl Default for Credentials {
    fn default() -> Self {
        Self {
            username: Self::default_username(),
            password: Self::default_password(),
        }
    }
}

impl Credentials {
    fn default_username() -> String {
        String::default()
    }

    fn default_password() -> String {
        String::default()
    }
}
