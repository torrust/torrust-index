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
use log::{debug, error, info};
use serde_json::{json, Value};
use text_colorizer::Colorize;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

use crate::tracker::statistics_importer::StatisticsImporter;
use crate::utils::clock::seconds_ago_utc;

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
    torrent_stats_update_interval: u64,
    tracker_statistics_importer: &Arc<StatisticsImporter>,
) -> JoinHandle<()> {
    let weak_tracker_statistics_importer = Arc::downgrade(tracker_statistics_importer);

    tokio::spawn(async move {
        info!("Tracker statistics importer launcher started");

        // Start the Importer API

        let _importer_api_handle = tokio::spawn(async move {
            let import_state = Arc::new(ImporterState {
                last_heartbeat: Arc::new(Mutex::new(Utc::now())),
                torrent_info_update_interval: torrent_stats_update_interval,
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

        // code-review:
        //
        // We set an execution interval to avoid intense polling to the
        // database. If we remove the interval we would be constantly queering
        // if there are torrent stats pending to update, unless there are
        // torrents to update. Maybe we should only sleep for 100 milliseconds
        // if we did not update any torrents in the latest execution. With this
        // current limit we can only import 50 torrent stats every 2000 seconds,
        // which is 500 torrents per second (1800000 torrents per hour).
        //
        // | Interval (secs) | Number of torrents imported per hour |
        // ------------------|--------------------------------------|
        // |           1 sec |               50 * (3600/1) = 180000 |
        // |           2 sec |               50 * (3600/2) =  90000 |
        // |           3 sec |               50 * (3600/3) =  60000 |
        // |           4 sec |               50 * (3600/4) =  45000 |
        // |           5 sec |               50 * (3600/5) =  36000 |
        //
        // The `execution_interval_in_milliseconds` could be a config option in
        // the future.

        let execution_interval_in_milliseconds = 2000;
        let execution_interval_duration = std::time::Duration::from_millis(execution_interval_in_milliseconds);
        let mut execution_interval = tokio::time::interval(execution_interval_duration);

        execution_interval.tick().await; // first tick is immediate...

        info!("Running tracker statistics importer every {execution_interval_in_milliseconds} milliseconds ...");

        loop {
            if let Err(e) = send_heartbeat(importer_port).await {
                error!("Failed to send heartbeat from importer cronjob: {}", e);
            }

            if let Some(statistics_importer) = weak_tracker_statistics_importer.upgrade() {
                let one_interval_ago = seconds_ago_utc(
                    torrent_stats_update_interval
                        .try_into()
                        .expect("update interval should be a positive integer"),
                );
                let limit = 50;

                debug!(
                    "Importing torrents statistics not updated since {} limited to a maximum of {} torrents ...",
                    one_interval_ago.to_string().yellow(),
                    limit.to_string().yellow()
                );

                match statistics_importer
                    .import_torrents_statistics_not_updated_since(one_interval_ago, limit)
                    .await
                {
                    Ok(()) => {}
                    Err(e) => error!("Failed to import statistics: {:?}", e),
                }

                drop(statistics_importer);
            } else {
                break;
            }

            execution_interval.tick().await;
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
