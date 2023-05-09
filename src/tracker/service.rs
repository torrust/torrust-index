use std::sync::Arc;

use super::api::{Client, ConnectionInfo};
use crate::config::Configuration;
use crate::databases::database::Database;
use crate::errors::ServiceError;
use crate::models::tracker_key::TrackerKey;

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
}
