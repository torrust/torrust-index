//! Crate tests for the `parse_token` and `BearerToken` boundary
//! logic (`src/web/api/server/v1/auth.rs` and
//! `src/web/api/server/v1/extractors/bearer_token.rs`).
//!
//! # Test index
//!
//! ## `parse_token`
//!
//! - [`parse_valid_bearer_token`] — extracts the token string from a
//!   well-formed `Bearer <token>` header.
//! - [`parse_bearer_token_trims_whitespace`] — leading/trailing
//!   whitespace around the token value is stripped.
//! - [`parse_rejects_empty_bearer`] — `"Bearer "` with no token →
//!   `TokenInvalid`.
//! - [`parse_rejects_missing_bearer_prefix`] — a header without the
//!   `Bearer` prefix → `TokenInvalid`.
//! - [`parse_rejects_non_ascii_header`] — non-UTF-8 header bytes →
//!   `TokenInvalid`.

use hyper::http::HeaderValue;

use crate::errors::AuthError;
use crate::web::api::server::v1::auth::parse_token;

#[test]
fn parse_valid_bearer_token() {
    let header = HeaderValue::from_static("Bearer eyJhbGciOiJSUzI1NiJ9.test.sig");
    let token = parse_token(&header).unwrap();
    assert_eq!(token, "eyJhbGciOiJSUzI1NiJ9.test.sig");
}

#[test]
fn parse_bearer_token_trims_whitespace() {
    let header = HeaderValue::from_static("Bearer   my_token   ");
    let token = parse_token(&header).unwrap();
    assert_eq!(token, "my_token");
}

#[test]
fn parse_rejects_empty_bearer() {
    let header = HeaderValue::from_static("Bearer ");
    let err = parse_token(&header).unwrap_err();
    assert_eq!(err, AuthError::TokenInvalid);
}

#[test]
fn parse_rejects_missing_bearer_prefix() {
    let header = HeaderValue::from_static("Basic abc123");
    let err = parse_token(&header).unwrap_err();
    assert_eq!(err, AuthError::TokenInvalid);
}

#[test]
fn parse_rejects_non_ascii_header() {
    let header = HeaderValue::from_bytes(b"Bearer \xff\xfe").unwrap();
    let err = parse_token(&header).unwrap_err();
    assert_eq!(err, AuthError::TokenInvalid);
}
