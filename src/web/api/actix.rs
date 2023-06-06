use std::net::SocketAddr;
use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer};
use log::info;
use tokio::sync::oneshot::{self, Sender};

use super::Running;
use crate::common::AppData;
use crate::routes;
use crate::web::api::ServerStartedMessage;

/// Starts the API server with `ActixWeb`.
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

        let server_future = start_server(config_socket_addr, app_data.clone(), tx);

        let _ = server_future.await;

        Ok(())
    });

    // Wait until the API server is running
    let bound_addr = match rx.await {
        Ok(msg) => msg.socket_addr,
        Err(e) => panic!("API server start. The API server was dropped: {e}"),
    };

    info!("API server started");

    Running {
        socket_addr: bound_addr,
        actix_web_api_server: Some(join_handle),
        axum_api_server: None,
    }
}

fn start_server(
    config_socket_addr: SocketAddr,
    app_data: Arc<AppData>,
    tx: Sender<ServerStartedMessage>,
) -> actix_web::dev::Server {
    let server = HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .app_data(web::Data::new(app_data.clone()))
            .wrap(middleware::Logger::default())
            .configure(routes::init)
    })
    .bind(config_socket_addr)
    .expect("can't bind server to socket address");

    let bound_addr = server.addrs()[0];

    info!("API server listening on http://{}", bound_addr);

    tx.send(ServerStartedMessage { socket_addr: bound_addr })
        .expect("the API server should not be dropped");

    server.run()
}
