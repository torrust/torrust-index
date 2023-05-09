use std::sync::Arc;

use log::{error, info};
use reqwest::{Error, Response};
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
    database: Arc<Box<dyn Database>>,
    api_client: ApiClient,
    token_valid_seconds: u64,
    tracker_url: String,
}

impl TrackerService {
    pub async fn new(cfg: Arc<Configuration>, database: Arc<Box<dyn Database>>) -> TrackerService {
        let settings = cfg.settings.read().await;
        let api_client = ApiClient::new(ApiConnectionInfo::new(
            settings.tracker.api_url.clone(),
            settings.tracker.token.clone(),
        ));
        let token_valid_seconds = settings.tracker.token_valid_seconds;
        let tracker_url = settings.tracker.url.clone();
        drop(settings);
        TrackerService {
            database,
            api_client,
            token_valid_seconds,
            tracker_url,
        }
    }

    /// Add a torrent to the tracker whitelist.
    ///
    /// # Errors
    ///
    /// Will return an error if the HTTP request failed (for example if the
    /// tracker API is offline) or if the tracker API returned an error.
    pub async fn whitelist_info_hash(&self, info_hash: String) -> Result<(), ServiceError> {
        let response = self.api_client.whitelist_info_hash(&info_hash).await;

        match response {
            Ok(response) => {
                if response.status().is_success() {
                    Ok(())
                } else {
                    Err(ServiceError::WhitelistingError)
                }
            }
            Err(_) => Err(ServiceError::TrackerOffline),
        }
    }

    /// Remove a torrent from the tracker whitelist.
    ///
    /// # Errors
    ///
    /// Will return an error if the HTTP request failed (for example if the
    /// tracker API is offline) or if the tracker API returned an error.
    pub async fn remove_info_hash_from_whitelist(&self, info_hash: String) -> Result<(), ServiceError> {
        let response = self.api_client.remove_info_hash_from_whitelist(&info_hash).await;

        match response {
            Ok(response) => {
                if response.status().is_success() {
                    Ok(())
                } else {
                    Err(ServiceError::InternalServerError)
                }
            }
            Err(_) => Err(ServiceError::InternalServerError),
        }
    }

    /// Get personal tracker announce url of a user.
    ///
    /// Eg: <https://tracker:7070/USER_TRACKER_KEY>
    ///
    /// If the user doesn't have a not expired tracker key, it will generate a
    /// new one and save it in the database.
    ///
    /// # Errors
    ///
    /// Will return an error if the HTTP request to get generated a new
    /// user tracker key failed.
    pub async fn get_personal_announce_url(&self, user_id: i64) -> Result<String, ServiceError> {
        let tracker_key = self.database.get_user_tracker_key(user_id).await;

        match tracker_key {
            Some(v) => Ok(self.announce_url_with_key(&v)),
            None => match self.retrieve_new_tracker_key(user_id).await {
                Ok(v) => Ok(self.announce_url_with_key(&v)),
                Err(_) => Err(ServiceError::TrackerOffline),
            },
        }
    }

    /// It builds the announce url appending the user tracker key.
    /// Eg: <https://tracker:7070/USER_TRACKER_KEY>
    fn announce_url_with_key(&self, tracker_key: &TrackerKey) -> String {
        format!("{}/{}", self.tracker_url, tracker_key.key)
    }

    /// Issue a new tracker key from tracker and save it in database,
    /// tied to a user
    async fn retrieve_new_tracker_key(&self, user_id: i64) -> Result<TrackerKey, ServiceError> {
        // Request new tracker key from tracker
        let response = self
            .api_client
            .retrieve_new_tracker_key(self.token_valid_seconds)
            .await
            .map_err(|_| ServiceError::InternalServerError)?;

        // Parse tracker key from response
        let tracker_key = response
            .json::<TrackerKey>()
            .await
            .map_err(|_| ServiceError::InternalServerError)?;

        // Add tracker key to database (tied to a user)
        self.database.add_tracker_key(user_id, &tracker_key).await?;

        // return tracker key
        Ok(tracker_key)
    }

    /// Get torrent info from tracker API
    ///
    /// # Errors
    ///
    /// Will return an error if the HTTP request failed or the torrent is not
    /// found.
    pub async fn get_torrent_info(&self, torrent_id: i64, info_hash: &str) -> Result<TorrentInfo, ServiceError> {
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

struct ApiConnectionInfo {
    pub url: String,
    pub token: String,
}

impl ApiConnectionInfo {
    pub fn new(url: String, token: String) -> Self {
        Self { url, token }
    }
}

struct ApiClient {
    pub connection_info: ApiConnectionInfo,
    base_url: String,
}

impl ApiClient {
    pub fn new(connection_info: ApiConnectionInfo) -> Self {
        let base_url = format!("{}/api/v1", connection_info.url);
        Self {
            connection_info,
            base_url,
        }
    }

    pub async fn whitelist_info_hash(&self, info_hash: &str) -> Result<Response, Error> {
        let request_url = format!(
            "{}/whitelist/{}?token={}",
            self.base_url, info_hash, self.connection_info.token
        );

        let client = reqwest::Client::new();

        client.post(request_url).send().await
    }

    pub async fn remove_info_hash_from_whitelist(&self, info_hash: &str) -> Result<Response, Error> {
        let request_url = format!(
            "{}/whitelist/{}?token={}",
            self.base_url, info_hash, self.connection_info.token
        );

        let client = reqwest::Client::new();

        client.delete(request_url).send().await
    }

    async fn retrieve_new_tracker_key(&self, token_valid_seconds: u64) -> Result<Response, Error> {
        let request_url = format!(
            "{}/key/{}?token={}",
            self.base_url, token_valid_seconds, self.connection_info.token
        );

        let client = reqwest::Client::new();

        client.post(request_url).send().await
    }

    pub async fn get_torrent_info(&self, info_hash: &str) -> Result<Response, Error> {
        let request_url = format!("{}/torrent/{}?token={}", self.base_url, info_hash, self.connection_info.token);

        let client = reqwest::Client::new();

        client.get(request_url).send().await
    }
}
