use std::sync::Arc;

use log::{error, info};

use super::service::{Service, TorrentInfo, TrackerAPIError};
use crate::config::Configuration;
use crate::databases::database::{self, Database};

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
        info!("Importing torrents statistics from tracker ...");
        let torrents = self.database.get_all_torrents_compact().await?;

        for torrent in torrents {
            info!("Updating torrent {} ...", torrent.torrent_id);

            let ret = self.import_torrent_statistics(torrent.torrent_id, &torrent.info_hash).await;

            if let Some(err) = ret.err() {
                let message = format!(
                    "Error updating torrent tracker stats for torrent with id {}: {:?}",
                    torrent.torrent_id, err
                );
                error!(target: "statistics_importer", "{}", message);
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
