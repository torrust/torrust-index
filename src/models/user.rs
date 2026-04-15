use std::fmt;
use std::str::FromStr;

use regex::Regex;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[allow(clippy::module_name_repetitions)]
pub type UserId = i64;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct User {
    pub user_id: UserId,
    pub date_registered: Option<String>,
    pub date_imported: Option<String>,
    pub role: String,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct UserAuthentication {
    pub user_id: UserId,
    pub password_hash: String,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct UserProfile {
    pub user_id: UserId,
    pub username: String,
    pub email: String,
    pub email_verified: bool,
    pub bio: String,
    pub avatar: String,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct UserCompact {
    pub user_id: UserId,
    pub username: String,
    pub role: String,
}

impl UserCompact {
    /// Whether this user has the admin role.
    ///
    /// Convenience for backward-compatible API responses during the
    /// deprecation period (ADR-T-008 Phase 1).
    #[must_use]
    pub fn is_admin(&self) -> bool {
        self.role == "admin"
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct UserFull {
    pub user_id: UserId,
    pub date_registered: Option<String>,
    pub date_imported: Option<String>,
    pub role: String,
    pub username: String,
    pub email: String,
    pub email_verified: bool,
    pub bio: String,
    pub avatar: String,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct UserListing {
    pub user_id: UserId,
    pub username: String,
    pub email: String,
    pub email_verified: bool,
    pub date_registered: String,
    pub role: String,
}

pub(crate) const MAX_USERNAME_LENGTH: usize = 20;
const USERNAME_VALIDATION_ERROR_MSG: &str = "Usernames must consist of 1-20 alphanumeric characters, dashes, or underscore";

#[derive(Debug, Clone, Error)]
#[error("UsernameParseError: {message}")]
pub struct UsernameParseError {
    message: String,
}

pub struct Username(String);

impl Username {
    /// Creates a `Username` directly without validation.
    /// Intended for test helpers and internal use.
    #[cfg(test)]
    pub(crate) fn new(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl fmt::Display for Username {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Implement the parsing logic
impl FromStr for Username {
    type Err = UsernameParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() > MAX_USERNAME_LENGTH {
            return Err(UsernameParseError {
                message: format!("username '{s}' is too long. {USERNAME_VALIDATION_ERROR_MSG}."),
            });
        }

        let pattern = format!(r"^[A-Za-z0-9-_]{{1,{MAX_USERNAME_LENGTH}}}$");
        let re = Regex::new(&pattern).expect("username regexp should be valid");

        if re.is_match(s) {
            Ok(Self(s.to_string()))
        } else {
            Err(UsernameParseError {
                message: format!("'{s}' is not a valid username. {USERNAME_VALIDATION_ERROR_MSG}."),
            })
        }
    }
}
