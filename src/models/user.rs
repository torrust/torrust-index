use std::fmt;
use std::str::FromStr;

use regex::Regex;
use serde::{Deserialize, Serialize};

#[allow(clippy::module_name_repetitions)]
pub type UserId = i64;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct User {
    pub user_id: UserId,
    pub date_registered: Option<String>,
    pub date_imported: Option<String>,
    pub administrator: bool,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct UserAuthentication {
    pub user_id: UserId,
    pub password_hash: String,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct UserAuthorization {
    pub user_id: UserId,
    pub administrator: bool,
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
    pub administrator: bool,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct UserFull {
    pub user_id: UserId,
    pub date_registered: Option<String>,
    pub date_imported: Option<String>,
    pub administrator: bool,
    pub username: String,
    pub email: String,
    pub email_verified: bool,
    pub bio: String,
    pub avatar: String,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserClaims {
    pub user: UserCompact,
    pub exp: u64, // epoch in seconds
}

const MAX_USERNAME_LENGTH: usize = 20;
const USERNAME_VALIDATION_ERROR_MSG: &str = "Usernames must consist of 1-20 alphanumeric characters, dashes, or underscore";

#[derive(Debug, Clone)]
pub struct UsernameParseError {
    message: String,
}

// Implement std::fmt::Display for UsernameParseError
impl fmt::Display for UsernameParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UsernameParseError: {}", self.message)
    }
}

// Implement std::error::Error for UsernameParseError
impl std::error::Error for UsernameParseError {}

pub struct Username(String);

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
            Ok(Username(s.to_string()))
        } else {
            Err(UsernameParseError {
                message: format!("'{s}' is not a valid username. {USERNAME_VALIDATION_ERROR_MSG}."),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn username_must_consist_of_1_to_20_alphanumeric_characters_or_dashes() {
        let username_str = "validUsername123";
        assert!(username_str.parse::<Username>().is_ok());
    }

    #[test]
    fn username_should_be_shorter_then_21_chars() {
        let username_str = "a".repeat(MAX_USERNAME_LENGTH + 1);
        assert!(username_str.parse::<Username>().is_err());
    }

    #[test]
    fn username_should_not_allow_invalid_characters() {
        let username_str = "invalid*Username";
        assert!(username_str.parse::<Username>().is_err());
    }

    #[test]
    fn username_should_be_displayed() {
        let username = Username("FirstLast-01".to_string());
        assert_eq!(username.to_string(), "FirstLast-01");
    }
}
