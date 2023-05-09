use reqwest::{Error, Response};
pub struct ApiConnectionInfo {
    pub url: String,
    pub token: String,
}

impl ApiConnectionInfo {
    pub fn new(url: String, token: String) -> Self {
        Self { url, token }
    }
}

pub struct ApiClient {
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

    pub async fn retrieve_new_tracker_key(&self, token_valid_seconds: u64) -> Result<Response, Error> {
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
