//! Tests for `config::v2::auth::Auth` — RSA key resolution.
//!
//! ## Index
//!
//! - `resolve_private_key_pem_from_inline` — inline PEM takes priority.
//! - `resolve_public_key_pem_from_inline` — inline PEM takes priority.
//! - `resolve_private_key_pem_returns_none_when_no_key` — returns `None` if
//!   no key is configured (ephemeral generation is handled by `JsonWebToken`).

use crate::config::v2::auth::Auth;

#[test]
fn resolve_private_key_pem_from_inline() {
    let auth = Auth {
        private_key_pem: Some("-----BEGIN PRIVATE KEY-----\nfake\n-----END PRIVATE KEY-----\n".to_owned()),
        private_key_path: None,
        ..Auth::default()
    };
    let pem = auth.resolve_private_key_pem().expect("should resolve inline PEM");
    assert!(pem.starts_with(b"-----BEGIN PRIVATE KEY-----"));
}

#[test]
fn resolve_public_key_pem_from_inline() {
    let auth = Auth {
        public_key_pem: Some("-----BEGIN PUBLIC KEY-----\nfake\n-----END PUBLIC KEY-----\n".to_owned()),
        public_key_path: None,
        ..Auth::default()
    };
    let pem = auth.resolve_public_key_pem().expect("should resolve inline PEM");
    assert!(pem.starts_with(b"-----BEGIN PUBLIC KEY-----"));
}

#[test]
fn resolve_private_key_pem_returns_none_when_no_key() {
    let auth = Auth {
        private_key_pem: None,
        private_key_path: None,
        ..Auth::default()
    };
    assert!(auth.resolve_private_key_pem().is_none());
}
