pub mod form;
pub mod responses;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct Settings {
    pub website: Website,
    pub tracker: Tracker,
    pub net: Net,
    pub auth: Auth,
    pub database: Database,
    pub mail: Mail,
    pub image_cache: ImageCache,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct Website {
    pub name: String,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct Tracker {
    pub url: String,
    pub mode: String,
    pub api_url: String,
    pub token: String,
    pub token_valid_seconds: u64,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct Net {
    pub port: u64,
    pub base_url: Option<String>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct Auth {
    pub email_on_signup: String,
    pub min_password_length: u64,
    pub max_password_length: u64,
    pub secret_key: String,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct Database {
    pub connect_url: String,
    pub torrent_info_update_interval: u64,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct Mail {
    pub email_verification_enabled: bool,
    pub from: String,
    pub reply_to: String,
    pub username: String,
    pub password: String,
    pub server: String,
    pub port: u64,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct ImageCache {
    pub max_request_timeout_ms: u64,
    pub capacity: u64,
    pub entry_size_limit: u64,
    pub user_quota_period_seconds: u64,
    pub user_quota_bytes: u64,
}
