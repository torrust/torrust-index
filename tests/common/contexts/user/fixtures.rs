use rand::Rng;

use crate::common::contexts::user::forms::RegistrationForm;

pub fn random_user_registration() -> RegistrationForm {
    let user_id = random_user_id();
    RegistrationForm {
        username: format!("username_{user_id}"),
        email: Some(format!("email_{user_id}@email.com")),
        password: "password".to_string(),
        confirm_password: "password".to_string(),
    }
}

fn random_user_id() -> u64 {
    let mut rng = rand::thread_rng();
    rng.gen_range(0..1_000_000)
}
