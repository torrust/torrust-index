use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RegistrationForm {
    pub username: String,
    pub email: Option<String>,
    pub password: String,
    pub confirm_password: String,
}
