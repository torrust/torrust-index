//! # `Tracker` / `ApiToken` direct tests
//!
//! `Tracker::override_tracker_api_token` and the public
//! accessors on `ApiToken` (`as_bytes`, `is_empty`, `Display`)
//! are part of the crate's API but were not exercised by any
//! other test file. This file pins their behaviour.
//!
//! | Test                                               | What it covers                            |
//! |----------------------------------------------------|-------------------------------------------|
//! | `override_tracker_api_token_replaces_value`        | Mutator swaps the inner token.            |
//! | `api_token_display_round_trips_through_format`     | `Display` matches the raw inner string.   |
//! | `api_token_as_bytes_matches_string_bytes`          | `as_bytes` is a zero-copy view.           |
//! | `api_token_is_empty_distinguishes_set_from_unset`  | The deserialiser-bypass guard branch.     |
//!
//! `ApiToken::new("")` panicking (the `assert!`) is already
//! pinned by `tests::quirks::empty_api_token_panics`.

use crate::tests::placeholder_settings;
use crate::v2::tracker::{ApiToken, Tracker};

#[test]
fn override_tracker_api_token_replaces_value() {
    let mut settings = placeholder_settings();
    let original = settings.tracker.token.to_string();

    let replacement = ApiToken::new("Replacement");
    settings.tracker.override_tracker_api_token(&replacement);

    assert_ne!(settings.tracker.token.to_string(), original);
    assert_eq!(settings.tracker.token.to_string(), "Replacement");
}

#[test]
fn override_tracker_api_token_is_idempotent_on_same_value() {
    let mut tracker = placeholder_settings().tracker;
    let pinned = ApiToken::new("Pinned");

    tracker.override_tracker_api_token(&pinned);
    let after_first = tracker.token.clone();
    tracker.override_tracker_api_token(&pinned);
    let after_second = tracker.token.clone();

    assert_eq!(after_first, after_second);
    assert_eq!(after_second, pinned);
}

#[test]
fn api_token_display_round_trips_through_format() {
    let token = ApiToken::new("hello-token");
    assert_eq!(format!("{token}"), "hello-token");
    assert_eq!(token.to_string(), "hello-token");
}

#[test]
fn api_token_as_bytes_matches_string_bytes() {
    let token = ApiToken::new("My-Token");
    assert_eq!(token.as_bytes(), b"My-Token");
}

#[test]
fn api_token_is_empty_distinguishes_set_from_unset() {
    // `ApiToken::new` rejects empty input, so the only path
    // that produces an empty token is `#[derive(Deserialize)]`
    // bypassing the `assert!`. Round-trip through the same
    // wire format the deserialiser sees.
    let nonempty: ApiToken = serde_json::from_str("\"set\"").expect("deser non-empty");
    let empty: ApiToken = serde_json::from_str("\"\"").expect("deser empty");
    assert!(!nonempty.is_empty());
    assert!(empty.is_empty());
}

#[test]
fn tracker_carries_default_urls_and_flags() {
    // Sanity-check the schema-level defaults pinned in
    // `Tracker`'s `default_*` fns: a `Tracker` materialised
    // through the loader must expose them verbatim.
    let tracker: &Tracker = &placeholder_settings().tracker;
    assert_eq!(tracker.api_url.as_str(), "http://localhost:1212/");
    assert_eq!(tracker.url.as_str(), "udp://localhost:6969");
    assert!(!tracker.listed);
    assert!(!tracker.private);
    assert_eq!(tracker.token_valid_seconds, 7_257_600);
}
