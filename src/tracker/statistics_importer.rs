use std::sync::Arc;

use log::{error, info};
use serde::{Deserialize, Serialize};

use super::api::{Client, ConnectionInfo};
use crate::config::Configuration;
use crate::databases::database::{Database, DatabaseError};
use crate::errors::ServiceError;

// If `TorrentInfo` struct is used in the future for other purposes, it should
// be moved to a separate file. Maybe a `ClientWrapper` struct which returns
// `TorrentInfo` and `TrackerKey` structs instead of `Response` structs.

#[derive(Debug, Serialize, Deserialize)]
pub struct TorrentInfo {
    pub info_hash: String,
    pub seeders: i64,
    pub completed: i64,
    pub leechers: i64,
    pub peers: Vec<Peer>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Peer {
    pub peer_id: Option<PeerId>,
    pub peer_addr: Option<String>,
    pub updated: Option<i64>,
    pub uploaded: Option<i64>,
    pub downloaded: Option<i64>,
    pub left: Option<i64>,
    pub event: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PeerId {
    pub id: Option<String>,
    pub client: Option<String>,
}

pub struct StatisticsImporter {
    database: Arc<Box<dyn Database>>,
    api_client: Client,
    tracker_url: String,
}

impl StatisticsImporter {
    pub async fn new(cfg: Arc<Configuration>, database: Arc<Box<dyn Database>>) -> Self {
        let settings = cfg.settings.read().await;
        let api_client = Client::new(ConnectionInfo::new(
            settings.tracker.api_url.clone(),
            settings.tracker.token.clone(),
        ));
        let tracker_url = settings.tracker.url.clone();
        drop(settings);
        Self {
            database,
            api_client,
            tracker_url,
        }
    }

    /// Import torrents statistics from tracker and update them in database.
    ///
    /// # Errors
    ///
    /// Will return an error if the database query failed.
    pub async fn import_all_torrents_statistics(&self) -> Result<(), DatabaseError> {
        info!("Importing torrents statistics from tracker ...");
        let torrents = self.database.get_all_torrents_compact().await?;

        for torrent in torrents {
            info!("Updating torrent {} ...", torrent.torrent_id);

            let ret = self.import_torrent_statistics(torrent.torrent_id, &torrent.info_hash).await;

            // code-review: should we treat differently for each case?. The
            // tracker API could be temporarily offline, or there could be a
            // tracker misconfiguration.
            //
            // This is the log when the torrent is not found in the tracker:
            //
            // ```
            // 2023-05-09T13:31:24.497465723+00:00 [torrust_index_backend::tracker::statistics_importer][ERROR] Error updating torrent tracker stats for torrent with id 140: TorrentNotFound
            // ```

            if let Some(err) = ret.err() {
                error!(
                    "Error updating torrent tracker stats for torrent with id {}: {:?}",
                    torrent.torrent_id, err
                );
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
    pub async fn import_torrent_statistics(&self, torrent_id: i64, info_hash: &str) -> Result<TorrentInfo, ServiceError> {
        let response = self
            .api_client
            .get_torrent_info(info_hash)
            .await
            .map_err(|_| ServiceError::InternalServerError)?;

        if let Ok(torrent_info) = response.json::<TorrentInfo>().await {
            let _ = self
                .database
                .update_tracker_info(torrent_id, &self.tracker_url, torrent_info.seeders, torrent_info.leechers)
                .await;
            Ok(torrent_info)
        } else {
            let _ = self.database.update_tracker_info(torrent_id, &self.tracker_url, 0, 0).await;
            Err(ServiceError::TorrentNotFound)
        }
    }
}
