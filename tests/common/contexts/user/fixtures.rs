use rand::Rng;

use crate::common::contexts::user::forms::RegistrationForm;

/// Default password used in tests
pub const DEFAULT_PASSWORD: &str = "password";

/// Sample valid password used in tests
pub const VALID_PASSWORD: &str = "12345678";

pub fn random_user_registration_form() -> RegistrationForm {
    let user_id = random_user_id();
    RegistrationForm {
        username: format!("username_{user_id}"),
        email: Some(format!("email_{user_id}@email.com")),
        password: DEFAULT_PASSWORD.to_string(),
        confirm_password: DEFAULT_PASSWORD.to_string(),
    }
}

fn random_user_id() -> u64 {
    let mut rng = rand::thread_rng();
    rng.gen_range(0..1_000_000)
}
