use url::Url;

use crate::tracker::service::build_announce_url_with_key;

#[test]
fn test_build_announce_url_with_key_with_announce_path() {
    let base_url = Url::parse("https://127.0.0.1:7070/announce").unwrap();
    let tracker_key = "mCGfCr8nvixxA0h8B4iz0sT8V3FIQLi7";

    let result = build_announce_url_with_key(&base_url, tracker_key);

    assert_eq!(
        result.to_string(),
        "https://127.0.0.1:7070/announce/mCGfCr8nvixxA0h8B4iz0sT8V3FIQLi7"
    );
}

#[test]
fn test_build_announce_url_with_key_with_trailing_slash() {
    let base_url = Url::parse("https://127.0.0.1:7070/announce/").unwrap();
    let tracker_key = "mCGfCr8nvixxA0h8B4iz0sT8V3FIQLi7";

    let result = build_announce_url_with_key(&base_url, tracker_key);

    assert_eq!(
        result.to_string(),
        "https://127.0.0.1:7070/announce/mCGfCr8nvixxA0h8B4iz0sT8V3FIQLi7"
    );
}

#[test]
fn test_build_announce_url_with_key_root_path() {
    let base_url = Url::parse("https://tracker.example.com/").unwrap();
    let tracker_key = "testkey123";

    let result = build_announce_url_with_key(&base_url, tracker_key);

    assert_eq!(result.to_string(), "https://tracker.example.com/testkey123");
}

#[test]
fn test_build_announce_url_with_key_no_path() {
    let base_url = Url::parse("https://tracker.example.com").unwrap();
    let tracker_key = "testkey123";

    let result = build_announce_url_with_key(&base_url, tracker_key);

    assert_eq!(result.to_string(), "https://tracker.example.com/testkey123");
}

#[test]
fn test_build_announce_url_with_key_multiple_path_segments() {
    let base_url = Url::parse("https://tracker.example.com/api/v1/announce").unwrap();
    let tracker_key = "complexkey456";

    let result = build_announce_url_with_key(&base_url, tracker_key);

    assert_eq!(
        result.to_string(),
        "https://tracker.example.com/api/v1/announce/complexkey456"
    );
}
