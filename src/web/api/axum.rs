use std::net::SocketAddr;
use std::sync::Arc;

use futures::Future;
use log::info;
use tokio::sync::oneshot::{self, Sender};

use super::v1::routes::router;
use super::{Running, ServerStartedMessage};
use crate::common::AppData;

/// Starts the API server with `Axum`.
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

        let handle = start_server(config_socket_addr, app_data.clone(), tx);

        if let Ok(()) = handle.await {
            info!("API server stopped");
        }

        Ok(())
    });

    // Wait until the API server is running
    let bound_addr = match rx.await {
        Ok(msg) => msg.socket_addr,
        Err(e) => panic!("API server start. The API server was dropped: {e}"),
    };

    Running {
        socket_addr: bound_addr,
        actix_web_api_server: None,
        axum_api_server: Some(join_handle),
    }
}

fn start_server(
    config_socket_addr: SocketAddr,
    app_data: Arc<AppData>,
    tx: Sender<ServerStartedMessage>,
) -> impl Future<Output = hyper::Result<()>> {
    let tcp_listener = std::net::TcpListener::bind(config_socket_addr).expect("tcp listener to bind to a socket address");

    let bound_addr = tcp_listener
        .local_addr()
        .expect("tcp listener to be bound to a socket address.");

    info!("API server listening on http://{}", bound_addr);

    let app = router(app_data);

    let server = axum::Server::from_tcp(tcp_listener)
        .expect("a new server from the previously created tcp listener.")
        .serve(app.into_make_service_with_connect_info::<SocketAddr>());

    tx.send(ServerStartedMessage { socket_addr: bound_addr })
        .expect("the API server should not be dropped");

    server.with_graceful_shutdown(async move {
        tokio::signal::ctrl_c().await.expect("Failed to listen to shutdown signal.");
        info!("Stopping API server on http://{} ...", bound_addr);
    })
}
