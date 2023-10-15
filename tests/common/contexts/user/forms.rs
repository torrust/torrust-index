use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct RegistrationForm {
    pub username: String,
    pub email: Option<String>,
    pub password: String,
    pub confirm_password: String,
}

pub type RegisteredUser = RegistrationForm;

#[derive(Serialize)]
pub struct LoginForm {
    pub login: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct TokenVerificationForm {
    pub token: String,
}

#[derive(Serialize)]
pub struct TokenRenewalForm {
    pub token: String,
}

pub struct Username {
    pub value: String,
}

impl Username {
    pub fn new(value: String) -> Self {
        Self { value }
    }
}
