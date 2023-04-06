use serde::{Deserialize, Serialize};

pub type UserId = i64;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct User {
    pub user_id: UserId,
    pub date_registered: Option<String>,
    pub date_imported: Option<String>,
    pub administrator: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct UserAuthentication {
    pub user_id: UserId,
    pub password_hash: String,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct UserProfile {
    pub user_id: UserId,
    pub username: String,
    pub email: String,
    pub email_verified: bool,
    pub bio: String,
    pub avatar: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct UserCompact {
    pub user_id: UserId,
    pub username: String,
    pub administrator: bool,
}

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserClaims {
    pub user: UserCompact,
    pub exp: u64, // epoch in seconds
}
