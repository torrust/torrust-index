//! Integration tests: full TOML / JSON round-trips through the
//! crate's *public* API only. Nothing here may import private items.
//!
//! ## Index
//!
//! | Test                                               | What it proves                                                  |
//! |----------------------------------------------------|-----------------------------------------------------------------|
//! | `defaults_round_trip_through_toml`                 | `Settings::default() → to_toml → load_settings` is the identity.|
//! | `defaults_round_trip_through_json_then_toml`       | JSON survives a re-serialise via TOML (proves shape symmetry).  |
//! | `two_pass_toml_is_a_fixed_point`                   | After one TOML round-trip, the next round-trip is byte-stable.  |
//! | `customised_settings_survive_round_trip`           | A non-default `Settings` (custom token, URLs, page sizes) too.  |

use torrust_index_config::{Info, Settings, load_settings};

#[test]
fn defaults_round_trip_through_toml() {
    let original = Settings::default();
    let toml = original.to_toml();
    let reloaded = load_settings(&Info::from_toml(&toml)).expect("default Settings must round-trip");
    assert_eq!(reloaded, original);
}

#[test]
fn defaults_round_trip_through_json_then_toml() {
    let original = Settings::default();
    let json = original.to_json();
    let from_json: Settings = serde_json::from_str(&json).expect("JSON must deserialise back to Settings");
    assert_eq!(from_json, original);

    // And what JSON gives us, TOML must accept too.
    let toml = from_json.to_toml();
    let reloaded = load_settings(&Info::from_toml(&toml)).expect("JSON→Settings→TOML must reload");
    assert_eq!(reloaded, original);
}

#[test]
fn two_pass_toml_is_a_fixed_point() {
    let original = Settings::default();
    let pass_1 = original.to_toml();
    let after_1: Settings = load_settings(&Info::from_toml(&pass_1)).unwrap();
    let pass_2 = after_1.to_toml();
    // The serialised TOML must be identical between successive passes
    // (i.e. there is no information loss after the first round-trip).
    assert_eq!(pass_1, pass_2, "to_toml must be a fixed point after one load cycle");
}

#[test]
fn customised_settings_survive_round_trip() {
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    use url::Url;

    let mut original = Settings::default();
    original.tracker.token = torrust_index_config::ApiToken::new("a-very-different-token");
    original.tracker.api_url = Url::parse("https://tracker.example.org:1212/").unwrap();
    original.tracker.url = Url::parse("https://tracker.example.org/announce").unwrap();
    original.net.bind_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 4242);
    original.api.default_torrent_page_size = 25;
    original.api.max_torrent_page_size = 250;
    original.website.name = "My Curated Index".to_owned();

    let toml = original.to_toml();
    let reloaded = load_settings(&Info::from_toml(&toml)).expect("customised Settings must round-trip");
    assert_eq!(reloaded, original);
}
