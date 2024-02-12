use std::time::Duration;

use reqwest::{Error, Response};
pub struct ConnectionInfo {
    /// The URL of the tracker. Eg: <https://tracker:7070> or <udp://tracker:6969>
    pub url: String,
    /// The token used to authenticate with the tracker API.
    pub token: String,
}

impl ConnectionInfo {
    #[must_use]
    pub fn new(url: String, token: String) -> Self {
        Self { url, token }
    }
}

pub struct Client {
    pub connection_info: ConnectionInfo,
    timeout: Duration,
    base_url: String,
}

impl Client {
    #[must_use]
    pub fn new(connection_info: ConnectionInfo) -> Self {
        let base_url = format!("{}/api/v1", connection_info.url);
        Self {
            connection_info,
            timeout: Duration::from_secs(5),
            base_url,
        }
    }

    /// Add a torrent to the tracker whitelist.
    ///
    /// # Errors
    ///
    /// Will return an error if the HTTP request fails.
    pub async fn whitelist_torrent(&self, info_hash: &str) -> Result<Response, Error> {
        let request_url = format!("{}/whitelist/{}", self.base_url, info_hash);

        let client = reqwest::Client::builder().timeout(self.timeout).build()?;

        let params = [("token", &self.connection_info.token)];

        client.post(request_url).query(&params).send().await
    }

    /// Remove a torrent from the tracker whitelist.
    ///
    /// # Errors
    ///
    /// Will return an error if the HTTP request fails.
    pub async fn remove_torrent_from_whitelist(&self, info_hash: &str) -> Result<Response, Error> {
        let request_url = format!("{}/whitelist/{}", self.base_url, info_hash);

        let client = reqwest::Client::builder().timeout(self.timeout).build()?;

        let params = [("token", &self.connection_info.token)];

        client.delete(request_url).query(&params).send().await
    }

    /// Retrieve a new tracker key.
    ///
    /// # Errors
    ///
    /// Will return an error if the HTTP request fails.
    pub async fn retrieve_new_tracker_key(&self, token_valid_seconds: u64) -> Result<Response, Error> {
        let request_url = format!("{}/key/{}", self.base_url, token_valid_seconds);

        let client = reqwest::Client::builder().timeout(self.timeout).build()?;

        let params = [("token", &self.connection_info.token)];

        client.post(request_url).query(&params).send().await
    }

    /// Retrieve the info for a torrent.
    ///
    /// # Errors
    ///
    /// Will return an error if the HTTP request fails.
    pub async fn get_torrent_info(&self, info_hash: &str) -> Result<Response, Error> {
        let request_url = format!("{}/torrent/{}", self.base_url, info_hash);

        let client = reqwest::Client::builder().timeout(self.timeout).build()?;

        let params = [("token", &self.connection_info.token)];

        client.get(request_url).query(&params).send().await
    }
}
