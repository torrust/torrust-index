use std::sync::Arc;

use hyper::StatusCode;
use log::error;
use reqwest::Response;
use serde::{Deserialize, Serialize};

use super::api::{Client, ConnectionInfo};
use crate::config::Configuration;
use crate::databases::database::Database;
use crate::errors::ServiceError;
use crate::models::tracker_key::TrackerKey;
use crate::models::user::UserId;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct TorrentInfo {
    pub info_hash: String,
    pub seeders: i64,
    pub completed: i64,
    pub leechers: i64,
    pub peers: Vec<Peer>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Peer {
    pub peer_id: Option<PeerId>,
    pub peer_addr: Option<String>,
    pub updated: Option<i64>,
    pub uploaded: Option<i64>,
    pub downloaded: Option<i64>,
    pub left: Option<i64>,
    pub event: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PeerId {
    pub id: Option<String>,
    pub client: Option<String>,
}

pub struct Service {
    database: Arc<Box<dyn Database>>,
    api_client: Client,
    token_valid_seconds: u64,
    tracker_url: String,
}

impl Service {
    pub async fn new(cfg: Arc<Configuration>, database: Arc<Box<dyn Database>>) -> Service {
        let settings = cfg.settings.read().await;
        let api_client = Client::new(ConnectionInfo::new(
            settings.tracker.api_url.clone(),
            settings.tracker.token.clone(),
        ));
        let token_valid_seconds = settings.tracker.token_valid_seconds;
        let tracker_url = settings.tracker.url.clone();
        drop(settings);
        Service {
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
        let response = self.api_client.whitelist_torrent(&info_hash).await;

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
        let response = self.api_client.remove_torrent_from_whitelist(&info_hash).await;

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
    pub async fn get_personal_announce_url(&self, user_id: UserId) -> Result<String, ServiceError> {
        let tracker_key = self.database.get_user_tracker_key(user_id).await;

        match tracker_key {
            Some(v) => Ok(self.announce_url_with_key(&v)),
            None => match self.retrieve_new_tracker_key(user_id).await {
                Ok(v) => Ok(self.announce_url_with_key(&v)),
                Err(_) => Err(ServiceError::TrackerOffline),
            },
        }
    }

    /// Get torrent info from tracker.
    ///
    /// # Errors
    ///
    /// Will return an error if the HTTP request to get torrent info fails or
    /// if the response cannot be parsed.
    pub async fn get_torrent_info(&self, info_hash: &str) -> Result<TorrentInfo, ServiceError> {
        let response = self
            .api_client
            .get_torrent_info(info_hash)
            .await
            .map_err(|_| ServiceError::InternalServerError)?;

        map_torrent_info_response(response).await
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
}

async fn map_torrent_info_response(response: Response) -> Result<TorrentInfo, ServiceError> {
    if response.status() == StatusCode::NOT_FOUND {
        return Err(ServiceError::TorrentNotFound);
    }

    let body = response.text().await.map_err(|_| {
        error!("Tracker API response without body");
        ServiceError::InternalServerError
    })?;

    if body == "\"torrent not known\"" {
        // todo: temporary fix. the service should return a 404 (StatusCode::NOT_FOUND).
        return Err(ServiceError::TorrentNotFound);
    }

    serde_json::from_str(&body).map_err(|e| {
        error!(
            "Failed to parse torrent info from tracker response. Body: {}, Error: {}",
            body, e
        );
        ServiceError::InternalServerError
    })
}

#[cfg(test)]
mod tests {

    mod getting_the_torrent_info_from_the_tracker {
        use hyper::{Response, StatusCode};

        use crate::errors::ServiceError;
        use crate::tracker::service::{map_torrent_info_response, TorrentInfo};

        #[tokio::test]
        async fn it_should_return_a_torrent_not_found_response_when_the_tracker_returns_the_current_torrent_not_known_response() {
            let tracker_response = Response::new("\"torrent not known\"");

            let result = map_torrent_info_response(tracker_response.into()).await.unwrap_err();

            assert_eq!(result, ServiceError::TorrentNotFound);
        }

        #[tokio::test]
        async fn it_should_return_a_torrent_not_found_response_when_the_tracker_returns_the_future_torrent_not_known_response() {
            // In the future the tracker should return a 4040 response.
            // See: https://github.com/torrust/torrust-tracker/issues/144

            let tracker_response = Response::builder().status(StatusCode::NOT_FOUND).body("").unwrap();

            let result = map_torrent_info_response(tracker_response.into()).await.unwrap_err();

            assert_eq!(result, ServiceError::TorrentNotFound);
        }

        #[tokio::test]
        async fn it_should_return_the_torrent_info_when_the_tracker_returns_the_torrent_info() {
            let body = r#"
                {
                    "info_hash": "4f2ae7294f2c4865c38565f92a077d1591a0dd41",
                    "seeders": 0,
                    "completed": 0,
                    "leechers": 0,
                    "peers": []
                }
            "#;

            let tracker_response = Response::new(body);

            let torrent_info = map_torrent_info_response(tracker_response.into()).await.unwrap();

            assert_eq!(
                torrent_info,
                TorrentInfo {
                    info_hash: "4f2ae7294f2c4865c38565f92a077d1591a0dd41".to_string(),
                    seeders: 0,
                    completed: 0,
                    leechers: 0,
                    peers: vec![]
                }
            );
        }

        #[tokio::test]
        async fn it_should_return_an_internal_server_error_when_the_tracker_response_cannot_be_parsed() {
            let invalid_json_body_for_torrent_info = r#"
                {
                    "field": "value"
                }
            "#;

            let tracker_response = Response::new(invalid_json_body_for_torrent_info);

            let result = map_torrent_info_response(tracker_response.into()).await.unwrap_err();

            assert_eq!(result, ServiceError::InternalServerError);
        }
    }
}
