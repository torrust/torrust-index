use crate::config::v2::auth::ClaimTokenPepper;

#[test]
#[should_panic(expected = "secret key cannot be empty")]
fn secret_key_can_not_be_empty() {
    drop(ClaimTokenPepper::new(""));
}
