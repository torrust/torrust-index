pub mod custom_axum;
pub mod signals;
pub mod v1;

use std::net::SocketAddr;
use std::sync::Arc;

use axum_server::Handle;
use log::info;
use tokio::sync::oneshot::{Receiver, Sender};
use v1::routes::router;

use self::signals::{Halted, Started};
use super::Running;
use crate::common::AppData;
use crate::web::api::server::custom_axum::TimeoutAcceptor;
use crate::web::api::server::signals::graceful_shutdown;

/// Starts the API server.
///
/// # Panics
///
/// Panics if the API server can't be started.
pub async fn start(app_data: Arc<AppData>, net_ip: &str, net_port: u16) -> Running {
    let config_socket_addr: SocketAddr = format!("{net_ip}:{net_port}")
        .parse()
        .expect("API server socket address to be valid.");

    let (tx_start, rx) = tokio::sync::oneshot::channel::<Started>();
    let (_tx_halt, rx_halt) = tokio::sync::oneshot::channel::<Halted>();

    // Run the API server
    let join_handle = tokio::spawn(async move {
        info!("Starting API server with net config: {} ...", config_socket_addr);

        start_server(config_socket_addr, app_data.clone(), tx_start, rx_halt).await;

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

async fn start_server(
    config_socket_addr: SocketAddr,
    app_data: Arc<AppData>,
    tx_start: Sender<Started>,
    rx_halt: Receiver<Halted>,
) {
    let router = router(app_data);
    let socket = std::net::TcpListener::bind(config_socket_addr).expect("Could not bind tcp_listener to address.");
    let address = socket.local_addr().expect("Could not get local_addr from tcp_listener.");

    let handle = Handle::new();

    tokio::task::spawn(graceful_shutdown(
        handle.clone(),
        rx_halt,
        format!("Shutting down API server on socket address: {address}"),
    ));

    info!("API server listening on http://{}", address); // # DevSkim: ignore DS137138

    tx_start
        .send(Started { socket_addr: address })
        .expect("the API server should not be dropped");

    custom_axum::from_tcp_with_timeouts(socket)
        .handle(handle)
        .acceptor(TimeoutAcceptor)
        .serve(router.into_make_service_with_connect_info::<std::net::SocketAddr>())
        .await
        .expect("API server should be running");
}
