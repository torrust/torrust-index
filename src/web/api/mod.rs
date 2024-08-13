//! The Torrust Index API.
//!
//! Currently, the API has only one version: `v1`.
//!
//! Refer to:
//!
//! - [`client::v1`]) module for more information about the API client.
//! - [`server::v1`]) module for more information about the API server.
pub mod client;
pub mod server;

use std::net::SocketAddr;
use std::sync::Arc;

use tokio::task::JoinHandle;

use self::server::signals::Halted;
use crate::common::AppData;
use crate::config::Tsl;
use crate::web::api;

/// API versions.
pub enum Version {
    V1,
}

/// The running API server.
pub struct Running {
    /// The socket address the API server is listening on.
    pub socket_addr: SocketAddr,
    /// The channel sender to send halt signal to the server.
    pub halt_task: tokio::sync::oneshot::Sender<Halted>,
    /// The handle for the running API server.
    pub task: JoinHandle<Result<(), std::io::Error>>,
}

/// Starts the API server.
#[must_use]
pub async fn start(
    app_data: Arc<AppData>,
    config_bind_address: SocketAddr,
    opt_tsl: Option<Tsl>,
    implementation: &Version,
) -> api::Running {
    match implementation {
        Version::V1 => server::start(app_data, config_bind_address, opt_tsl).await,
    }
}
