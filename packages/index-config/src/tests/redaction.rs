//! Tests for [`crate::Settings::remove_secrets`].
//!
//! Every secret-bearing slot listed in the source must end up
//! redacted; non-secret fields must be untouched.
//!
//! ## Index
//!
//! | Test                                          | What it proves                                          |
//! |-----------------------------------------------|---------------------------------------------------------|
//! | `tracker_token_is_replaced_with_stars`        | Tracker API token becomes `***`.                        |
//! | `database_password_is_replaced_with_stars`    | The password component of the connect URL is masked.    |
//! | `database_url_without_password_is_unchanged`  | URLs that have nothing to redact survive verbatim.      |
//! | `smtp_password_is_replaced_with_stars`        | SMTP credentials password becomes `***`.                |
//! | `auth_keys_are_redacted_when_set`             | Inline PEMs and key paths are individually redacted.    |
//! | `non_secret_fields_are_preserved`             | Logging threshold, network bind address, etc. survive.  |

use std::net::SocketAddr;

use url::Url;

use crate::Settings;

#[test]
fn tracker_token_is_replaced_with_stars() {
    let mut s = Settings::default();
    let mut redacted = s.clone();
    redacted.remove_secrets();
    assert_ne!(s.tracker.token.to_string(), redacted.tracker.token.to_string());
    assert_eq!(redacted.tracker.token.to_string(), "***");
    let _ = &mut s; // silence unused-mut suggestion if any
}

#[test]
fn database_password_is_replaced_with_stars() {
    let mut s = Settings::default();
    s.database.connect_url = Url::parse("mysql://root:hunter2@db:3306/idx").unwrap();
    s.remove_secrets();
    assert_eq!(s.database.connect_url.password(), Some("***"));
    assert_eq!(s.database.connect_url.username(), "root");
}

#[test]
fn database_url_without_password_is_unchanged() {
    let mut s = Settings::default();
    s.database.connect_url = Url::parse("sqlite://data.db?mode=rwc").unwrap();
    let before = s.database.connect_url.clone();
    s.remove_secrets();
    assert_eq!(s.database.connect_url, before);
}

#[test]
fn smtp_password_is_replaced_with_stars() {
    let mut s = Settings::default();
    s.mail.smtp.credentials.password = "super-secret".to_owned();
    s.remove_secrets();
    assert_eq!(s.mail.smtp.credentials.password, "***");
}

#[test]
fn auth_keys_are_redacted_when_set() {
    let mut s = Settings::default();
    s.auth.private_key_pem = Some("-----BEGIN PRIVATE KEY-----\nAAAA\n-----END PRIVATE KEY-----".into());
    s.auth.public_key_pem = Some("-----BEGIN PUBLIC KEY-----\nBBBB\n-----END PUBLIC KEY-----".into());
    s.auth.private_key_path = Some("/etc/torrust/jwt.key".into());
    s.auth.public_key_path = Some("/etc/torrust/jwt.pub".into());

    s.remove_secrets();

    assert_eq!(s.auth.private_key_pem.as_deref(), Some("***-redacted-private-key-pem***"));
    assert_eq!(s.auth.public_key_pem.as_deref(), Some("***-redacted-public-key-pem***"));
    assert_eq!(s.auth.private_key_path.as_deref(), Some("***-redacted***"));
    assert_eq!(s.auth.public_key_path.as_deref(), Some("***-redacted***"));
}

#[test]
fn non_secret_fields_are_preserved() {
    let mut s = Settings::default();
    let original_bind: SocketAddr = s.net.bind_address;
    let original_threshold = s.logging.threshold.clone();
    s.remove_secrets();
    assert_eq!(s.net.bind_address, original_bind);
    assert_eq!(s.logging.threshold, original_threshold);
}
