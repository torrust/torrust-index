use serde::{Serialize, Deserialize};
use sqlx::query_as;

#[allow(dead_code)]
pub struct User {
    pub user_id: i64,
    pub username: String,
    pub email: String,
    pub email_verified: bool,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // username
    pub exp: u64, // epoch in seconds
}
