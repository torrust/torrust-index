pub mod custom_axum;
pub mod signals;
pub mod v1;

use std::net::SocketAddr;
use std::panic::Location;
use std::sync::Arc;

use axum_server::tls_rustls::RustlsConfig;
use axum_server::Handle;
use log::{error, info};
use thiserror::Error;
use tokio::sync::oneshot::{Receiver, Sender};
use torrust_index_located_error::LocatedError;
use v1::routes::router;

use self::signals::{Halted, Started};
use super::Running;
use crate::common::AppData;
use crate::config::Tsl;
use crate::web::api::server::custom_axum::TimeoutAcceptor;
use crate::web::api::server::signals::graceful_shutdown;

pub type DynError = Arc<dyn std::error::Error + Send + Sync>;

/// Starts the API server.
///
/// # Panics
///
/// Panics if the API server can't be started.
pub async fn start(app_data: Arc<AppData>, net_ip: &str, net_port: u16, opt_tsl: Option<Tsl>) -> Running {
    let config_socket_addr: SocketAddr = format!("{net_ip}:{net_port}")
        .parse()
        .expect("API server socket address to be valid.");

    let opt_rust_tls_config = make_rust_tls(&opt_tsl)
        .await
        .map(|tls| tls.expect("it should have a valid net tls configuration"));

    let (tx_start, rx) = tokio::sync::oneshot::channel::<Started>();
    let (tx_halt, rx_halt) = tokio::sync::oneshot::channel::<Halted>();

    // Run the API server
    let join_handle = tokio::spawn(async move {
        info!("Starting API server with net config: {} ...", config_socket_addr);

        start_server(config_socket_addr, app_data.clone(), tx_start, rx_halt, opt_rust_tls_config).await;

        info!("API server stopped");

        Ok(())
    });

    // Wait until the API server is running
    let bound_addr = match rx.await {
        Ok(started) => started.socket_addr,
        Err(err) => {
            let msg = format!("Unable to start API server: {err}");
            error!("{}", msg);
            panic!("{}", msg);
        }
    };

    Running {
        socket_addr: bound_addr,
        halt_task: tx_halt,
        task: join_handle,
    }
}

async fn start_server(
    config_socket_addr: SocketAddr,
    app_data: Arc<AppData>,
    tx_start: Sender<Started>,
    rx_halt: Receiver<Halted>,
    rust_tls_config: Option<RustlsConfig>,
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

    let tls = rust_tls_config.clone();
    let protocol = if tls.is_some() { "https" } else { "http" };

    info!("API server listening on {}://{}", protocol, address); // # DevSkim: ignore DS137138

    tx_start
        .send(Started { socket_addr: address })
        .expect("the API server should not be dropped");

    match tls {
        Some(tls) => custom_axum::from_tcp_rustls_with_timeouts(socket, tls)
            .handle(handle)
            // The TimeoutAcceptor is commented because TSL does not work with it.
            // See: https://github.com/torrust/torrust-index/issues/204
            //.acceptor(TimeoutAcceptor)
            .serve(router.into_make_service_with_connect_info::<std::net::SocketAddr>())
            .await
            .expect("API server should be running"),
        None => custom_axum::from_tcp_with_timeouts(socket)
            .handle(handle)
            .acceptor(TimeoutAcceptor)
            .serve(router.into_make_service_with_connect_info::<std::net::SocketAddr>())
            .await
            .expect("API server should be running"),
    };
}

#[derive(Error, Debug)]
pub enum Error {
    /// Enabled tls but missing config.
    #[error("tls config missing")]
    MissingTlsConfig { location: &'static Location<'static> },

    /// Unable to parse tls Config.
    #[error("bad tls config: {source}")]
    BadTlsConfig {
        source: LocatedError<'static, dyn std::error::Error + Send + Sync>,
        ssl_cert_path: String,
        ssl_key_path: String,
    },
}

pub async fn make_rust_tls(tsl_config: &Option<Tsl>) -> Option<Result<RustlsConfig, Error>> {
    match tsl_config {
        Some(tsl) => {
            if let (Some(cert), Some(key)) = (tsl.ssl_cert_path.clone(), tsl.ssl_key_path.clone()) {
                info!("Using https. Cert path: {cert}.");
                info!("Using https. Key path: {key}.");

                let ssl_cert_path = cert.clone().to_string();
                let ssl_key_path = key.clone().to_string();

                Some(
                    RustlsConfig::from_pem_file(cert, key)
                        .await
                        .map_err(|err| Error::BadTlsConfig {
                            source: (Arc::new(err) as DynError).into(),
                            ssl_cert_path,
                            ssl_key_path,
                        }),
                )
            } else {
                Some(Err(Error::MissingTlsConfig {
                    location: Location::caller(),
                }))
            }
        }
        None => {
            info!("TLS not enabled");
            None
        }
    }
}
