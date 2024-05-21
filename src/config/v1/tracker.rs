use serde::{Deserialize, Serialize};
use torrust_index_located_error::Located;

use super::{ValidationError, Validator};
use crate::config::{parse_url, TrackerMode};

/// Configuration for the associated tracker.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tracker {
    /// Connection string for the tracker. For example: `udp://TRACKER_IP:6969`.
    pub url: String,
    /// The mode of the tracker. For example: `Public`.
    /// See `TrackerMode` in [`torrust-tracker-primitives`](https://docs.rs/torrust-tracker-primitives)
    /// crate for more information.
    pub mode: TrackerMode,
    /// The url of the tracker API. For example: `http://localhost:1212`.
    pub api_url: String,
    /// The token used to authenticate with the tracker API.
    pub token: String,
    /// The amount of seconds the token is valid.
    pub token_valid_seconds: u64,
}

impl Tracker {
    pub fn override_tracker_api_token(&mut self, tracker_api_token: &str) {
        self.token = tracker_api_token.to_string();
    }
}

impl Validator for Tracker {
    fn validate(&self) -> Result<(), ValidationError> {
        let tracker_mode = self.mode.clone();
        let tracker_url = self.url.clone();

        let tracker_url = match parse_url(&tracker_url) {
            Ok(url) => url,
            Err(err) => {
                return Err(ValidationError::InvalidTrackerUrl {
                    source: Located(err).into(),
                })
            }
        };

        if tracker_mode.is_close() && (tracker_url.scheme() != "http" && tracker_url.scheme() != "https") {
            return Err(ValidationError::UdpTrackersInPrivateModeNotSupported);
        }

        Ok(())
    }
}

impl Default for Tracker {
    fn default() -> Self {
        Self {
            url: "udp://localhost:6969".to_string(),
            mode: TrackerMode::default(),
            api_url: "http://localhost:1212".to_string(),
            token: "MyAccessToken".to_string(),
            token_valid_seconds: 7_257_600,
        }
    }
}
