//! It imports statistics for all torrents from the linked tracker.
//!
//! It imports the number of seeders and leechers for all torrents from the
//! associated tracker.
//!
//! You can execute it with:
//!
//! ```text
//! cargo run --quiet --bin import_tracker_statistics -- 2>import-tracker-statistics.ndjson
//! jq . import-tracker-statistics.ndjson
//! ```
//!
//! ADR-T-010 classifies this as a side-effect command: stdout remains empty and
//! diagnostics are JSON records on stderr. Scripts should branch on the process
//! exit code and parse stderr as NDJSON when they need diagnostics.
//!
//! Statistics are also imported:
//!
//! - Periodically by the importer job. The importer job is executed every hour
//!   by default. See [`TrackerStatisticsImporter`](crate::config::TrackerStatisticsImporter)
//!   for more details.
//! - When a new torrent is added.
//! - When the API returns data about a torrent statistics are collected from
//!   the tracker in real time.
use std::sync::Arc;

use thiserror::Error;
use tracing::info;

use crate::bootstrap::config::DEFAULT_PATH_CONFIG;
use crate::config::{Configuration, Error as ConfigError, Info};
use crate::databases::database;
use crate::tracker::service::Service;
use crate::tracker::statistics_importer::StatisticsImporter;

#[derive(Debug, Error)]
pub enum ImportError {
    #[error("failed to build configuration lookup: {source}")]
    BuildConfigurationInfo { source: ConfigError },

    #[error("failed to load configuration: {source}")]
    LoadConfiguration { source: ConfigError },

    #[error("failed to connect to database: {source}")]
    ConnectDatabase { source: database::Error },

    #[error("failed to import tracker statistics: {source}")]
    ImportStatistics { source: database::Error },
}

/// Import Tracker Statistics Command
///
/// # Errors
///
/// Returns an error if configuration loading, database connection, or the
/// statistics import fails.
pub async fn run() -> Result<(), ImportError> {
    import().await
}

/// Import Command Arguments
///
/// # Errors
///
/// Returns an error if configuration loading, database connection, or the
/// statistics import fails.
pub async fn import() -> Result<(), ImportError> {
    info!("importing statistics from linked tracker");

    let config_info =
        Info::new(DEFAULT_PATH_CONFIG.to_string()).map_err(|source| ImportError::BuildConfigurationInfo { source })?;
    let configuration = Configuration::load(&config_info).map_err(|source| ImportError::LoadConfiguration { source })?;

    let cfg = Arc::new(configuration);

    let settings = cfg.settings.read().await;

    let tracker_url = settings.tracker.url.clone();
    info!(tracker_url = %tracker_url, "loaded tracker configuration");

    let database = Arc::new(
        database::connect(settings.database.connect_url.as_ref())
            .await
            .map_err(|source| ImportError::ConnectDatabase { source })?,
    );
    drop(settings);

    let tracker_service = Arc::new(Service::new(cfg.clone(), database.clone()).await);
    let tracker_statistics_importer =
        Arc::new(StatisticsImporter::new(cfg.clone(), tracker_service.clone(), database.clone()).await);

    tracker_statistics_importer
        .import_all_torrents_statistics()
        .await
        .map_err(|source| ImportError::ImportStatistics { source })?;

    info!("imported statistics from linked tracker");

    Ok(())
}
