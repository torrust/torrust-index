//! Cronjob to import tracker torrent data and updating seeders and leechers
//! info.
//!
//! It has two services:
//!
//! - The importer which is the cronjob executed at regular intervals.
//! - The importer API.
//!
//! The cronjob sends a heartbeat signal to the API each time it is executed.
//! The last heartbeat signal time is used to determine whether the cronjob was
//! executed successfully or not. The API has a `health_check` endpoint which is
//! used when the application is running in containers.
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use log::{error, info};
use serde_json::{json, Value};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

use crate::tracker::statistics_importer::StatisticsImporter;

const IMPORTER_API_IP: &str = "127.0.0.1";

#[derive(Clone)]
struct ImporterState {
    /// Shared variable to store the timestamp of the last heartbeat sent
    /// by the cronjob.
    pub last_heartbeat: Arc<Mutex<DateTime<Utc>>>,
    /// Interval between importation executions
    pub torrent_info_update_interval: u64,
}

/// # Panics
///
/// Will panic if it can't start the tracker statistics importer API
#[must_use]
pub fn start(
    importer_port: u16,
    torrent_info_update_interval: u64,
    tracker_statistics_importer: &Arc<StatisticsImporter>,
) -> JoinHandle<()> {
    let weak_tracker_statistics_importer = Arc::downgrade(tracker_statistics_importer);

    tokio::spawn(async move {
        info!("Tracker statistics importer launcher started");

        // Start the Importer API

        let _importer_api_handle = tokio::spawn(async move {
            let import_state = Arc::new(ImporterState {
                last_heartbeat: Arc::new(Mutex::new(Utc::now())),
                torrent_info_update_interval,
            });

            let app = Router::new()
                .route("/", get(|| async { Json(json!({})) }))
                .route("/health_check", get(health_check_handler))
                .with_state(import_state.clone())
                .route("/heartbeat", post(heartbeat_handler))
                .with_state(import_state);

            let addr = format!("{IMPORTER_API_IP}:{importer_port}");

            info!("Tracker statistics importer API server listening on http://{}", addr); // # DevSkim: ignore DS137138

            let socket_addr: SocketAddr = addr.parse().expect("importer API to have a valid socket address");

            let listener = TcpListener::bind(socket_addr)
                .await
                .expect("importer API TCP listener to bind to socket address");

            axum::serve(listener, app).await.unwrap();
        });

        // Start the Importer cronjob

        info!("Tracker statistics importer cronjob starting ...");

        let interval = std::time::Duration::from_secs(torrent_info_update_interval);
        let mut interval = tokio::time::interval(interval);

        interval.tick().await; // first tick is immediate...

        loop {
            interval.tick().await;

            info!("Running tracker statistics importer ...");

            if let Err(e) = send_heartbeat(importer_port).await {
                error!("Failed to send heartbeat from importer cronjob: {}", e);
            }

            if let Some(tracker) = weak_tracker_statistics_importer.upgrade() {
                drop(tracker.import_all_torrents_statistics().await);
            } else {
                break;
            }
        }
    })
}

/// Endpoint for container health check.
async fn health_check_handler(State(state): State<Arc<ImporterState>>) -> Json<Value> {
    let margin_in_seconds = 10;
    let now = Utc::now();
    let last_heartbeat = state.last_heartbeat.lock().unwrap();

    if now.signed_duration_since(*last_heartbeat).num_seconds()
        <= (state.torrent_info_update_interval + margin_in_seconds).try_into().unwrap()
    {
        Json(json!({ "status": "Ok" }))
    } else {
        Json(json!({ "status": "Error" }))
    }
}

/// The tracker statistics importer cronjob sends a heartbeat on each execution
/// to inform that it's alive. This endpoint handles receiving that signal.
async fn heartbeat_handler(State(state): State<Arc<ImporterState>>) -> Json<Value> {
    let now = Utc::now();
    let mut last_heartbeat = state.last_heartbeat.lock().unwrap();
    *last_heartbeat = now;
    Json(json!({ "status": "Heartbeat received" }))
}

/// Send a heartbeat from the importer cronjob to the importer API.
async fn send_heartbeat(importer_port: u16) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    let url = format!("http://{IMPORTER_API_IP}:{importer_port}/heartbeat"); // # DevSkim: ignore DS137138

    client.post(url).send().await?;

    Ok(())
}
