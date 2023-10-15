use serde::{Deserialize, Serialize};

// Registration

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RegistrationForm {
    pub username: String,
    pub email: Option<String>,
    pub password: String,
    pub confirm_password: String,
}

// Authentication

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LoginForm {
    pub login: String, // todo: rename to `username`
    pub password: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JsonWebToken {
    pub token: String, // // todo: rename to `encoded` or `value`
}
