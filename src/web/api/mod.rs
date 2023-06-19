//! The Torrust Index Backend API.
//!
//! Currently, the API has only one version: `v1`.
//!
//! Refer to the [`v1`](crate::web::api::v1) module for more information.
pub mod actix;
pub mod axum;
pub mod v1;

use std::net::SocketAddr;
use std::sync::Arc;

use tokio::task::JoinHandle;

use crate::common::AppData;
use crate::web::api;

pub const API_VERSION: &str = "v1";

/// API implementations.
pub enum Implementation {
    /// API implementation with Actix Web.
    ActixWeb,
    /// API implementation with Axum.
    Axum,
}

/// The running API server.
pub struct Running {
    /// The socket address the API server is listening on.
    pub socket_addr: SocketAddr,
    /// The API server when using Actix Web.
    pub actix_web_api_server: Option<JoinHandle<Result<(), std::io::Error>>>,
    /// The handle for the running API server task when using Axum.
    pub axum_api_server: Option<JoinHandle<Result<(), std::io::Error>>>,
}

#[must_use]
#[derive(Debug)]
pub struct ServerStartedMessage {
    pub socket_addr: SocketAddr,
}

/// Starts the API server.
///
/// We are migrating the API server from Actix Web to Axum. While the migration
/// is in progress, we will keep both implementations, running the Axum one only
/// for testing purposes.
#[must_use]
pub async fn start(app_data: Arc<AppData>, net_ip: &str, net_port: u16, implementation: &Implementation) -> api::Running {
    match implementation {
        Implementation::ActixWeb => actix::start(app_data, net_ip, net_port).await,
        Implementation::Axum => axum::start(app_data, net_ip, net_port).await,
    }
}
