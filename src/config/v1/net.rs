use serde::{Deserialize, Serialize};
use url::Url;

use crate::config::Tsl;

/// The the base URL for the API.
///
/// NOTICE: that `port` and por in `base_url` does not necessarily match because
/// the application migth be running behind a proxy. The local socket could be
/// bound to, for example, port 80 but the application could be exposed publicly
/// via port 443, which is a very common setup.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Network {
    /// The port to listen on. Default to `3001`.
    #[serde(default = "Network::default_port")]
    pub port: u16,
    /// The base URL for the API. For example: `http://localhost`.
    /// If not set, the base URL will be inferred from the request.
    #[serde(default = "Network::default_base_url")]
    pub base_url: Option<Url>,
    /// TSL configuration.
    #[serde(default = "Network::default_tsl")]
    pub tsl: Option<Tsl>,
}

impl Default for Network {
    fn default() -> Self {
        Self {
            port: Self::default_port(),
            base_url: Self::default_base_url(),
            tsl: Self::default_tsl(),
        }
    }
}

impl Network {
    fn default_port() -> u16 {
        3001
    }

    fn default_base_url() -> Option<Url> {
        None
    }

    fn default_tsl() -> Option<Tsl> {
        None
    }
}
