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
    pub port: u16,
    /// The base URL for the API. For example: `http://localhost`.
    /// If not set, the base URL will be inferred from the request.
    pub base_url: Option<Url>,
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
