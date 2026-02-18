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

// Profile

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChangePasswordForm {
    pub current_password: String,
    pub password: String,
    pub confirm_password: String,
}

// Email verification resend

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResendVerificationForm {
    pub email: String,
}

// Password reset

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SendPasswordLinkForm {
    pub email: String,
}

// Password reset completion

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResetPasswordForm {
    pub token: String,
    pub password: String,
    pub confirm_password: String,
}
