//! Quirky / edge-case tests that don't fit anywhere else.
//!
//! These exist to catch the sort of breakage that "normal" tests
//! never quite reach: empty-string TLS paths, IPv6 bind addresses,
//! emoji tracker tokens, the stubbornly-unsupported UDP-private
//! tracker, and the lone `Threshold` enum that pretends to be a
//! `LevelFilter`.
//!
//! ## Index
//!
//! | Test                                                 | What it proves                                                       |
//! |------------------------------------------------------|----------------------------------------------------------------------|
//! | `tls_empty_string_paths_deserialise_to_none`         | The `NoneAsEmptyString` adapter empties out → `None`.                |
//! | `tls_section_with_no_fields_deserialises_to_none`    | An empty `[net.tls]` table leaves both paths as `None`.              |
//! | `ipv6_bind_address_round_trips`                      | `[::1]:7000` survives parse → serialise → parse.                     |
//! | `tracker_token_accepts_unicode_grapheme_clusters`    | A 🦀-laden token loads and pretty-prints intact.                     |
//! | `tracker_validator_rejects_udp_private`              | The classic "you can't have a private UDP tracker" guard fires.      |
//! | `tracker_validator_accepts_https_private`            | …but HTTPS private trackers are fine.                                |
//! | `threshold_display_is_lowercase`                     | All six `Threshold` variants render in lowercase.                    |
//! | `threshold_levelfilter_conversion_is_total`          | Every `Threshold` maps to a distinct `LevelFilter`.                  |
//! | `empty_api_token_panics`                             | `ApiToken::new("")` is a contract violation.                         |
//! | `to_json_is_pretty_printed`                          | `Settings::to_json` emits indented JSON (not minified).              |

use std::collections::HashSet;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};

use tracing::level_filters::LevelFilter;
use url::Url;

use crate::tests::{MINIMUM_VALID_TOML, info_from, placeholder_settings};
use crate::v2::tracker::{ApiToken, Tracker};
use crate::validator::{ValidationError, Validator};
use crate::{Threshold, load_settings};

#[test]
fn tls_empty_string_paths_deserialise_to_none() {
    // Inject a `[net.tls]` section with empty paths on top of the minimum config.
    let toml = format!("{MINIMUM_VALID_TOML}\n[net.tls]\nssl_cert_path = \"\"\nssl_key_path = \"\"\n");
    let settings = load_settings(&info_from(&toml)).expect("TOML must load");
    let tls = settings.net.tls.expect("[net.tls] table must materialise as Some");
    assert!(tls.ssl_cert_path.is_none(), "empty string must collapse to None");
    assert!(tls.ssl_key_path.is_none(), "empty string must collapse to None");
}

#[test]
fn tls_section_with_no_fields_deserialises_to_none() {
    // A `[net.tls]` table with no fields at all must not synthesise empty
    // paths — otherwise `make_rust_tls` would later try to load a cert
    // from `""` and emit a misleading "no such file" error at startup.
    let toml = format!("{MINIMUM_VALID_TOML}\n[net.tls]\n");
    let settings = load_settings(&info_from(&toml)).expect("TOML must load");
    let tls = settings.net.tls.expect("[net.tls] table must materialise as Some");
    assert!(tls.ssl_cert_path.is_none(), "missing field must default to None");
    assert!(tls.ssl_key_path.is_none(), "missing field must default to None");
}

#[test]
fn ipv6_bind_address_round_trips() {
    let mut s = placeholder_settings();
    s.net.bind_address = SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 7000);
    let toml = s.to_toml();
    let reloaded = load_settings(&info_from(&toml)).expect("IPv6 bind address must round-trip");
    assert_eq!(reloaded.net.bind_address, s.net.bind_address);
}

#[test]
fn tracker_token_accepts_unicode_grapheme_clusters() {
    let token = ApiToken::new("🦀-token-📦-ñoño");
    assert_eq!(token.to_string(), "🦀-token-📦-ñoño");
    // Bytes match the UTF-8 of the input.
    assert_eq!(token.as_bytes(), "🦀-token-📦-ñoño".as_bytes());
}

#[test]
fn tracker_validator_rejects_udp_private() {
    let t = Tracker {
        api_url: Url::parse("http://localhost:1212/").unwrap(),
        listed: false,
        private: true,
        token: ApiToken::new("MyAccessToken"),
        token_valid_seconds: 7_257_600,
        url: Url::parse("udp://localhost:6969").unwrap(),
    };
    match t.validate() {
        Err(ValidationError::UdpTrackersInPrivateModeNotSupported) => {}
        other => panic!("expected UdpTrackersInPrivateModeNotSupported, got {other:?}"),
    }
}

#[test]
fn tracker_validator_accepts_https_private() {
    let t = Tracker {
        api_url: Url::parse("http://localhost:1212/").unwrap(),
        listed: false,
        private: true,
        token: ApiToken::new("MyAccessToken"),
        token_valid_seconds: 7_257_600,
        url: Url::parse("https://tracker.example.com/announce").unwrap(),
    };
    t.validate().expect("HTTPS private trackers are allowed");
}

#[test]
fn threshold_display_is_lowercase() {
    let pairs = [
        (Threshold::Off, "off"),
        (Threshold::Error, "error"),
        (Threshold::Warn, "warn"),
        (Threshold::Info, "info"),
        (Threshold::Debug, "debug"),
        (Threshold::Trace, "trace"),
    ];
    for (t, expected) in pairs {
        assert_eq!(t.to_string(), expected);
    }
}

#[test]
fn threshold_levelfilter_conversion_is_total() {
    let mapped: HashSet<LevelFilter> = [
        Threshold::Off,
        Threshold::Error,
        Threshold::Warn,
        Threshold::Info,
        Threshold::Debug,
        Threshold::Trace,
    ]
    .into_iter()
    .map(LevelFilter::from)
    .collect();
    assert_eq!(mapped.len(), 6, "every Threshold must map to a unique LevelFilter");
}

#[test]
#[should_panic(expected = "tracker API token cannot be empty")]
fn empty_api_token_panics() {
    drop(ApiToken::new(""));
}

#[test]
fn to_json_is_pretty_printed() {
    let json = placeholder_settings().to_json();
    assert!(json.contains('\n'), "to_json should be pretty-printed (multi-line)");
    assert!(json.starts_with('{'));
    assert!(json.trim_end().ends_with('}'));
}
