use std::fmt;

use serde::{Deserialize, Serialize};
use url::Url;

use super::{ValidationError, Validator};

/// Configuration for the associated tracker.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tracker {
    /// The url of the tracker API. For example: `http://localhost:1212/`.
    #[serde(default = "Tracker::default_api_url")]
    pub api_url: Url,

    /// Whether the tracker is running in listed mode or not.
    #[serde(default = "Tracker::default_listed")]
    pub listed: bool,

    /// Whether the tracker is running in private mode or not.
    #[serde(default = "Tracker::default_private")]
    pub private: bool,

    /// The token used to authenticate with the tracker API.
    #[serde(default = "Tracker::default_token")]
    pub token: ApiToken,

    /// The amount of seconds the tracker API token is valid.
    #[serde(default = "Tracker::default_token_valid_seconds")]
    pub token_valid_seconds: u64,

    /// Connection string for the tracker. For example: `udp://TRACKER_IP:6969`.
    #[serde(default = "Tracker::default_url")]
    pub url: Url,
}

impl Validator for Tracker {
    fn validate(&self) -> Result<(), ValidationError> {
        if self.private && (self.url.scheme() != "http" && self.url.scheme() != "https") {
            return Err(ValidationError::UdpTrackersInPrivateModeNotSupported);
        }

        Ok(())
    }
}

impl Default for Tracker {
    fn default() -> Self {
        Self {
            url: Self::default_url(),
            listed: Self::default_listed(),
            private: Self::default_private(),
            api_url: Self::default_api_url(),
            token: Self::default_token(),
            token_valid_seconds: Self::default_token_valid_seconds(),
        }
    }
}

impl Tracker {
    pub fn override_tracker_api_token(&mut self, tracker_api_token: &ApiToken) {
        self.token = tracker_api_token.clone();
    }

    fn default_url() -> Url {
        Url::parse("udp://localhost:6969").unwrap()
    }

    fn default_listed() -> bool {
        false
    }

    fn default_private() -> bool {
        false
    }

    fn default_api_url() -> Url {
        Url::parse("http://localhost:1212/").unwrap()
    }

    fn default_token() -> ApiToken {
        ApiToken::new("MyAccessToken")
    }

    fn default_token_valid_seconds() -> u64 {
        7_257_600
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
    fn apai_token_can_not_be_empty() {
        drop(ApiToken::new(""));
    }
}
