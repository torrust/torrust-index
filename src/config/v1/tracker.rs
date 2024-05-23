use std::fmt;

use serde::{Deserialize, Serialize};
use url::Url;

use super::{ValidationError, Validator};
use crate::config::TrackerMode;

/// Configuration for the associated tracker.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tracker {
    /// Connection string for the tracker. For example: `udp://TRACKER_IP:6969`.
    pub url: Url,
    /// The mode of the tracker. For example: `Public`.
    /// See `TrackerMode` in [`torrust-tracker-primitives`](https://docs.rs/torrust-tracker-primitives)
    /// crate for more information.
    pub mode: TrackerMode,
    /// The url of the tracker API. For example: `http://localhost:1212/`.
    pub api_url: Url,
    /// The token used to authenticate with the tracker API.
    pub token: ApiToken,
    /// The amount of seconds the tracker API token is valid.
    pub token_valid_seconds: u64,
}

impl Tracker {
    pub fn override_tracker_api_token(&mut self, tracker_api_token: &ApiToken) {
        self.token = tracker_api_token.clone();
    }
}

impl Validator for Tracker {
    fn validate(&self) -> Result<(), ValidationError> {
        let tracker_mode = self.mode.clone();
        let tracker_url = self.url.clone();

        if tracker_mode.is_close() && (tracker_url.scheme() != "http" && tracker_url.scheme() != "https") {
            return Err(ValidationError::UdpTrackersInPrivateModeNotSupported);
        }

        Ok(())
    }
}

impl Default for Tracker {
    fn default() -> Self {
        Self {
            url: Url::parse("udp://localhost:6969").unwrap(),
            mode: TrackerMode::default(),
            api_url: Url::parse("http://localhost:1212/").unwrap(),
            token: ApiToken::new("MyAccessToken"),
            token_valid_seconds: 7_257_600,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApiToken(String);

impl ApiToken {
    /// # Panics
    ///
    /// Will panic if the tracker API token if empty.
    #[must_use]
    pub fn new(key: &str) -> Self {
        assert!(!key.is_empty(), "tracker API token cannot be empty");

        Self(key.to_owned())
    }

    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl fmt::Display for ApiToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::ApiToken;

    #[test]
    #[should_panic(expected = "tracker API token cannot be empty")]
    fn secret_key_can_not_be_empty() {
        drop(ApiToken::new(""));
    }
}
