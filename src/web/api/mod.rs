//! The Torrust Index Backend API.
//!
//! Currently, the API has only one version: `v1`.
//!
//! Refer to the [`v1`](crate::web::api::v1) module for more information.
pub mod server;
pub mod v1;

use std::net::SocketAddr;
use std::sync::Arc;

use tokio::task::JoinHandle;

use crate::common::AppData;
use crate::web::api;

/// API versions.
pub enum Version {
    V1,
}

/// The running API server.
pub struct Running {
    /// The socket address the API server is listening on.
    pub socket_addr: SocketAddr,
    /// The handle for the running API server.
    pub api_server: Option<JoinHandle<Result<(), std::io::Error>>>,
}

#[must_use]
#[derive(Debug)]
pub struct ServerStartedMessage {
    pub socket_addr: SocketAddr,
}

/// Starts the API server.
#[must_use]
pub async fn start(app_data: Arc<AppData>, net_ip: &str, net_port: u16, implementation: &Version) -> api::Running {
    match implementation {
        Version::V1 => server::start(app_data, net_ip, net_port).await,
    }
}
