use std::time::Duration;

use reqwest::{Error, Response};
pub struct ConnectionInfo {
    /// The URL of the tracker API. Eg: <https://tracker:1212>.
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

const TOKEN_PARAM_NAME: &str = "token";

pub struct Client {
    pub connection_info: ConnectionInfo,
    api_base_url: String,
    client: reqwest::Client,
    token_param: [(String, String); 1],
}

impl Client {
    /// # Errors
    ///
    /// Will fails if it can't build a HTTP client with a timeout.
    pub fn new(connection_info: ConnectionInfo) -> Result<Self, Error> {
        let base_url = format!("{}/api/v1", connection_info.url);
        let client = reqwest::Client::builder().timeout(Duration::from_secs(5)).build()?;
        let token_param = [(TOKEN_PARAM_NAME.to_string(), connection_info.token.to_string())];

        Ok(Self {
            connection_info,
            api_base_url: base_url,
            client,
            token_param,
        })
    }

    /// Add a torrent to the tracker whitelist.
    ///
    /// # Errors
    ///
    /// Will return an error if the HTTP request fails.
    pub async fn whitelist_torrent(&self, info_hash: &str) -> Result<Response, Error> {
        let request_url = format!("{}/whitelist/{}", self.api_base_url, info_hash);

        self.client.post(request_url).query(&self.token_param).send().await
    }

    /// Remove a torrent from the tracker whitelist.
    ///
    /// # Errors
    ///
    /// Will return an error if the HTTP request fails.
    pub async fn remove_torrent_from_whitelist(&self, info_hash: &str) -> Result<Response, Error> {
        let request_url = format!("{}/whitelist/{}", self.api_base_url, info_hash);

        self.client.delete(request_url).query(&self.token_param).send().await
    }

    /// Retrieve a new tracker key.
    ///
    /// # Errors
    ///
    /// Will return an error if the HTTP request fails.
    pub async fn retrieve_new_tracker_key(&self, token_valid_seconds: u64) -> Result<Response, Error> {
        let request_url = format!("{}/key/{}", self.api_base_url, token_valid_seconds);

        self.client.post(request_url).query(&self.token_param).send().await
    }

    /// Retrieve the info for one torrent.
    ///
    /// # Errors
    ///
    /// Will return an error if the HTTP request fails.
    pub async fn get_torrent_info(&self, info_hash: &str) -> Result<Response, Error> {
        let request_url = format!("{}/torrent/{}", self.api_base_url, info_hash);

        self.client.get(request_url).query(&self.token_param).send().await
    }

    /// Retrieve the info for multiple torrents at the same time.
    ///
    /// # Errors
    ///
    /// Will return an error if the HTTP request fails.
    pub async fn get_torrents_info(&self, info_hashes: &[String]) -> Result<Response, Error> {
        let request_url = format!("{}/torrents", self.api_base_url);

        let mut query_params: Vec<(String, String)> = Vec::with_capacity(info_hashes.len() + 1);

        query_params.push((TOKEN_PARAM_NAME.to_string(), self.connection_info.token.clone()));

        for info_hash in info_hashes {
            query_params.push(("info_hash".to_string(), info_hash.clone()));
        }

        self.client.get(request_url).query(&query_params).send().await
    }
}
