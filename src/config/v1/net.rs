use std::net::{IpAddr, Ipv4Addr, SocketAddr};

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
    /// The base URL for the API. For example: `http://localhost`.
    /// If not set, the base URL will be inferred from the request.
    #[serde(default = "Network::default_base_url")]
    pub base_url: Option<Url>,
    /// The address the tracker will bind to.
    /// The format is `ip:port`, for example `0.0.0.0:6969`. If you want to
    /// listen to all interfaces, use `0.0.0.0`. If you want the operating
    /// system to choose a random port, use port `0`.
    #[serde(default = "Network::default_bind_address")]
    pub bind_address: SocketAddr,
    /// TSL configuration.
    #[serde(default = "Network::default_tsl")]
    pub tsl: Option<Tsl>,
}

impl Default for Network {
    fn default() -> Self {
        Self {
            bind_address: Self::default_bind_address(),
            base_url: Self::default_base_url(),
            tsl: Self::default_tsl(),
        }
    }
}

impl Network {
    fn default_bind_address() -> SocketAddr {
        SocketAddr::new(Self::default_ip(), Self::default_port())
    }

    fn default_ip() -> IpAddr {
        IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))
    }

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
