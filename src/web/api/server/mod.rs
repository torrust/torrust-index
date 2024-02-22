pub mod v1;

use std::net::SocketAddr;
use std::sync::Arc;

use log::info;
use tokio::net::TcpListener;
use tokio::sync::oneshot::{self, Sender};
use v1::routes::router;

use super::{Running, ServerStartedMessage};
use crate::common::AppData;

/// Starts the API server.
///
/// # Panics
///
/// Panics if the API server can't be started.
pub async fn start(app_data: Arc<AppData>, net_ip: &str, net_port: u16) -> Running {
    let config_socket_addr: SocketAddr = format!("{net_ip}:{net_port}")
        .parse()
        .expect("API server socket address to be valid.");

    let (tx, rx) = oneshot::channel::<ServerStartedMessage>();

    // Run the API server
    let join_handle = tokio::spawn(async move {
        info!("Starting API server with net config: {} ...", config_socket_addr);

        start_server(config_socket_addr, app_data.clone(), tx).await;

        info!("API server stopped");

        Ok(())
    });

    // Wait until the API server is running
    let bound_addr = match rx.await {
        Ok(msg) => msg.socket_addr,
        Err(e) => panic!("API server start. The API server was dropped: {e}"),
    };

    Running {
        socket_addr: bound_addr,
        api_server: Some(join_handle),
    }
}

async fn start_server(config_socket_addr: SocketAddr, app_data: Arc<AppData>, tx: Sender<ServerStartedMessage>) {
    let tcp_listener = TcpListener::bind(config_socket_addr)
        .await
        .expect("tcp listener to bind to a socket address");

    let bound_addr = tcp_listener
        .local_addr()
        .expect("tcp listener to be bound to a socket address.");

    info!("API server listening on http://{}", bound_addr); // # DevSkim: ignore DS137138

    let app = router(app_data);

    tx.send(ServerStartedMessage { socket_addr: bound_addr })
        .expect("the API server should not be dropped");

    axum::serve(tcp_listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(async move {
            tokio::signal::ctrl_c().await.expect("Failed to listen to shutdown signal.");
            info!("Stopping API server on http://{} ...", bound_addr); // # DevSkim: ignore DS137138
        })
        .await
        .expect("API server should be running");
}
