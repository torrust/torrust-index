use std::sync::Arc;

use log::{error, info};
use serde::{Deserialize, Serialize};

use crate::config::Configuration;
use crate::databases::database::Database;
use crate::errors::ServiceError;
use crate::models::tracker_key::TrackerKey;

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

pub struct TrackerService {
    cfg: Arc<Configuration>,
    database: Arc<Box<dyn Database>>,
}

impl TrackerService {
    pub fn new(cfg: Arc<Configuration>, database: Arc<Box<dyn Database>>) -> TrackerService {
        TrackerService { cfg, database }
    }

    pub async fn whitelist_info_hash(&self, info_hash: String) -> Result<(), ServiceError> {
        let settings = self.cfg.settings.read().await;

        let request_url = format!(
            "{}/api/v1/whitelist/{}?token={}",
            settings.tracker.api_url, info_hash, settings.tracker.token
        );

        drop(settings);

        let client = reqwest::Client::new();

        let response = client
            .post(request_url)
            .send()
            .await
            .map_err(|_| ServiceError::TrackerOffline)?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(ServiceError::InternalServerError)
        }
    }

    pub async fn remove_info_hash_from_whitelist(&self, info_hash: String) -> Result<(), ServiceError> {
        let settings = self.cfg.settings.read().await;

        let request_url = format!(
            "{}/api/v1/whitelist/{}?token={}",
            settings.tracker.api_url, info_hash, settings.tracker.token
        );

        drop(settings);

        let client = reqwest::Client::new();

        let response = match client.delete(request_url).send().await {
            Ok(v) => Ok(v),
            Err(_) => Err(ServiceError::InternalServerError),
        }?;

        if response.status().is_success() {
            return Ok(());
        }

        Err(ServiceError::InternalServerError)
    }

    // get personal tracker announce url of a user
    // Eg: https://tracker.torrust.com/announce/USER_TRACKER_KEY
    pub async fn get_personal_announce_url(&self, user_id: i64) -> Result<String, ServiceError> {
        let settings = self.cfg.settings.read().await;

        // get a valid tracker key for this user from database
        let tracker_key = self.database.get_user_tracker_key(user_id).await;

        match tracker_key {
            Some(v) => Ok(format!("{}/{}", settings.tracker.url, v.key)),
            None => match self.retrieve_new_tracker_key(user_id).await {
                Ok(v) => Ok(format!("{}/{}", settings.tracker.url, v.key)),
                Err(_) => Err(ServiceError::TrackerOffline),
            },
        }
    }

    // issue a new tracker key from tracker and save it in database, tied to a user
    pub async fn retrieve_new_tracker_key(&self, user_id: i64) -> Result<TrackerKey, ServiceError> {
        let settings = self.cfg.settings.read().await;

        let request_url = format!(
            "{}/api/v1/key/{}?token={}",
            settings.tracker.api_url, settings.tracker.token_valid_seconds, settings.tracker.token
        );

        drop(settings);

        let client = reqwest::Client::new();

        // issue new tracker key
        let response = client
            .post(request_url)
            .send()
            .await
            .map_err(|_| ServiceError::InternalServerError)?;

        // get tracker key from response
        let tracker_key = response
            .json::<TrackerKey>()
            .await
            .map_err(|_| ServiceError::InternalServerError)?;

        // add tracker key to database (tied to a user)
        self.database.add_tracker_key(user_id, &tracker_key).await?;

        // return tracker key
        Ok(tracker_key)
    }

    // get torrent info from tracker api
    pub async fn get_torrent_info(&self, torrent_id: i64, info_hash: &str) -> Result<TorrentInfo, ServiceError> {
        let settings = self.cfg.settings.read().await;

        let tracker_url = settings.tracker.url.clone();

        let request_url = format!(
            "{}/api/v1/torrent/{}?token={}",
            settings.tracker.api_url, info_hash, settings.tracker.token
        );

        drop(settings);

        let client = reqwest::Client::new();
        let response = match client.get(request_url).send().await {
            Ok(v) => Ok(v),
            Err(_) => Err(ServiceError::InternalServerError),
        }?;

        let torrent_info = match response.json::<TorrentInfo>().await {
            Ok(torrent_info) => {
                let _ = self
                    .database
                    .update_tracker_info(torrent_id, &tracker_url, torrent_info.seeders, torrent_info.leechers)
                    .await;
                Ok(torrent_info)
            }
            Err(_) => {
                let _ = self.database.update_tracker_info(torrent_id, &tracker_url, 0, 0).await;
                Err(ServiceError::TorrentNotFound)
            }
        }?;

        Ok(torrent_info)
    }

    pub async fn update_torrents(&self) -> Result<(), ServiceError> {
        info!("Updating torrents ...");
        let torrents = self.database.get_all_torrents_compact().await?;

        for torrent in torrents {
            info!("Updating torrent {} ...", torrent.torrent_id);
            let ret = self
                .update_torrent_tracker_stats(torrent.torrent_id, &torrent.info_hash)
                .await;
            if let Some(err) = ret.err() {
                error!(
                    "Error updating torrent tracker stats for torrent {}: {:?}",
                    torrent.torrent_id, err
                );
            }
        }

        Ok(())
    }

    pub async fn update_torrent_tracker_stats(&self, torrent_id: i64, info_hash: &str) -> Result<TorrentInfo, ServiceError> {
        self.get_torrent_info(torrent_id, info_hash).await
    }
}
