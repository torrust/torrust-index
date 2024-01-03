use std::sync::Arc;

use derive_more::{Display, Error};
use hyper::StatusCode;
use log::{debug, error};
use serde::{Deserialize, Serialize};

use super::api::{Client, ConnectionInfo};
use crate::config::Configuration;
use crate::databases::database::Database;
use crate::models::tracker_key::TrackerKey;
use crate::models::user::UserId;

#[derive(Debug, Display, PartialEq, Eq, Error)]
#[allow(dead_code)]
pub enum TrackerAPIError {
    #[display(fmt = "Error with tracker connection.")]
    TrackerOffline,

    #[display(fmt = "Invalid token for tracker API. Check the tracker token in settings.")]
    InvalidToken,

    #[display(fmt = "Tracker returned an internal server error.")]
    InternalServerError,

    #[display(fmt = "Tracker returned an unexpected response status.")]
    UnexpectedResponseStatus,

    #[display(fmt = "Could not save the newly generated user key into the database.")]
    CannotSaveUserKey,

    #[display(fmt = "Torrent not found.")]
    TorrentNotFound,

    #[display(fmt = "Expected body in tracker response, received empty body.")]
    MissingResponseBody,

    #[display(fmt = "Expected body in tracker response, received empty body.")]
    FailedToParseTrackerResponse { body: String },
}

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
    pub async fn whitelist_info_hash(&self, info_hash: String) -> Result<(), TrackerAPIError> {
        debug!(target: "tracker-service", "add to whitelist: {info_hash}");

        let maybe_response = self.api_client.whitelist_torrent(&info_hash).await;

        debug!(target: "tracker-service", "add to whitelist response result: {:?}", maybe_response);

        match maybe_response {
            Ok(response) => {
                let status: StatusCode = response.status();

                let body = response.text().await.map_err(|_| {
                    error!(target: "tracker-service", "response without body");
                    TrackerAPIError::MissingResponseBody
                })?;

                match status {
                    StatusCode::OK => Ok(()),
                    StatusCode::INTERNAL_SERVER_ERROR => {
                        if body == "Unhandled rejection: Err { reason: \"token not valid\" }" {
                            Err(TrackerAPIError::InvalidToken)
                        } else {
                            error!(target: "tracker-service", "add to whitelist 500 response: status {status}, body: {body}");
                            Err(TrackerAPIError::InternalServerError)
                        }
                    }
                    _ => {
                        error!(target: "tracker-service", "add to whitelist unexpected response: status {status}, body: {body}");
                        Err(TrackerAPIError::UnexpectedResponseStatus)
                    }
                }
            }
            Err(_) => Err(TrackerAPIError::TrackerOffline),
        }
    }

    /// Remove a torrent from the tracker whitelist.
    ///
    /// # Errors
    ///
    /// Will return an error if the HTTP request failed (for example if the
    /// tracker API is offline) or if the tracker API returned an error.
    pub async fn remove_info_hash_from_whitelist(&self, info_hash: String) -> Result<(), TrackerAPIError> {
        debug!(target: "tracker-service", "remove from whitelist: {info_hash}");

        let maybe_response = self.api_client.remove_torrent_from_whitelist(&info_hash).await;

        debug!(target: "tracker-service", "remove from whitelist response result: {:?}", maybe_response);

        match maybe_response {
            Ok(response) => {
                let status: StatusCode = response.status();

                let body = response.text().await.map_err(|_| {
                    error!(target: "tracker-service", "response without body");
                    TrackerAPIError::MissingResponseBody
                })?;

                match status {
                    StatusCode::OK => Ok(()),
                    StatusCode::INTERNAL_SERVER_ERROR => {
                        if body == Self::invalid_token_body() {
                            Err(TrackerAPIError::InvalidToken)
                        } else {
                            error!(target: "tracker-service", "remove from whitelist 500 response: status {status}, body: {body}");
                            Err(TrackerAPIError::InternalServerError)
                        }
                    }
                    _ => {
                        error!(target: "tracker-service", "remove from whitelist unexpected response: status {status}, body: {body}");
                        Err(TrackerAPIError::UnexpectedResponseStatus)
                    }
                }
            }
            Err(_) => Err(TrackerAPIError::TrackerOffline),
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
    pub async fn get_personal_announce_url(&self, user_id: UserId) -> Result<String, TrackerAPIError> {
        debug!(target: "tracker-service", "get personal announce url for user: {user_id}");

        let tracker_key = self.database.get_user_tracker_key(user_id).await;

        match tracker_key {
            Some(tracker_key) => Ok(self.announce_url_with_key(&tracker_key)),
            None => match self.retrieve_new_tracker_key(user_id).await {
                Ok(new_tracker_key) => Ok(self.announce_url_with_key(&new_tracker_key)),
                Err(_) => Err(TrackerAPIError::TrackerOffline),
            },
        }
    }

    /// Get torrent info from tracker.
    ///
    /// # Errors
    ///
    /// Will return an error if the HTTP request to get torrent info fails or
    /// if the response cannot be parsed.
    pub async fn get_torrent_info(&self, info_hash: &str) -> Result<TorrentInfo, TrackerAPIError> {
        debug!(target: "tracker-service", "get torrent info: {info_hash}");

        let maybe_response = self.api_client.get_torrent_info(info_hash).await;

        debug!(target: "tracker-service", "get torrent info response result: {:?}", maybe_response);

        match maybe_response {
            Ok(response) => {
                let status: StatusCode = response.status();

                let body = response.text().await.map_err(|_| {
                    error!(target: "tracker-service", "response without body");
                    TrackerAPIError::MissingResponseBody
                })?;

                match status {
                    StatusCode::NOT_FOUND => Err(TrackerAPIError::TorrentNotFound),
                    StatusCode::OK => {
                        if body == Self::torrent_not_known_body() {
                            // todo: temporary fix. the service should return a 404 (StatusCode::NOT_FOUND).
                            return Err(TrackerAPIError::TorrentNotFound);
                        }

                        serde_json::from_str(&body).map_err(|e| {
                            error!(
                                target: "tracker-service", "Failed to parse torrent info from tracker response. Body: {}, Error: {}",
                                body, e
                            );
                            TrackerAPIError::FailedToParseTrackerResponse { body }
                        })
                    }
                    StatusCode::INTERNAL_SERVER_ERROR => {
                        if body == Self::invalid_token_body() {
                            Err(TrackerAPIError::InvalidToken)
                        } else {
                            error!(target: "tracker-service", "get torrent info 500 response: status {status}, body: {body}");
                            Err(TrackerAPIError::InternalServerError)
                        }
                    }
                    _ => {
                        error!(target: "tracker-service", "get torrent info unhandled response: status {status}, body: {body}");
                        Err(TrackerAPIError::UnexpectedResponseStatus)
                    }
                }
            }
            Err(_) => Err(TrackerAPIError::TrackerOffline),
        }
    }

    /// Issue a new tracker key from tracker.
    async fn retrieve_new_tracker_key(&self, user_id: i64) -> Result<TrackerKey, TrackerAPIError> {
        debug!(target: "tracker-service", "retrieve key: {user_id}");

        let maybe_response = self.api_client.retrieve_new_tracker_key(self.token_valid_seconds).await;

        debug!(target: "tracker-service", "retrieve key response result: {:?}", maybe_response);

        match maybe_response {
            Ok(response) => {
                let status: StatusCode = response.status();

                let body = response.text().await.map_err(|_| {
                    error!(target: "tracker-service", "response without body");
                    TrackerAPIError::MissingResponseBody
                })?;

                match status {
                    StatusCode::OK => {
                        // Parse tracker key from response
                        let tracker_key =
                            serde_json::from_str(&body).map_err(|_| TrackerAPIError::FailedToParseTrackerResponse { body })?;

                        // Add tracker key to database (tied to a user)
                        self.database
                            .add_tracker_key(user_id, &tracker_key)
                            .await
                            .map_err(|_| TrackerAPIError::CannotSaveUserKey)?;

                        Ok(tracker_key)
                    }
                    StatusCode::INTERNAL_SERVER_ERROR => {
                        if body == Self::invalid_token_body() {
                            Err(TrackerAPIError::InvalidToken)
                        } else {
                            error!(target: "tracker-service", "retrieve key 500 response: status {status}, body: {body}");
                            Err(TrackerAPIError::InternalServerError)
                        }
                    }
                    _ => {
                        error!(target: "tracker-service", " retrieve key unexpected response: status {status}, body: {body}");
                        Err(TrackerAPIError::UnexpectedResponseStatus)
                    }
                }
            }
            Err(_) => Err(TrackerAPIError::TrackerOffline),
        }
    }

    /// It builds the announce url appending the user tracker key.
    /// Eg: <https://tracker:7070/USER_TRACKER_KEY>
    fn announce_url_with_key(&self, tracker_key: &TrackerKey) -> String {
        format!("{}/{}", self.tracker_url, tracker_key.key)
    }

    fn invalid_token_body() -> String {
        "Unhandled rejection: Err { reason: \"token not valid\" }".to_string()
    }

    fn torrent_not_known_body() -> String {
        "\"torrent not known\"".to_string()
    }
}
