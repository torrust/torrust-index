use crate::models::user::{Username, MAX_USERNAME_LENGTH};

#[test]
fn username_must_consist_of_1_to_20_alphanumeric_characters_or_dashes() {
    let username_str = "validUsername123";
    assert!(username_str.parse::<Username>().is_ok());
}

#[test]
fn username_should_be_shorter_then_21_chars() {
    let username_str = "a".repeat(MAX_USERNAME_LENGTH + 1);
    assert!(username_str.parse::<Username>().is_err());
}

#[test]
fn username_should_not_allow_invalid_characters() {
    let username_str = "invalid*Username";
    assert!(username_str.parse::<Username>().is_err());
}

#[test]
fn username_should_be_displayed() {
    let username = Username::new("FirstLast-01");
    assert_eq!(username.to_string(), "FirstLast-01");
}
