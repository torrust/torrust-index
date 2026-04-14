use crate::config::v2::auth::JwtSigningSecret;

#[test]
#[should_panic(expected = "secret key cannot be empty")]
fn secret_key_can_not_be_empty() {
    drop(JwtSigningSecret::new(""));
}

#[test]
#[should_panic(expected = "secret key must be at least 32 bytes")]
fn secret_key_must_meet_minimum_length() {
    drop(JwtSigningSecret::new("too-short"));
}

#[test]
fn secret_key_accepts_valid_length() {
    let key = JwtSigningSecret::new("a]32-byte-minimum-length-secret!");
    assert_eq!(key.as_bytes().len(), 32);
}
