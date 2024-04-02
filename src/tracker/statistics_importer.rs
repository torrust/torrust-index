use std::sync::Arc;
use std::time::Instant;

use chrono::{DateTime, Utc};
use log::{debug, error, info};
use text_colorizer::Colorize;

use super::service::{Service, TorrentInfo, TrackerAPIError};
use crate::config::Configuration;
use crate::databases::database::{self, Database};

const LOG_TARGET: &str = "Tracker Stats Importer";

pub struct StatisticsImporter {
    database: Arc<Box<dyn Database>>,
    tracker_service: Arc<Service>,
    tracker_url: String,
}

impl StatisticsImporter {
    pub async fn new(cfg: Arc<Configuration>, tracker_service: Arc<Service>, database: Arc<Box<dyn Database>>) -> Self {
        let settings = cfg.settings.read().await;
        let tracker_url = settings.tracker.url.clone();
        drop(settings);
        Self {
            database,
            tracker_service,
            tracker_url,
        }
    }

    /// Import torrents statistics from tracker and update them in database.
    ///
    /// # Errors
    ///
    /// Will return an error if the database query failed.
    pub async fn import_all_torrents_statistics(&self) -> Result<(), database::Error> {
        let torrents = self.database.get_all_torrents_compact().await?;

        if torrents.is_empty() {
            return Ok(());
        }

        info!(target: LOG_TARGET, "Importing {} torrents statistics from tracker {} ...", torrents.len().to_string().yellow(), self.tracker_url.yellow());

        // Start the timer before the loop
        let start_time = Instant::now();

        for torrent in torrents {
            info!(target: LOG_TARGET, "Importing torrent #{} statistics ...", torrent.torrent_id.to_string().yellow());

            let ret = self.import_torrent_statistics(torrent.torrent_id, &torrent.info_hash).await;

            if let Some(err) = ret.err() {
                if err != TrackerAPIError::TorrentNotFound {
                    let message = format!(
                        "Error updating torrent tracker stats for torrent. Torrent: id {}; infohash {}. Error: {:?}",
                        torrent.torrent_id, torrent.info_hash, err
                    );
                    error!(target: "statistics_importer", "{}", message);
                    // todo: return a service error that can be a tracker API error or a database error.
                }
            }
        }

        let elapsed_time = start_time.elapsed();

        info!(target: LOG_TARGET, "Statistics import completed in {:.2?}", elapsed_time);

        Ok(())
    }

    /// Import torrents statistics not updated recently..
    ///
    /// # Errors
    ///
    /// Will return an error if the database query failed.
    pub async fn import_torrents_statistics_not_updated_since(
        &self,
        datetime: DateTime<Utc>,
        limit: i64,
    ) -> Result<(), database::Error> {
        debug!(target: LOG_TARGET, "Importing torrents statistics not updated since {} limited to a maximum of {} torrents ...", datetime.to_string().yellow(), limit.to_string().yellow());

        let torrents = self
            .database
            .get_torrents_with_stats_not_updated_since(datetime, limit)
            .await?;

        if torrents.is_empty() {
            return Ok(());
        }

        info!(target: LOG_TARGET, "Importing {} torrents statistics from tracker {} ...", torrents.len().to_string().yellow(), self.tracker_url.yellow());

        // Import stats for all torrents in one request

        let info_hashes: Vec<String> = torrents.iter().map(|t| t.info_hash.clone()).collect();

        let torrent_info_vec = match self.tracker_service.get_torrents_info(&info_hashes).await {
            Ok(torrents_info) => torrents_info,
            Err(err) => {
                let message = format!("Error getting torrents tracker stats. Error: {err:?}");
                error!(target: LOG_TARGET, "{}", message);
                // todo: return a service error that can be a tracker API error or a database error.
                return Ok(());
            }
        };

        // Update stats for all torrents

        for torrent in torrents {
            match torrent_info_vec.iter().find(|t| t.info_hash == torrent.info_hash) {
                None => {
                    // No stats for this torrent in the tracker
                    drop(
                        self.database
                            .update_tracker_info(torrent.torrent_id, &self.tracker_url, 0, 0)
                            .await,
                    );
                }
                Some(torrent_info) => {
                    // Update torrent stats for this tracker
                    drop(
                        self.database
                            .update_tracker_info(
                                torrent.torrent_id,
                                &self.tracker_url,
                                torrent_info.seeders,
                                torrent_info.leechers,
                            )
                            .await,
                    );
                }
            }
        }

        Ok(())
    }

    /// Import torrent statistics from tracker and update them in database.
    ///
    /// # Errors
    ///
    /// Will return an error if the HTTP request failed or the torrent is not
    /// found.
    pub async fn import_torrent_statistics(&self, torrent_id: i64, info_hash: &str) -> Result<TorrentInfo, TrackerAPIError> {
        match self.tracker_service.get_torrent_info(info_hash).await {
            Ok(torrent_info) => {
                drop(
                    self.database
                        .update_tracker_info(torrent_id, &self.tracker_url, torrent_info.seeders, torrent_info.leechers)
                        .await,
                );
                Ok(torrent_info)
            }
            Err(err) => {
                drop(self.database.update_tracker_info(torrent_id, &self.tracker_url, 0, 0).await);
                Err(err)
            }
        }
    }
}
