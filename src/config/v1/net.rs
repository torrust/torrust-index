use serde::{Deserialize, Serialize};

use crate::config::Tsl;

/// The the base URL for the API.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Network {
    /// The port to listen on. Default to `3001`.
    pub port: u16,
    /// The base URL for the API. For example: `http://localhost`.
    /// If not set, the base URL will be inferred from the request.
    pub base_url: Option<String>,
    /// TSL configuration.
    pub tsl: Option<Tsl>,
}

impl Default for Network {
    fn default() -> Self {
        Self {
            port: 3001,
            base_url: None,
            tsl: None,
        }
    }
}
